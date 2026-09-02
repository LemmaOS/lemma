import { create } from "zustand";

import i18n from "@/i18n";
import type { ChatEvent } from "@/gen/lemma/v1/chat_pb";
import { type Message, MessageStatus } from "@/gen/lemma/v1/conversation_pb";
import { chatClient, conversationClient } from "@/lib/clients";
import { getDb, listMessages, type MessageRow } from "@/lib/db";
import { errorText } from "@/lib/errors";
import { pullAll } from "@/lib/sync";

export interface ChatItem {
    id: string;
    role: "user" | "assistant";
    content: string;
    // UI-facing vocabulary mapped from MessageStatus.
    status: "streaming" | "done" | "aborted" | "error";
    providerId: string;
    model: string;
    error?: string;
}

interface ChatState {
    conversationId: string | null;
    items: ChatItem[];
    streaming: boolean;
    hasMore: boolean;
    open: (conversationId: string) => Promise<void>;
    syncFromCache: () => Promise<void>;
    loadMore: () => Promise<void>;
    send: (providerId: string, model: string, content: string) => Promise<void>;
    abort: () => Promise<void>;
}

// In-flight stream state lives outside the store: it is not render data,
// and only one stream runs at a time.
let controller: AbortController | null = null;
let activeMessageId: string | null = null;
let userAborted = false;

// Counts code points, not UTF-16 code units: the server measures the
// resume offset in chars (Rust chars().skip()), so a string containing
// emoji would misalign the replay if measured by .length.
const charLen = (s: string) => Array.from(s).length;

const PAGE_SIZE = 50;
const MAX_RESUME = 3;

function statusFromProto(s: MessageStatus): ChatItem["status"] {
    switch (s) {
        case MessageStatus.STREAMING:
            return "streaming";
        case MessageStatus.ABORTED:
            return "aborted";
        case MessageStatus.ERROR:
            return "error";
        default:
            return "done";
    }
}

function protoToItem(m: Message): ChatItem {
    return {
        id: m.id,
        role: m.role === "user" ? "user" : "assistant",
        content: m.content,
        status: statusFromProto(m.status),
        providerId: m.providerId,
        model: m.model,
    };
}

function rowToItem(m: MessageRow): ChatItem {
    return {
        id: m.id,
        role: m.role === "user" ? "user" : "assistant",
        content: m.content,
        status: statusFromProto(m.status as MessageStatus),
        providerId: m.providerId,
        model: m.model,
    };
}

