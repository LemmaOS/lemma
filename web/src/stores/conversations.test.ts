import "fake-indexeddb/auto";

import { create } from "@bufbuild/protobuf";
import { TimestampSchema } from "@bufbuild/protobuf/wkt";
import { beforeEach, expect, it, vi } from "vitest";

import {
    type Conversation,
    ConversationSchema,
    type ConversationStatus,
} from "@/gen/lemma/v1/conversation_pb";
import { conversationClient, syncClient } from "@/lib/clients";
import { closeDb, conversationToRow, openDb } from "@/lib/db";
import { useConversationsStore } from "./conversations";

vi.mock("@/lib/clients", () => ({
    conversationClient: {
        listConversations: vi.fn(),
        listArchived: vi.fn(),
        createConversation: vi.fn(),
        renameConversation: vi.fn(),
        archiveConversation: vi.fn(),
        restoreConversation: vi.fn(),
        deleteArchived: vi.fn(),
    },
    syncClient: { pull: vi.fn(), watch: vi.fn() },
}));

function convProto(id: string, status: number): Conversation {
    return create(ConversationSchema, {
        id,
        title: `t-${id}`,
        status: status as ConversationStatus,
        createdAt: create(TimestampSchema, { seconds: 1000n }),
        updatedAt: create(TimestampSchema, { seconds: 1000n }),
    });
}

beforeEach(async () => {
    vi.clearAllMocks();
    closeDb();
    const db = openDb("conv-store-test");
    await db.delete();
    await db.open();
    useConversationsStore.setState({ list: [], archived: [], loaded: false });
});

it("离线时 refresh 仍能从缓存加载列表", async () => {
    const db = openDb("conv-store-test");
    await db.conversations.bulkPut([
        conversationToRow(convProto("c1", 1), 5n),
        conversationToRow(convProto("c2", 2), 6n),
    ]);
    vi.mocked(syncClient.pull).mockRejectedValue(new Error("offline"));

    await useConversationsStore.getState().refresh();

    const s = useConversationsStore.getState();
    expect(s.loaded).toBe(true);
    expect(s.list.map((c) => c.id)).toEqual(["c1"]);
    expect(s.archived.map((c) => c.id)).toEqual(["c2"]);
});

it("归档后从活跃列表消失并触发补拉", async () => {
    vi.mocked(conversationClient.archiveConversation).mockResolvedValue(
        {} as never,
    );
    vi.mocked(syncClient.pull).mockResolvedValue({
        conversations: [],
        messages: [],
        archived: [],
        active: [],
        nextAfter: 0n,
        hasMore: false,
    } as never);
    useConversationsStore.setState({
        list: [convProto("c1", 1)],
        loaded: true,
    });

    await useConversationsStore.getState().archive("c1");

    const s = useConversationsStore.getState();
    expect(s.list).toHaveLength(0);
    expect(s.archived.map((c) => c.id)).toEqual(["c1"]);
    expect(syncClient.pull).toHaveBeenCalled();
});
