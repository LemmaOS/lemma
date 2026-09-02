import "fake-indexeddb/auto";
import { beforeEach, expect, it, vi } from "vitest";

vi.mock("@/lib/clients", () => ({
    chatClient: {
        sendMessage: vi.fn(),
        resumeStream: vi.fn(),
        abortMessage: vi.fn(),
    },
    conversationClient: {
        listMessages: vi.fn(),
    },
}));

import type {
    ChatEvent,
    ResumeStreamResponse,
    SendMessageResponse,
} from "@/gen/lemma/v1/chat_pb";
import { MessageStatus } from "@/gen/lemma/v1/conversation_pb";
import { chatClient, conversationClient } from "@/lib/clients";
import { closeDb, openDb, upsertMessages } from "@/lib/db";
import { useChat } from "./chat";

const sendMessage = vi.mocked(chatClient.sendMessage);
const resumeStream = vi.mocked(chatClient.resumeStream);
const abortMessage = vi.mocked(chatClient.abortMessage);
const listMessages = vi.mocked(conversationClient.listMessages);

function ev<T>(kind: ChatEvent["kind"]): T {
    return { event: { kind } } as unknown as T;
}

const started = (messageId: string): SendMessageResponse =>
    ev({ case: "started", value: { messageId, clientMsgId: "" } as never });
const delta = (content: string): SendMessageResponse =>
    ev({ case: "delta", value: { content } as never });
const done: SendMessageResponse = ev({ case: "done", value: {} as never });

// An async iterable that rejects on the first read, like a stream whose
// connection drops before any event arrives.
function throwStream(err: Error): AsyncIterable<never> {
    return {
        [Symbol.asyncIterator]() {
            return { next: () => Promise.reject(err) };
        },
    };
}

beforeEach(() => {
    vi.clearAllMocks();
    closeDb();
    useChat.setState({
        conversationId: "conv-1",
        items: [],
        streaming: false,
        hasMore: false,
    });
});

it("send 走通 started → delta → done", async () => {
    sendMessage.mockImplementation(async function* () {
        yield started("m1");
        yield delta("你");
        yield delta("好");
        yield done;
    });

    await useChat.getState().send("p1", "gpt-x", "你好");

    const { items, streaming } = useChat.getState();
    expect(streaming).toBe(false);
    expect(items).toHaveLength(2);
    expect(items[0]).toMatchObject({
        role: "user",
        content: "你好",
        status: "done",
    });
    expect(items[1]).toMatchObject({
        role: "assistant",
        content: "你好",
        status: "done",
    });
});

it("断线后按已收字符数 offset 续传", async () => {
    sendMessage.mockImplementationOnce(async function* () {
        yield started("m1");
        yield delta("你");
        throw new Error("network down");
    });
    resumeStream.mockImplementationOnce(async function* () {
        yield delta("好") as unknown as ResumeStreamResponse;
        yield done as unknown as ResumeStreamResponse;
    });

    await useChat.getState().send("p1", "gpt-x", "你好");

    expect(resumeStream).toHaveBeenCalledWith(
        { messageId: "m1", offset: 1n },
        expect.anything(),
    );
    const { items } = useChat.getState();
    expect(items[1]).toMatchObject({ content: "你好", status: "done" });
});

it("abort 通知服务端并标记 aborted", async () => {
    sendMessage.mockImplementation(async function* (_req, opts) {
        yield started("m1");
        yield delta("半");
        await new Promise((_, reject) => {
            opts?.signal?.addEventListener("abort", () =>
                reject(new Error("aborted")),
            );
        });
    });
    abortMessage.mockResolvedValue({} as never);

    const p = useChat.getState().send("p1", "gpt-x", "你好");
    await vi.waitFor(() => {
        expect(useChat.getState().items[1]?.content).toBe("半");
    });

    await useChat.getState().abort();
    await p;

    expect(abortMessage).toHaveBeenCalledWith({ messageId: "m1" });
    expect(useChat.getState().items[1].status).toBe("aborted");
    expect(useChat.getState().streaming).toBe(false);
});

it("open 加载历史并转正序", async () => {
    listMessages.mockResolvedValue({
        messages: [
            {
                id: "m2",
                role: "assistant",
                content: "答",
                status: MessageStatus.DONE,
                providerId: "p1",
                model: "gpt-x",
            },
            {
                id: "m1",
                role: "user",
                content: "问",
                status: MessageStatus.DONE,
            },
        ],
        hasMore: false,
    } as never);

    await useChat.getState().open("conv-1");

    const { items } = useChat.getState();
    expect(items.map((i) => i.id)).toEqual(["m1", "m2"]);
    expect(items[1]).toMatchObject({
        content: "答",
        status: "done",
        model: "gpt-x",
    });
});

it("open 优先读本地缓存", async () => {
    const db = openDb("chat-cache-test");
    await db.delete();
    await db.open();
    await upsertMessages(db, [
        {
            id: "m1",
            conversationId: "conv-1",
            role: "user",
            content: "hi",
            providerId: "",
            model: "",
            status: MessageStatus.DONE,
            createdAtMs: 1,
            seq: 1,
            syncSeq: "1",
        },
    ]);

    await useChat.getState().open("conv-1");

    const { items, hasMore } = useChat.getState();
    expect(items).toHaveLength(1);
    expect(items[0]).toMatchObject({ id: "m1", content: "hi", status: "done" });
    expect(hasMore).toBe(false);
    expect(listMessages).not.toHaveBeenCalled();
    closeDb();
});