export const useChat = create<ChatState>()((set, get) => ({
    conversationId: null,
    items: [],
    streaming: false,
    hasMore: false,

    open: async (conversationId) => {
        // Prefer the local cache; without an open database, fall back to a
        // server page (which comes newest-first and is reversed here).
        const db = getDb();
        if (db) {
            const rows = await listMessages(db, conversationId);
            set({
                conversationId,
                items: rows.map(rowToItem),
                hasMore: false,
            });
            return;
        }
        const res = await conversationClient.listMessages({
            conversationId,
            limit: PAGE_SIZE,
        });
        set({
            conversationId,
            items: res.messages.map(protoToItem).reverse(),
            hasMore: res.hasMore,
        });
    },

    syncFromCache: async () => {
        const { conversationId, streaming } = get();
        // Never clobber an in-flight stream's optimistic items.
        const db = getDb();
        if (!db || !conversationId || streaming) return;
        const rows = await listMessages(db, conversationId);
        set({ items: rows.map(rowToItem), hasMore: false });
    },

    loadMore: async () => {
        const { conversationId, items, hasMore } = get();
        if (!conversationId || !hasMore || items.length === 0) return;
        const res = await conversationClient.listMessages({
            conversationId,
            beforeId: items[0].id,
            limit: PAGE_SIZE,
        });
        set((s) => ({
            items: [...res.messages.map(protoToItem).reverse(), ...s.items],
            hasMore: res.hasMore,
        }));
    },

    send: async (providerId, model, content) => {
        const { conversationId, streaming } = get();
        if (!conversationId || streaming) return;

        // clientMsgId doubles as the idempotency key: a resend with the
        // same id returns the already-created messages instead of
        // duplicating them. The assistant placeholder derives its temporary
        // id from it.
        const clientMsgId = crypto.randomUUID();
        const aiTempId = `${clientMsgId}:ai`;
        controller = new AbortController();
        const { signal } = controller;
        activeMessageId = null;
        userAborted = false;

        // Optimistically render both messages under temporary ids; the
        // post-send pull replaces them with the persisted rows.
        set((s) => ({
            streaming: true,
            items: [
                ...s.items,
                {
                    id: clientMsgId,
                    role: "user",
                    content,
                    status: "done",
                    providerId: "",
                    model: "",
                },
                {
                    id: aiTempId,
                    role: "assistant",
                    content: "",
                    status: "streaming",
                    providerId,
                    model,
                },
            ],
        }));

        const updateAi = (patch: Partial<ChatItem>) =>
            set((s) => ({
                items: s.items.map((it) =>
                    it.id === aiTempId ? { ...it, ...patch } : it,
                ),
            }));
        const appendAi = (chunk: string) =>
            set((s) => ({
                items: s.items.map((it) =>
                    it.id === aiTempId
                        ? { ...it, content: it.content + chunk }
                        : it,
                ),
            }));

        const applyEvent = (event?: ChatEvent) => {
            const kind = event?.kind;
            if (!kind) return;
            switch (kind.case) {
                case "started":
                    activeMessageId = kind.value.messageId;
                    break;
                case "delta":
                    appendAi(kind.value.content);
                    break;
                case "done":
                    updateAi({ status: "done" });
                    break;
                case "aborted":
                    updateAi({ status: "aborted" });
                    break;
                case "error":
                    updateAi({ status: "error", error: kind.value.message });
                    break;
            }
        };

        try {
            let resumes = 0;
            // Resume replays an in-flight message from the current content
            // length; it needs the messageId from "started", so a send that
            // failed before its first event is never retried.
            for (;;) {
                try {
                    if (!activeMessageId) {
                        const stream = chatClient.sendMessage(
                            {
                                conversationId,
                                content,
                                providerId,
                                model,
                                clientMsgId,
                            },
                            { signal },
                        );
                        for await (const res of stream) applyEvent(res.event);
                    } else {
                        const current =
                            get().items.find((it) => it.id === aiTempId)
                                ?.content ?? "";
                        const stream = chatClient.resumeStream(
                            {
                                messageId: activeMessageId,
                                offset: BigInt(charLen(current)),
                            },
                            { signal },
                        );
                        for await (const res of stream) applyEvent(res.event);
                    }
                    // The stream ended normally; leave the resume loop.
                    break;
                } catch (e) {
                    if (userAborted || signal.aborted) throw e;
                    resumes += 1;
                    if (!activeMessageId || resumes > MAX_RESUME) throw e;
                    await new Promise((r) => setTimeout(r, 500 * resumes));
                }
            }
        } catch (e) {
            if (userAborted || signal.aborted) {
                updateAi({ status: "aborted" });
            } else {
                updateAi({ status: "error", error: errorText(e, i18n.t) });
            }
        } finally {
            controller = null;
            activeMessageId = null;
            set({ streaming: false });
            // The server has persisted the final message state; pull so the
            // cache converges right away instead of waiting for a hint.
            void pullAll().catch(() => {});
        }
    },

    abort: async () => {
        userAborted = true;
        // Tell the server to finalize the message as aborted before cutting
        // the local stream; the database is the source of truth.
        const id = activeMessageId;
        if (id) {
            try {
                await chatClient.abortMessage({ messageId: id });
            } catch {
                // Persisting the abort is best-effort; the local stream is
                // cut either way.
            }
        }
        controller?.abort();
    },
}));
