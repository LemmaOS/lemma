import { create } from "zustand";

import type { Conversation } from "@/gen/lemma/v1/conversation_pb";
import { conversationClient } from "@/lib/clients";

interface ConversationsState {
    list: Conversation[];
    archived: Conversation[];
    loaded: boolean;
    refresh: () => Promise<void>;
    refreshArchived: () => Promise<void>;
    create: () => Promise<string>; // 返回新会话 id
    rename: (id: string, title: string) => Promise<void>;
    archive: (id: string) => Promise<void>;
    restore: (id: string) => Promise<void>;
    deleteArchived: (id: string) => Promise<void>;
}

export const useConversationsStore = create<ConversationsState>()(
    (set, get) => ({
        list: [],
        archived: [],
        loaded: false,

        refresh: async () => {
            const res = await conversationClient.listConversations({});
            set({ list: res.conversations, loaded: true });
        },

        refreshArchived: async () => {
            const res = await conversationClient.listArchived({});
            set({ archived: res.conversations });
        },

        create: async () => {
            const res = await conversationClient.createConversation({});
            if (!res.conversation)
                throw new Error("no conversation in response");
            set((s) => ({ list: [res.conversation!, ...s.list] }));
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
        },

        archive: async (id) => {
            await conversationClient.archiveConversation({ id });
            // 归档后从活跃列表消失，元数据进归档列表
            const item = get().list.find((c) => c.id === id);
            set((s) => ({
                list: s.list.filter((c) => c.id !== id),
                archived: item ? [item, ...s.archived] : s.archived,
            }));
        },

        restore: async (id) => {
            const res = await conversationClient.restoreConversation({ id });
            set((s) => ({
                archived: s.archived.filter((c) => c.id !== id),
                list: res.conversation ? [res.conversation, ...s.list] : s.list,
            }));
        },

        // 彻底删除，不可恢复
        deleteArchived: async (id) => {
            await conversationClient.deleteArchived({ id });
            set((s) => ({ archived: s.archived.filter((c) => c.id !== id) }));
        },
    }),
);
