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
import { pullAll } from "@/lib/sync";
import { useConversationsStore } from "./conversations";

const pullAllMock = vi.mocked(pullAll);

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

// The store kicks pullAll fire-and-forget after every action; mocking the
// sync module boundary keeps those floating pulls from bleeding into the
// next test.
vi.mock("@/lib/sync", () => ({ pullAll: vi.fn() }));

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
    pullAllMock.mockResolvedValue(undefined);
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
    pullAllMock.mockRejectedValue(new Error("offline"));

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
    useConversationsStore.setState({
        list: [convProto("c1", 1)],
        loaded: true,
    });

    await useConversationsStore.getState().archive("c1");

    const s = useConversationsStore.getState();
    expect(s.list).toHaveLength(0);
    expect(s.archived.map((c) => c.id)).toEqual(["c1"]);
    expect(pullAllMock).toHaveBeenCalled();
});

it("无缓存库时 refresh 直接返回", async () => {
    closeDb();

    await useConversationsStore.getState().refresh();

    expect(useConversationsStore.getState().loaded).toBe(false);
    expect(syncClient.pull).not.toHaveBeenCalled();
});

it("refreshArchived 从缓存加载归档列表", async () => {
    const db = openDb("conv-store-test");
    await db.conversations.bulkPut([conversationToRow(convProto("a1", 2), 5n)]);
    pullAllMock.mockRejectedValue(new Error("offline"));

    await useConversationsStore.getState().refreshArchived();

    const s = useConversationsStore.getState();
    expect(s.loaded).toBe(true);
    expect(s.archived.map((c) => c.id)).toEqual(["a1"]);
});

it("create 把新会话插到列表头并返回 id", async () => {
    vi.mocked(conversationClient.createConversation).mockResolvedValue({
        conversation: convProto("c9", 1),
    } as never);
    useConversationsStore.setState({ list: [convProto("c1", 1)] });

    const id = await useConversationsStore.getState().create();

    expect(id).toBe("c9");
    expect(useConversationsStore.getState().list.map((c) => c.id)).toEqual([
        "c9",
        "c1",
    ]);
});

it("create 响应缺 conversation 抛错", async () => {
    vi.mocked(conversationClient.createConversation).mockResolvedValue(
        {} as never,
    );

    await expect(useConversationsStore.getState().create()).rejects.toThrow(
        "no conversation in response",
    );
});

it("rename 更新列表中的标题", async () => {
    const renamed = convProto("c1", 1);
    renamed.title = "新标题";
    vi.mocked(conversationClient.renameConversation).mockResolvedValue({
        conversation: renamed,
    } as never);
    useConversationsStore.setState({ list: [convProto("c1", 1)] });

    await useConversationsStore.getState().rename("c1", "新标题");

    expect(useConversationsStore.getState().list[0].title).toBe("新标题");
});

it("rename 响应缺 conversation 时不动列表", async () => {
    vi.mocked(conversationClient.renameConversation).mockResolvedValue(
        {} as never,
    );
    useConversationsStore.setState({ list: [convProto("c1", 1)] });

    await useConversationsStore.getState().rename("c1", "新标题");

    expect(useConversationsStore.getState().list[0].title).toBe("t-c1");
});

it("restore 把会话移回活跃列表", async () => {
    vi.mocked(conversationClient.restoreConversation).mockResolvedValue({
        conversation: convProto("c1", 1),
    } as never);
    useConversationsStore.setState({
        archived: [convProto("c1", 2)],
        list: [],
    });

    await useConversationsStore.getState().restore("c1");

    const s = useConversationsStore.getState();
    expect(s.archived).toHaveLength(0);
    expect(s.list.map((c) => c.id)).toEqual(["c1"]);
});

it("deleteArchived 只移除目标归档项", async () => {
    vi.mocked(conversationClient.deleteArchived).mockResolvedValue({} as never);
    useConversationsStore.setState({
        archived: [convProto("c1", 2), convProto("c2", 2)],
    });

    await useConversationsStore.getState().deleteArchived("c1");

    expect(useConversationsStore.getState().archived.map((c) => c.id)).toEqual([
        "c2",
    ]);
});
