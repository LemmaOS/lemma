import "fake-indexeddb/auto";
import Dexie from "dexie";
import { beforeEach, describe, expect, it } from "vitest";

import {
    closeDb,
    conversationToRow,
    deleteConversationCascade,
    getCursor,
    LemmaDb,
    listArchived,
    listConversations,
    listMessages,
    openDb,
    replaceArchived,
    setCursor,
    upsertConversations,
    upsertMessages,
    type ConversationRow,
    type MessageRow,
} from "@/lib/db";

function conv(
    id: string,
    over: Partial<ConversationRow> = {},
): ConversationRow {
    return {
        id,
        title: id,
        status: 1,
        archivedAtMs: null,
        messageCount: 0,
        createdAtMs: 1000,
        updatedAtMs: 1000,
        syncSeq: "1",
        ...over,
    };
}

function msg(
    id: string,
    convId: string,
    over: Partial<MessageRow> = {},
): MessageRow {
    return {
        id,
        conversationId: convId,
        role: "user",
        content: id,
        providerId: "",
        model: "",
        status: 4,
        createdAtMs: 1000,
        seq: 0,
        syncSeq: "1",
        ...over,
    };
}

describe("db", () => {
    let db: LemmaDb;

    beforeEach(async () => {
        closeDb();
        db = openDb("test-user");
        await db.delete();
        await db.open();
    });

    it("游标默认 0，写入后读回", async () => {
        expect(await getCursor(db)).toBe(0n);
        await setCursor(db, 42n);
        expect(await getCursor(db)).toBe(42n);
    });

    it("LWW：低 syncSeq 不覆盖高 syncSeq", async () => {
        await upsertConversations(db, [
            conv("c1", { title: "新", syncSeq: "5" }),
        ]);
        await upsertConversations(db, [
            conv("c1", { title: "旧", syncSeq: "3" }),
        ]);
        const row = await db.conversations.get("c1");
        expect(row?.title).toBe("新");
    });

    it("消息按会话 + seq 正序", async () => {
        await upsertMessages(db, [
            msg("m2", "c1", { seq: 2 }),
            msg("m1", "c1", { seq: 1 }),
            msg("m3", "c2", { seq: 1 }),
        ]);
        const rows = await listMessages(db, "c1");
        expect(rows.map((r) => r.id)).toEqual(["m1", "m2"]);
    });

    it("seq 优先于 createdAtMs（回归：同事务插入顺序颠倒）", async () => {
        await upsertMessages(db, [
            msg("m1", "c1", { seq: 1, createdAtMs: 2000 }),
            msg("m2", "c1", { seq: 2, createdAtMs: 1000 }),
        ]);
        const rows = await listMessages(db, "c1");
        expect(rows.map((r) => r.id)).toEqual(["m1", "m2"]);
    });

    it("归档全量刷新：清掉不在新列表里的归档行", async () => {
        await upsertConversations(db, [
            conv("a1", { status: 2, archivedAtMs: 1000 }),
            conv("a2", { status: 2, archivedAtMs: 2000 }),
        ]);
        await replaceArchived(db, [
            conv("a2", { status: 2, archivedAtMs: 2000, syncSeq: "2" }),
        ]);
        const archived = await listArchived(db);
        expect(archived.map((r) => r.id)).toEqual(["a2"]);
    });

    it("彻底删除：会话连同消息一起清", async () => {
        await upsertConversations(db, [conv("c1")]);
        await upsertMessages(db, [msg("m1", "c1"), msg("m2", "c1")]);
        await deleteConversationCascade(db, "c1");
        expect(await db.conversations.get("c1")).toBeUndefined();
        expect(await listMessages(db, "c1")).toEqual([]);
    });

    it("活跃列表排除归档，按更新时间倒序", async () => {
        await upsertConversations(db, [
            conv("c1", { updatedAtMs: 1000 }),
            conv("c2", { updatedAtMs: 3000 }),
            conv("a1", { status: 2, archivedAtMs: 5000 }),
        ]);
        const rows = await listConversations(db);
        expect(rows.map((r) => r.id)).toEqual(["c2", "c1"]);
    });

    it("proto 转换：Timestamp 转毫秒、bigint 转字符串", () => {
        const row = conversationToRow(
            {
                $typeName: "lemma.v1.Conversation",
                id: "c1",
                title: "t",
                status: 1,
                archivedAt: undefined,
                messageCount: 3,
                createdAt: {
                    $typeName: "google.protobuf.Timestamp",
                    seconds: 1700000000n,
                    nanos: 0,
                },
                updatedAt: {
                    $typeName: "google.protobuf.Timestamp",
                    seconds: 1700000001n,
                    nanos: 0,
                },
            },
            9n,
        );
        expect(row.createdAtMs).toBe(1700000000000);
        expect(row.updatedAtMs).toBe(1700000001000);
        expect(row.syncSeq).toBe("9");
    });

    it("LWW：消息同样拒绝低 syncSeq 回滚", async () => {
        await upsertMessages(db, [
            msg("m1", "c1", { content: "新", syncSeq: "5" }),
        ]);
        await upsertMessages(db, [
            msg("m1", "c1", { content: "旧", syncSeq: "3" }),
        ]);
        expect((await db.messages.get("m1"))?.content).toBe("新");
    });

    it("归档列表容忍缺失 archivedAtMs", async () => {
        await upsertConversations(db, [
            conv("a1", { status: 2, archivedAtMs: null }),
            conv("a2", { status: 2, archivedAtMs: 1000 }),
        ]);
        const rows = await listArchived(db);
        expect(rows.map((r) => r.id)).toEqual(["a2", "a1"]);
    });

    it("v2 升级清空旧索引下的消息和游标", async () => {
        // Simulate a cache written before the v2 re-index shipped.
        const legacy = new Dexie("lemma-upgrade-user");
        legacy.version(1).stores({
            conversations: "id, updatedAtMs",
            messages: "id, [conversationId+createdAtMs]",
            meta: "key",
        });
        await legacy.open();
        await legacy
            .table("messages")
            .put({ id: "m1", conversationId: "c1", createdAtMs: 1 });
        await legacy.table("meta").put({ key: "cursor", value: "9" });
        legacy.close();

        const upgraded = openDb("upgrade-user");
        await upgraded.open();
        expect(await upgraded.messages.count()).toBe(0);
        expect(await getCursor(upgraded)).toBe(0n);
        closeDb();
        await Dexie.delete("lemma-upgrade-user");
    });
});
