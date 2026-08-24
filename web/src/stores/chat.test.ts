import "fake-indexeddb/auto";
import { beforeEach, expect, it, vi } from "vitest";

// mock 掉整个客户端层：测试目标是状态机，不是网络
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

// 造事件：状态机只读 kind.case / kind.value，直接字面量强转
function ev<T>(kind: ChatEvent["kind"]): T {
    return { event: { kind } } as unknown as T;
}

const started = (messageId: string): SendMessageResponse =>
    ev({ case: "started", value: { messageId, clientMsgId: "" } as never });
const delta = (content: string): SendMessageResponse =>
    ev({ case: "delta", value: { content } as never });
const done: SendMessageResponse = ev({ case: "done", value: {} as never });

beforeEach(() => {
    vi.clearAllMocks();
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
    // 第一次：started + 一个 delta 后网络断开
    sendMessage.mockImplementationOnce(async function* () {
        yield started("m1");
        yield delta("你");
        throw new Error("network down");
    });
    // 续传：补差额 + done
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
        // 挂起直到本地掐流
        await new Promise((_, reject) => {
            opts?.signal?.addEventListener("abort", () =>
                reject(new Error("aborted")),
            );
        });
    });
    abortMessage.mockResolvedValue({} as never);

    const p = useChat.getState().send("p1", "gpt-x", "你好");
    // 等流式内容出现
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
    // 服务端返回倒序（最新在前）
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
