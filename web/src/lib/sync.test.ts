import "fake-indexeddb/auto";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/clients", () => ({
    syncClient: { pull: vi.fn(), watch: vi.fn() },
}));

import type { PullResponse, WatchResponse } from "@/gen/lemma/v1/sync_pb";
import { syncClient } from "@/lib/clients";
import {
    closeDb,
    conversationToRow,
    getCursor,
    type LemmaDb,
    listMessages,
    messageToRow,
    openDb,
    upsertConversations,
    upsertMessages,
} from "@/lib/db";
import { applyPull, pullAll, startSync, stopSync } from "@/lib/sync";
import { useSyncStatus } from "@/stores/sync";

const pullMock = syncClient.pull as unknown as ReturnType<typeof vi.fn>;
const watchMock = syncClient.watch as unknown as ReturnType<typeof vi.fn>;

function convProto(id: string, status: 1 | 2 = 1) {
    return {
        $typeName: "lemma.v1.Conversation" as const,
        id,
        title: `t-${id}`,
        status,
        archivedAt: undefined,
        messageCount: 0,
        createdAt: undefined,
        updatedAt: undefined,
    };
}

function convEntry(id: string, syncSeq: bigint, status: 1 | 2 = 1) {
    return {
        $typeName: "lemma.v1.SyncConversation" as const,
        conversation: convProto(id, status),
        syncSeq,
    };
}

function pullRes(over: Partial<PullResponse>): PullResponse {
    return {
        conversations: [],
        messages: [],
        archived: [],
        active: [],
        nextAfter: 0n,
        hasMore: false,
        ...over,
    } as PullResponse;
}

function hintRes(seq: bigint): WatchResponse {
    return {
        kind: { case: "hint", value: { syncSeq: seq } },
    } as WatchResponse;
}

// Yields the events, then hangs forever like a real long-lived stream.
function streamOf(events: WatchResponse[]): AsyncIterable<WatchResponse> {
    return (async function* () {
        for (const e of events) yield e;
        await new Promise(() => {});
    })();
}

// Rejects on the first read, simulating a connection that drops instantly.
function errorStream(): AsyncIterable<WatchResponse> {
    return {
        [Symbol.asyncIterator]() {
            return {
                next: () => Promise.reject(new Error("connection lost")),
            };
        },
    };
}

async function waitFor(
    cond: () => boolean | Promise<boolean>,
    timeoutMs = 3000,
): Promise<void> {
    const start = Date.now();
    for (;;) {
        if (await cond()) return;
        if (Date.now() - start > timeoutMs) throw new Error("waitFor timeout");
        await new Promise((r) => setTimeout(r, 20));
    }
}

