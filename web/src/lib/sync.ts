import type { PullResponse } from "@/gen/lemma/v1/sync_pb";
import { syncClient } from "@/lib/clients";
import {
    conversationToRow,
    deleteConversationCascade,
    deleteMessagesOf,
    getCursor,
    getDb,
    type LemmaDb,
    messageToRow,
    pruneActiveExcept,
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
// Concurrent pullAll calls share this one in-flight pull.
let pulling: Promise<void> | null = null;

type SyncListener = () => void;
const listeners = new Set<SyncListener>();

/** Subscribes to post-pull notifications; call the result to unsubscribe. */
export function onSynced(cb: SyncListener): () => void {
    listeners.add(cb);
    return () => {
        listeners.delete(cb);
    };
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

/**
 * Applies one pull page to the cache: upserts changed rows, prunes cached
 * conversations absent from the server's active roster, full-refreshes the
 * archived list, and drops cached messages of archived conversations.
 */
export async function applyPull(db: LemmaDb, res: PullResponse): Promise<void> {
    const convRows = res.conversations.flatMap((e) =>
        e.conversation ? [conversationToRow(e.conversation, e.syncSeq)] : [],
    );
    const msgRows = res.messages.flatMap((e) =>
        e.message ? [messageToRow(e.message, e.syncSeq)] : [],
    );
    // Roster entries carry no per-entry syncSeq; stamping them with 0 lets
    // the LWW check keep any newer row from the incremental feed.
    const archivedRows = res.archived.map((c) => conversationToRow(c, 0n));
    await upsertConversations(db, convRows);
    await upsertMessages(db, msgRows);
    const zombieIds = await pruneActiveExcept(
        db,
        new Set(res.active.map((c) => c.id)),
    );
    for (const id of zombieIds) await deleteConversationCascade(db, id);
    const removed = await replaceArchived(db, archivedRows);
    for (const id of removed) await deleteConversationCascade(db, id);
    await deleteMessagesOf(
        db,
        res.archived.map((c) => c.id),
    );
}

/** Pulls all pending changes page by page and persists the new cursor. */
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
            for (const cb of listeners) cb();
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

async function watchLoop(): Promise<void> {
    let backoff = BACKOFF_START_MS;
    while (running) {
        try {
            watchAbort = new AbortController();
            const stream = syncClient.watch({}, { signal: watchAbort.signal });
            useSyncStatus.getState().setOnline(true);
            backoff = BACKOFF_START_MS;
            // Catch up before consuming hints so nothing that changed
            // while the stream was down is missed.
            await pullAll();
            for await (const res of stream) {
                // Non-hint events are heartbeats keeping proxies from
                // killing the idle stream.
                if (res.kind.case !== "hint") continue;
                const db = getDb();
                if (!db) continue;
                // A hint only says "something changed"; pull to find out.
                // Hints at or below the cursor are already applied.
                if (res.kind.value.syncSeq > (await getCursor(db))) {
                    await pullAll();
                }
            }
        } catch {
            if (!running) break;
            useSyncStatus.getState().setOnline(false);
        }
        if (!running) break;
        await sleep(backoff);
        backoff = Math.min(backoff * 2, BACKOFF_MAX_MS);
    }
}

/** Starts the background watch loop; safe to call repeatedly. */
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
