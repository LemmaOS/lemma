import { timestampDate } from "@bufbuild/protobuf/wkt";
import Dexie, { type EntityTable } from "dexie";

import type { Conversation, Message } from "@/gen/lemma/v1/conversation_pb";

/**
 * Cached conversation: the proto entity flattened plus sync metadata.
 * Timestamps are epoch millis and syncSeq is a string, since neither
 * IndexedDB keys nor JSON can hold a bigint.
 */
export interface ConversationRow {
    id: string;
    title: string;
    // ConversationStatus enum value; 2 is archived.
    status: number;
    archivedAtMs: number | null;
    messageCount: number;
    createdAtMs: number;
    updatedAtMs: number;
    syncSeq: string;
}

/** Cached message; same flattening rules as ConversationRow. */
export interface MessageRow {
    id: string;
    conversationId: string;
    role: string;
    content: string;
    providerId: string;
    model: string;
    // MessageStatus enum value, kept as a number.
    status: number;
    createdAtMs: number;
    // Per-conversation monotonic sequence number (insertion order).
    seq: number;
    syncSeq: string;
}

export interface MetaRow {
    key: string;
    value: string;
}

/** Per-user cache database; separate databases keep accounts isolated. */
export class LemmaDb extends Dexie {
    conversations!: EntityTable<ConversationRow, "id">;
    messages!: EntityTable<MessageRow, "id">;
    meta!: EntityTable<MetaRow, "key">;

    constructor(userId: string) {
        super(`lemma-${userId}`);
        // Dexie version blocks are append-only: never edit a shipped one,
        // add the next number instead.
        this.version(1).stores({
            conversations: "id, updatedAtMs",
            messages: "id, [conversationId+createdAtMs]",
            meta: "key",
        });
        // Version 2 re-indexes messages by seq instead of createdAtMs;
        // rows cached under the old ordering are dropped and re-pulled.
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

// The open database is a module-level singleton; openDb swaps it when the
// account changes.
let current: LemmaDb | null = null;

export function openDb(userId: string): LemmaDb {
    if (current?.name === `lemma-${userId}`) return current;
    current?.close();
    current = new LemmaDb(userId);
    return current;
}

/** Returns the open database, or null when signed out. */
export function getDb(): LemmaDb | null {
    return current;
}

export function closeDb(): void {
    current?.close();
    current = null;
}

const ms = (ts: Conversation["updatedAt"]): number =>
    ts ? timestampDate(ts).getTime() : 0;

/**
 * Flattens a proto Conversation into a cache row, stamped with the sync
 * sequence the entity arrived with.
 */
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

/** Flattens a proto Message like conversationToRow. */
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

/** The highest sync_seq applied to this cache so far. */
export async function getCursor(db: LemmaDb): Promise<bigint> {
    const row = await db.meta.get(CURSOR_KEY);
    return row ? BigInt(row.value) : 0n;
}

export async function setCursor(db: LemmaDb, seq: bigint): Promise<void> {
    await db.meta.put({ key: CURSOR_KEY, value: seq.toString() });
}

/** Active conversations, most recently updated first. */
export async function listConversations(
    db: LemmaDb,
): Promise<ConversationRow[]> {
    const rows = await db.conversations.toArray();
    // Status 2 is ConversationStatus.ARCHIVED.
    return rows
        .filter((r) => r.status !== 2)
        .sort((a, b) => b.updatedAtMs - a.updatedAtMs);
}

/** Archived conversations, most recently archived first. */
export async function listArchived(db: LemmaDb): Promise<ConversationRow[]> {
    const rows = await db.conversations.toArray();
    return rows
        .filter((r) => r.status === 2)
        .sort((a, b) => (b.archivedAtMs ?? 0) - (a.archivedAtMs ?? 0));
}

/** All cached messages of a conversation, in seq order. */
export async function listMessages(
    db: LemmaDb,
    conversationId: string,
): Promise<MessageRow[]> {
    // The compound-index range covers exactly this conversation's seq span.
    return db.messages
        .where("[conversationId+seq]")
        .between([conversationId, 0], [conversationId, Infinity])
        .toArray();
}

/**
 * Last-write-wins upsert: an incoming row is skipped when the cached row
 * has a higher syncSeq, so a stale pull page cannot roll back newer data.
 * Equal syncSeq still overwrites, letting a re-pulled page heal a row that
 * a previous write missed.
 */
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

/** Last-write-wins upsert, same rule as upsertConversations. */
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

/**
 * Full refresh of the archived list: cached archived rows absent from the
 * new list are deleted, and their ids are returned so the caller can
 * cascade-delete their messages.
 */
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

/**
 * Deletes cached active conversations absent from the server's roster and
 * returns their ids, so the caller can cascade-delete their messages.
 */
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

export async function deleteConversationCascade(
    db: LemmaDb,
    id: string,
): Promise<void> {
    await db.transaction("rw", db.conversations, db.messages, async () => {
        await db.conversations.delete(id);
        await db.messages.where("conversationId").equals(id).delete();
    });
}

/**
 * Drops the cached messages of the given conversations. Messages of
 * archived conversations live only in the server-side archive, so a
 * restored conversation re-pulls its history.
 */
export async function deleteMessagesOf(
    db: LemmaDb,
    conversationIds: string[],
): Promise<void> {
    if (conversationIds.length === 0) return;
    await db.messages.where("conversationId").anyOf(conversationIds).delete();
}