describe("sync", () => {
    let db: LemmaDb;

    beforeEach(async () => {
        closeDb();
        db = openDb("sync-test");
        await db.delete();
        await db.open();
    });

    afterEach(() => {
        stopSync();
        vi.clearAllMocks();
    });

    it("pullAll 循环分页直到 hasMore=false，游标持久化", async () => {
        pullMock
            .mockResolvedValueOnce(
                pullRes({
                    conversations: [convEntry("c1", 1n)],
                    nextAfter: 1n,
                    hasMore: true,
                    active: [convProto("c1", 1), convProto("c2", 1)],
                }),
            )
            .mockResolvedValueOnce(
                pullRes({
                    conversations: [convEntry("c2", 2n)],
                    nextAfter: 2n,
                    hasMore: false,
                    active: [convProto("c1", 1), convProto("c2", 1)],
                }),
            );

        await pullAll();

        expect(pullMock).toHaveBeenCalledTimes(2);
        expect(pullMock.mock.calls[0][0]).toEqual({ after: 0n });
        expect(pullMock.mock.calls[1][0]).toEqual({ after: 1n });
        expect(await getCursor(db)).toBe(2n);
        expect((await db.conversations.toArray()).map((r) => r.id)).toEqual([
            "c1",
            "c2",
        ]);
    });

    it("applyPull：归档全量刷新清理彻底删除的会话及其消息", async () => {
        await upsertConversations(db, [
            conversationToRow(convProto("a1", 2), 3n),
        ]);
        await upsertMessages(db, [
            messageToRow(
                {
                    $typeName: "lemma.v1.Message",
                    id: "m1",
                    conversationId: "a1",
                    role: "user",
                    content: "x",
                    providerId: "",
                    model: "",
                    seq: 1n,
                    status: 4,
                    createdAt: undefined,
                    updatedAt: undefined,
                } as never,
                3n,
            ),
        ]);

        await applyPull(db, pullRes({ archived: [convProto("a2", 2)] }));

        expect(await db.conversations.get("a1")).toBeUndefined();
        expect(await listMessages(db, "a1")).toEqual([]);
        expect(await db.conversations.get("a2")).toBeDefined();
    });

    it("applyPull：归档会话的缓存消息被清空，活跃会话不受影响", async () => {
        await upsertConversations(db, [
            conversationToRow(convProto("a1", 1), 3n),
            conversationToRow(convProto("live", 1), 3n),
        ]);
        const msg = (id: string, cid: string) =>
            messageToRow(
                {
                    $typeName: "lemma.v1.Message",
                    id,
                    conversationId: cid,
                    role: "user",
                    content: "x",
                    providerId: "",
                    model: "",
                    seq: 1n,
                    status: 4,
                    createdAt: undefined,
                    updatedAt: undefined,
                } as never,
                3n,
            );
        await upsertMessages(db, [msg("m1", "a1"), msg("m2", "live")]);

        await applyPull(
            db,
            pullRes({
                conversations: [convEntry("a1", 6n, 2)],
                archived: [convProto("a1", 2)],
                active: [convProto("live", 1)],
            }),
        );

        expect((await db.conversations.get("a1"))?.status).toBe(2);
        expect(await db.messages.get("m1")).toBeUndefined();
        expect(await db.messages.get("m2")).toBeDefined();
    });

    it("watch 连接后先补拉，hint 落后时再拉", async () => {
        let calls = 0;
        pullMock.mockImplementation(() => {
            calls += 1;
            if (calls === 1) return Promise.resolve(pullRes({ nextAfter: 0n }));
            return Promise.resolve(
                pullRes({
                    conversations: [convEntry("c9", 5n)],
                    nextAfter: 5n,
                    active: [convProto("c9", 1)],
                }),
            );
        });
        watchMock.mockImplementation(() => streamOf([hintRes(5n)]));

        startSync();
        await waitFor(async () => (await getCursor(db)) === 5n);

        expect(pullMock).toHaveBeenCalledTimes(2);
        expect(await db.conversations.get("c9")).toBeDefined();
    });

    it("断流后指数退避重连并恢复在线", async () => {
        pullMock.mockResolvedValue(pullRes({}));
        let watchCalls = 0;
        watchMock.mockImplementation(() => {
            watchCalls += 1;
            return watchCalls === 1 ? errorStream() : streamOf([]);
        });

        startSync();
        await waitFor(() => useSyncStatus.getState().online === false);
        // The reconnect only happens after the 1s starting backoff.
        await waitFor(() => useSyncStatus.getState().online === true);

        expect(watchCalls).toBe(2);
    });

    it("applyPull 按活跃名单清理僵尸会话及其消息", async () => {
        const db = openDb("sync-test");
        await db.delete();
        await db.open();
        await upsertConversations(db, [
            conversationToRow(convProto("zombie", 1), 5n),
            conversationToRow(convProto("live", 1), 5n),
        ]);
        await upsertMessages(db, [
            messageToRow(
                {
                    $typeName: "lemma.v1.Message",
                    id: "z1",
                    conversationId: "zombie",
                    role: "user",
                    content: "x",
                    providerId: "",
                    model: "",
                    status: 2,
                    seq: 1n,
                    createdAt: undefined,
                } as never,
                5n,
            ),
        ]);

        await applyPull(db, pullRes({ active: [convProto("live", 1)] }));

        expect(await db.conversations.get("zombie")).toBeUndefined();
        expect(await db.messages.get("z1")).toBeUndefined();
        expect(await db.conversations.get("live")).toBeDefined();
        closeDb();
    });
});
