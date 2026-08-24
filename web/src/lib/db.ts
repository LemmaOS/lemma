import { timestampDate } from "@bufbuild/protobuf/wkt";
import Dexie, { type EntityTable } from "dexie";

import type { Conversation, Message } from "@/gen/lemma/v1/conversation_pb";

// 缓存行：proto 实体拍平 + 同步元数据；Timestamp 转毫秒，bigint 转字符串
export interface ConversationRow {
    id: string;
    title: string;
    status: number; // ConversationStatus
    archivedAtMs: number | null;
    messageCount: number;
    createdAtMs: number;
    updatedAtMs: number;
    syncSeq: string;
}

export interface MessageRow {
    id: string;
    conversationId: string;
    role: string;
    content: string;
    providerId: string;
    model: string;
    status: number; // MessageStatus
    createdAtMs: number;
    // 会话内单调序号（插入顺序）；number 安全到 2^53，现实不可能超
    seq: number;
    syncSeq: string;
}

export interface MetaRow {
    key: string;
    value: string;
}

export class LemmaDb extends Dexie {
    conversations!: EntityTable<ConversationRow, "id">;
    messages!: EntityTable<MessageRow, "id">;
    meta!: EntityTable<MetaRow, "key">;

    constructor(userId: string) {
        super(`lemma-${userId}`);
        this.version(1).stores({
            conversations: "id, updatedAtMs",
            messages: "id, [conversationId+createdAtMs]",
            meta: "key",
        });
        this.version(2)
            .stores({
                conversations: "id, updatedAtMs",
                messages: "id, [conversationId+seq]",
                meta: "key",
            })
            .upgrade(async (tx) => {
                await tx.table("messages").clear();
                await tx.table("meta").delete("cursor");
            });
    }
}

// 当前打开的库（模块级单例；切账号时换库）
let current: LemmaDb | null = null;

export function openDb(userId: string): LemmaDb {
    if (current?.name === `lemma-${userId}`) return current;
    current?.close();
    current = new LemmaDb(userId);
    return current;
}

export function getDb(): LemmaDb | null {
    return current;
}

export function closeDb(): void {
    current?.close();
    current = null;
}

const ms = (ts: Conversation["updatedAt"]): number =>
    ts ? timestampDate(ts).getTime() : 0;

export function conversationToRow(
    c: Conversation,
    syncSeq: bigint,
): ConversationRow {
    return {
        id: c.id,
        title: c.title,
        status: c.status,
        archivedAtMs: c.archivedAt ? ms(c.archivedAt) : null,
        messageCount: c.messageCount,
        createdAtMs: ms(c.createdAt),
        updatedAtMs: ms(c.updatedAt),
        syncSeq: syncSeq.toString(),
    };
}

export function messageToRow(m: Message, syncSeq: bigint): MessageRow {
    return {
        id: m.id,
        conversationId: m.conversationId,
        role: m.role,
        content: m.content,
        providerId: m.providerId,
        model: m.model,
        status: m.status,
        seq: Number(m.seq ?? 0n),
        createdAtMs: ms(m.createdAt),
        syncSeq: syncSeq.toString(),
    };
}

const CURSOR_KEY = "cursor";

export async function getCursor(db: LemmaDb): Promise<bigint> {
    const row = await db.meta.get(CURSOR_KEY);
    return row ? BigInt(row.value) : 0n;
}

export async function setCursor(db: LemmaDb, seq: bigint): Promise<void> {
    await db.meta.put({ key: CURSOR_KEY, value: seq.toString() });
}

/** 活跃会话列表，按更新时间倒序 */
export async function listConversations(
    db: LemmaDb,
): Promise<ConversationRow[]> {
    // status: 1=ACTIVE（0 是 UNSPECIFIED，2 是 ARCHIVED）
    const rows = await db.conversations.toArray();
    return rows
        .filter((r) => r.status !== 2)
        .sort((a, b) => b.updatedAtMs - a.updatedAtMs);
}

/** 归档会话列表，按归档时间倒序 */
export async function listArchived(db: LemmaDb): Promise<ConversationRow[]> {
    const rows = await db.conversations.toArray();
    return rows
        .filter((r) => r.status === 2)
        .sort((a, b) => (b.archivedAtMs ?? 0) - (a.archivedAtMs ?? 0));
}

/** 会话消息，按会话内序号正序 */
export async function listMessages(
    db: LemmaDb,
    conversationId: string,
): Promise<MessageRow[]> {
    return db.messages
        .where("[conversationId+seq]")
        .between([conversationId, 0], [conversationId, Infinity])
        .toArray();
}

/** LWW 写入：只接受 syncSeq 更大或相等的行（相等允许写穿透后被同步覆盖语义外的重放） */
export async function upsertConversations(
    db: LemmaDb,
    rows: ConversationRow[],
): Promise<void> {
    await db.transaction("rw", db.conversations, async () => {
        for (const row of rows) {
            const existing = await db.conversations.get(row.id);
            if (existing && BigInt(existing.syncSeq) > BigInt(row.syncSeq))
                continue;
            await db.conversations.put(row);
        }
    });
}

export async function upsertMessages(
    db: LemmaDb,
    rows: MessageRow[],
): Promise<void> {
    await db.transaction("rw", db.messages, async () => {
        for (const row of rows) {
            const existing = await db.messages.get(row.id);
            if (existing && BigInt(existing.syncSeq) > BigInt(row.syncSeq))
                continue;
            await db.messages.put(row);
        }
    });
}

/** 归档全量刷新：删掉不在新列表里的归档行，返回被删的会话 id（调用方级联清消息） */
export async function replaceArchived(
    db: LemmaDb,
    rows: ConversationRow[],
): Promise<string[]> {
    return db.transaction("rw", db.conversations, async () => {
        const keep = new Set(rows.map((r) => r.id));
        const stale = await db.conversations
            .filter((r) => r.status === 2 && !keep.has(r.id))
            .toArray();
        await db.conversations.bulkDelete(stale.map((r) => r.id));
        for (const row of rows) {
            const existing = await db.conversations.get(row.id);
            if (existing && BigInt(existing.syncSeq) > BigInt(row.syncSeq))
                continue;
            await db.conversations.put(row);
        }
        return stale.map((r) => r.id);
    });
}

/** 活跃会话对账：删掉服务端活跃名单之外的 active 行，返回被删 id（调用方级联清消息） */
export async function pruneActiveExcept(
    db: LemmaDb,
    keepIds: Set<string>,
): Promise<string[]> {
    return db.transaction("rw", db.conversations, async () => {
        const stale = await db.conversations
            .filter((r) => r.status !== 2 && !keepIds.has(r.id))
            .toArray();
        await db.conversations.bulkDelete(stale.map((r) => r.id));
        return stale.map((r) => r.id);
    });
}

/** 彻底删除：会话连同消息一起清 */
export async function deleteConversationCascade(
    db: LemmaDb,
    id: string,
): Promise<void> {
    await db.transaction("rw", db.conversations, db.messages, async () => {
        await db.conversations.delete(id);
        await db.messages.where("conversationId").equals(id).delete();
    });
}