it("open 映射 streaming/aborted/error 状态", async () => {
    const msg = (id: string, status: MessageStatus) => ({
        id,
        role: "assistant",
        content: id,
        status,
        providerId: "p1",
        model: "gpt-x",
    });
    listMessages.mockResolvedValue({
        // Server pages come newest-first; open reverses them.
        messages: [
            msg("m3", MessageStatus.ERROR),
            msg("m2", MessageStatus.ABORTED),
            msg("m1", MessageStatus.STREAMING),
        ],
        hasMore: false,
    } as never);

    await useChat.getState().open("conv-1");

    expect(useChat.getState().items.map((i) => i.status)).toEqual([
        "streaming",
        "aborted",
        "error",
    ]);
});

it("syncFromCache 非流式时从缓存刷新", async () => {
    const db = openDb("chat-cache-test");
    await db.delete();
    await db.open();
    await upsertMessages(db, [
        {
            id: "m1",
            conversationId: "conv-1",
            role: "user",
            content: "cached",
            providerId: "",
            model: "",
            status: MessageStatus.DONE,
            createdAtMs: 1,
            seq: 1,
            syncSeq: "1",
        },
    ]);
    useChat.setState({
        items: [
            {
                id: "stale",
                role: "user",
                content: "",
                status: "done",
                providerId: "",
                model: "",
            },
        ],
    });

    await useChat.getState().syncFromCache();

    expect(useChat.getState().items.map((i) => i.id)).toEqual(["m1"]);
    closeDb();
});

it("syncFromCache 流式中不动 optimistic 项", async () => {
    useChat.setState({
        streaming: true,
        items: [
            {
                id: "live",
                role: "assistant",
                content: "半",
                status: "streaming",
                providerId: "p1",
                model: "gpt-x",
            },
        ],
    });

    await useChat.getState().syncFromCache();

    expect(useChat.getState().items.map((i) => i.id)).toEqual(["live"]);
});

it("loadMore 把更早的一页拼到前面", async () => {
    useChat.setState({
        hasMore: true,
        items: [
            {
                id: "m2",
                role: "user",
                content: "二",
                status: "done",
                providerId: "",
                model: "",
            },
        ],
    });
    listMessages.mockResolvedValue({
        messages: [
            {
                id: "m1",
                role: "user",
                content: "一",
                status: MessageStatus.DONE,
                providerId: "",
                model: "",
            },
        ],
        hasMore: false,
    } as never);

    await useChat.getState().loadMore();

    const s = useChat.getState();
    expect(s.items.map((i) => i.id)).toEqual(["m1", "m2"]);
    expect(s.hasMore).toBe(false);
    expect(listMessages).toHaveBeenCalledWith({
        conversationId: "conv-1",
        beforeId: "m2",
        limit: 50,
    });
});

it("loadMore 无更多时直接返回", async () => {
    await useChat.getState().loadMore();

    expect(listMessages).not.toHaveBeenCalled();
});

it("send 在流式中或无会话时直接返回", async () => {
    useChat.setState({ streaming: true });
    await useChat.getState().send("p1", "gpt-x", "你好");
    useChat.setState({ streaming: false, conversationId: null });
    await useChat.getState().send("p1", "gpt-x", "你好");

    expect(sendMessage).not.toHaveBeenCalled();
    expect(useChat.getState().items).toHaveLength(0);
});

it("aborted 事件标记中止", async () => {
    sendMessage.mockImplementation(async function* () {
        yield started("m1");
        yield ev({ case: "aborted", value: {} as never });
    });

    await useChat.getState().send("p1", "gpt-x", "你好");

    expect(useChat.getState().items[1].status).toBe("aborted");
});

it("error 事件带出世态文案", async () => {
    sendMessage.mockImplementation(async function* () {
        yield started("m1");
        yield ev({
            case: "error",
            value: { message: "model exploded" } as never,
        });
    });

    await useChat.getState().send("p1", "gpt-x", "你好");

    const item = useChat.getState().items[1];
    expect(item.status).toBe("error");
    expect(item.error).toBe("model exploded");
});

it("无 kind 的事件被忽略", async () => {
    sendMessage.mockImplementation(async function* () {
        yield started("m1");
        yield { event: {} } as unknown as SendMessageResponse;
        yield done;
    });

    await useChat.getState().send("p1", "gpt-x", "你好");

    expect(useChat.getState().items[1].status).toBe("done");
});

it("首轮即失败不重试，标记 error", async () => {
    sendMessage.mockImplementation(
        () => throwStream(new Error("boom")) as never,
    );

    await useChat.getState().send("p1", "gpt-x", "你好");

    expect(resumeStream).not.toHaveBeenCalled();
    const item = useChat.getState().items[1];
    expect(item.status).toBe("error");
    expect(item.error).toContain("boom");
});

it("续传三次仍失败则放弃", async () => {
    vi.useFakeTimers();
    try {
        sendMessage.mockImplementation(async function* () {
            yield started("m1");
            yield delta("半");
            throw new Error("down");
        });
        resumeStream.mockImplementation(
            () => throwStream(new Error("down")) as never,
        );

        const p = useChat.getState().send("p1", "gpt-x", "你好");
        await vi.runAllTimersAsync();
        await p;
    } finally {
        vi.useRealTimers();
    }

    expect(resumeStream).toHaveBeenCalledTimes(3);
    expect(useChat.getState().items[1].status).toBe("error");
});
