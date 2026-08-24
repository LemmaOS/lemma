import type { PullResponse } from "@/gen/lemma/v1/sync_pb";
import { syncClient } from "@/lib/clients";
import {
    conversationToRow,
    deleteConversationCascade,
    getCursor,
    getDb,
    type LemmaDb,
    messageToRow,
    replaceArchived,
    setCursor,
    upsertConversations,
    upsertMessages,
} from "@/lib/db";
import { useSyncStatus } from "@/stores/sync";

const BACKOFF_START_MS = 1000;
const BACKOFF_MAX_MS = 30000;

let running = false;
let watchAbort: AbortController | null = null;
let pulling: Promise<void> | null = null;

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

/** 应用一页 Pull：LWW 落库 + 归档全量刷新（顺带清理彻底删除的残留） */
export async function applyPull(db: LemmaDb, res: PullResponse): Promise<void> {
    const convRows = res.conversations.flatMap((e) =>
        e.conversation ? [conversationToRow(e.conversation, e.syncSeq)] : [],
    );
    const msgRows = res.messages.flatMap((e) =>
        e.message ? [messageToRow(e.message, e.syncSeq)] : [],
    );
    // 归档列表不带 syncSeq，用 0 占位——增量条目的 seq 必更大，不会误覆盖
    const archivedRows = res.archived.map((c) => conversationToRow(c, 0n));
    await upsertConversations(db, convRows);
    await upsertMessages(db, msgRows);
    const removed = await replaceArchived(db, archivedRows);
    for (const id of removed) await deleteConversationCascade(db, id);
}

/** 游标循环补拉直到追上服务端；并发触发合并成同一次 */
export async function pullAll(): Promise<void> {
    const db = getDb();
    if (!db) return;
    pulling ??= (async () => {
        useSyncStatus.getState().setSyncing(true);
        try {
            let after = await getCursor(db);
            for (;;) {
                const res = await syncClient.pull({ after });
                await applyPull(db, res);
                after = res.nextAfter;
                if (!res.hasMore) break;
            }
            await setCursor(db, after);
            useSyncStatus.getState().setOnline(true);
        } finally {
            useSyncStatus.getState().setSyncing(false);
        }
    })();
    try {
        await pulling;
    } finally {
        pulling = null;
    }
}

/** watch 循环：连上先补拉，hint 落后再拉；断流指数退避重连 */
async function watchLoop(): Promise<void> {
    let backoff = BACKOFF_START_MS;
    while (running) {
        try {
            watchAbort = new AbortController();
            const stream = syncClient.watch({}, { signal: watchAbort.signal });
            useSyncStatus.getState().setOnline(true);
            backoff = BACKOFF_START_MS;
            await pullAll();
            for await (const res of stream) {
                if (res.kind.case !== "hint") continue;
                const db = getDb();
                if (!db) continue;
                if (res.kind.value.syncSeq > (await getCursor(db))) {
                    await pullAll();
                }
            }
        } catch {
            // stopSync 主动掐断不算故障
            if (!running) break;
            useSyncStatus.getState().setOnline(false);
        }
        if (!running) break;
        await sleep(backoff);
        backoff = Math.min(backoff * 2, BACKOFF_MAX_MS);
    }
}

export function startSync(): void {
    if (running) return;
    running = true;
    void watchLoop();
}

export function stopSync(): void {
    running = false;
    watchAbort?.abort();
    watchAbort = null;
}
