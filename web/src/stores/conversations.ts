import { create as createMessage } from "@bufbuild/protobuf";
import { type Timestamp, TimestampSchema } from "@bufbuild/protobuf/wkt";
import { create } from "zustand";

import {
    type Conversation,
    ConversationSchema,
    type ConversationStatus,
} from "@/gen/lemma/v1/conversation_pb";
import { conversationClient } from "@/lib/clients";
import {
    type ConversationRow,
    getDb,
    listArchived,
    listConversations,
} from "@/lib/db";
import { pullAll } from "@/lib/sync";

interface ConversationsState {
    list: Conversation[];
    archived: Conversation[];
    loaded: boolean;
    hydrateFromCache: () => Promise<void>;
    refresh: () => Promise<void>;
    refreshArchived: () => Promise<void>;
    create: () => Promise<string>; // 返回新会话 id
    rename: (id: string, title: string) => Promise<void>;
    archive: (id: string) => Promise<void>;
    restore: (id: string) => Promise<void>;
    deleteArchived: (id: string) => Promise<void>;
}

function toTimestamp(ms: number): Timestamp {
    return createMessage(TimestampSchema, {
        seconds: BigInt(Math.floor(ms / 1000)),
        nanos: (ms % 1000) * 1_000_000,
    });
}

function rowToConversation(r: ConversationRow): Conversation {
    return createMessage(ConversationSchema, {
        id: r.id,
        title: r.title,
        status: r.status as ConversationStatus,
        archivedAt:
            r.archivedAtMs === null ? undefined : toTimestamp(r.archivedAtMs),
        messageCount: r.messageCount,
        createdAt: toTimestamp(r.createdAtMs),
        updatedAt: toTimestamp(r.updatedAtMs),
    });
}

export const useConversationsStore = create<ConversationsState>()(
    (set, get) => ({
        list: [],
        archived: [],
        loaded: false,

        // 从 IndexedDB 缓存直接铺数据（离线也能秒开）
        hydrateFromCache: async () => {
            const db = getDb();
            if (!db) return;
            const [list, archived] = await Promise.all([
                listConversations(db),
                listArchived(db),
            ]);
            set({
                list: list.map(rowToConversation),
                archived: archived.map(rowToConversation),
                loaded: true,
            });
        },

        // 缓存优先秒开；同步引擎补拉完成后会再 hydrate 收敛
        refresh: async () => {
            await get().hydrateFromCache();
            void pullAll().catch(() => {});
        },

        refreshArchived: async () => {
            await get().hydrateFromCache();
            void pullAll().catch(() => {});
        },

        // 变更成功后立即补拉一次，让缓存即时收敛（watch hint 3s 内也会兜底）
        create: async () => {
            const res = await conversationClient.createConversation({});
            if (!res.conversation)
                throw new Error("no conversation in response");
            set((s) => ({ list: [res.conversation!, ...s.list] }));
            void pullAll().catch(() => {});
            return res.conversation.id;
        },

        rename: async (id, title) => {
            const res = await conversationClient.renameConversation({
                id,
                title,
            });
            if (!res.conversation) return;
            set((s) => ({
                list: s.list.map((c) => (c.id === id ? res.conversation! : c)),
            }));
            void pullAll().catch(() => {});
        },

        archive: async (id) => {
            await conversationClient.archiveConversation({ id });
            const item = get().list.find((c) => c.id === id);
            set((s) => ({
                list: s.list.filter((c) => c.id !== id),
                archived: item ? [item, ...s.archived] : s.archived,
            }));
            void pullAll().catch(() => {});
        },

        restore: async (id) => {
            const res = await conversationClient.restoreConversation({ id });
            set((s) => ({
                archived: s.archived.filter((c) => c.id !== id),
                list: res.conversation ? [res.conversation, ...s.list] : s.list,
            }));
            void pullAll().catch(() => {});
        },

        // 彻底删除，不可恢复
        deleteArchived: async (id) => {
            await conversationClient.deleteArchived({ id });
            set((s) => ({
                archived: s.archived.filter((c) => c.id !== id),
            }));
            void pullAll().catch(() => {});
        },
    }),
);
