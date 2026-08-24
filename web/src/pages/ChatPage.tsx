import { LogOut, Plus, Settings } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { ChatComposer, type ModelOption } from "@/components/chat/ChatComposer";
import { EmptyState } from "@/components/chat/EmptyState";
import { MessageItem } from "@/components/chat/MessageItem";
import { SessionList } from "@/components/chat/SessionList";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useConversations } from "@/hooks/useConversations";
import { useProviders } from "@/hooks/useProviders";
import { useAuth } from "@/stores/auth";
import { useChat } from "@/stores/chat";

const MODEL_KEY = "lemma.model";

export default function ChatPage() {
    const { t } = useTranslation();
    const user = useAuth((s) => s.user);
    const logout = useAuth((s) => s.logout);
    const conversations = useConversations();
    const providers = useProviders();
    const chat = useChat();

    const [activeId, setActiveId] = useState<string | null>(null);
    const [draft, setDraft] = useState("");
    const [editingId, setEditingId] = useState<string | null>(null);
    const [modelKey, setModelKey] = useState<string | null>(
        () => localStorage.getItem(MODEL_KEY) || null,
    );

    const scrollRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLTextAreaElement>(null);
    // 向前翻页时记录旧 scrollHeight，渲染后用它恢复视口位置（而不是滚到底）
    const prependHeightRef = useRef<number | null>(null);

    // 归档组的数据只在侧栏展开时需要，进入页面先拉一次
    useEffect(() => {
        void conversations.refreshArchived();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    // 未显式选中时默认最新会话（渲染期派生，不用 effect 写回 state）
    const effectiveActiveId =
        activeId ??
        (conversations.loaded ? (conversations.list[0]?.id ?? null) : null);

    useEffect(() => {
        if (effectiveActiveId) void chat.open(effectiveActiveId);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [effectiveActiveId]);

    // 消息变化：翻页 prepend 恢复视口；其余情况（新消息/流式）滚到底
    useEffect(() => {
        const el = scrollRef.current;
        if (!el) return;
        if (prependHeightRef.current !== null) {
            el.scrollTop = el.scrollHeight - prependHeightRef.current;
            prependHeightRef.current = null;
            return;
        }
        el.scrollTop = el.scrollHeight;
    }, [chat.items, effectiveActiveId]);

    const modelOptions = useMemo<ModelOption[]>(
        () =>
            providers.list
                .filter((p) => p.enabled)
                .flatMap((p) =>
                    p.models.map((m) => ({
                        key: `${p.id}:${m}`,
                        providerId: p.id,
                        model: m,
                        label: `${p.name} · ${m}`,
                    })),
                ),
        [providers.list],
    );
    const selectedModel =
        modelOptions.find((o) => o.key === modelKey) ?? modelOptions[0] ?? null;

    const providerName = (id: string) =>
        providers.list.find((p) => p.id === id)?.name;

    const handleSelectModel = (key: string) => {
        setModelKey(key);
        localStorage.setItem(MODEL_KEY, key);
    };

    const handleScroll = () => {
        const el = scrollRef.current;
        if (!el || el.scrollTop > 0 || !chat.hasMore || !effectiveActiveId)
            return;
        prependHeightRef.current = el.scrollHeight;
        void chat.loadMore();
    };

    const handleSend = async () => {
        const text = draft.trim();
        if (!text || chat.streaming || !selectedModel) return;
        let cid = effectiveActiveId;
        if (!cid) {
            // 空状态首发：先建会话再发
            cid = await conversations.create();
            setActiveId(cid);
            await chat.open(cid);
        }
        setDraft("");
        void chat.send(selectedModel.providerId, selectedModel.model, text);
    };

    const handleNewChat = async () => {
        const id = await conversations.create();
        setActiveId(id);
        inputRef.current?.focus();
    };

    const handleArchive = async (id: string) => {
        await conversations.archive(id);
        if (id === activeId) setActiveId(null);
    };

    const username = user?.username ?? "";

    return (
        <div className="flex h-dvh bg-background text-foreground">
            {/* Sidebar */}
            <aside className="flex w-[260px] shrink-0 flex-col border-r border-sidebar-border bg-sidebar">
                <div className="flex items-center justify-between px-4 pt-4 pb-2">
                    <span className="truncate text-sm font-semibold">
                        {t("auth.productName")}
                    </span>
                    <ThemeToggle />
                </div>
                <div className="px-3 pb-2">
                    <Button className="w-full" onClick={handleNewChat}>
                        <Plus className="size-4" />
                        {t("chat.newChat")}
                    </Button>
                </div>
                <SessionList
                    conversations={conversations.list}
                    archived={conversations.archived}
                    activeId={effectiveActiveId}
                    editingId={editingId}
                    onSelect={setActiveId}
                    onStartRename={setEditingId}
                    onCommitRename={(id, title) => {
                        void conversations.rename(id, title);
                        setEditingId(null);
                    }}
                    onCancelRename={() => setEditingId(null)}
                    onArchive={(id) => void handleArchive(id)}
                    onRestore={(id) => void conversations.restore(id)}
                    onDelete={(id) => void conversations.deleteArchived(id)}
                />
                <Separator />
                <div className="flex items-center gap-2 px-3 py-3">
                    <Avatar className="size-8">
                        <AvatarFallback className="text-xs">
                            {username.charAt(0).toUpperCase()}
                        </AvatarFallback>
                    </Avatar>
                    <span className="flex-1 truncate text-sm">{username}</span>
                    <Button variant="ghost" size="icon-sm" asChild>
                        <Link
                            to="/settings/providers"
                            aria-label={t("common.settings")}
                        >
                            <Settings className="size-4" />
                        </Link>
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        onClick={() => void logout()}
                        aria-label={t("common.logout")}
                    >
                        <LogOut className="size-4" />
                    </Button>
                </div>
            </aside>

            {/* Conversation */}
            <main className="flex min-w-0 flex-1 flex-col">
                <div
                    ref={scrollRef}
                    onScroll={handleScroll}
                    className="flex-1 overflow-y-auto"
                >
                    {!effectiveActiveId || chat.items.length === 0 ? (
                        <EmptyState
                            onPickSuggestion={(text) => {
                                setDraft(text);
                                inputRef.current?.focus();
                            }}
                        />
                    ) : (
                        <div className="mx-auto w-full max-w-3xl px-6 py-8">
                            <div className="space-y-8">
                                {chat.items.map((message) => (
                                    <MessageItem
                                        key={message.id}
                                        message={message}
                                        source={
                                            message.role === "assistant" &&
                                            message.model
                                                ? `${providerName(message.providerId) ?? ""} · ${message.model}`
                                                : undefined
                                        }
                                    />
                                ))}
                            </div>
                        </div>
                    )}
                </div>
                <ChatComposer
                    value={draft}
                    onChange={setDraft}
                    onSend={() => void handleSend()}
                    onStop={() => void chat.abort()}
                    streaming={chat.streaming}
                    options={modelOptions}
                    selectedKey={selectedModel?.key ?? null}
                    onSelect={handleSelectModel}
                    inputRef={inputRef}
                />
            </main>
        </div>
    );
}
