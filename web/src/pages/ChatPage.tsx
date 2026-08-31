import { timestampDate } from "@bufbuild/protobuf/wkt";
import { PanelLeft } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { AppSidebar } from "@/components/chat/AppSidebar";
import { ChatComposer } from "@/components/chat/ChatComposer";
import { EmptyState } from "@/components/chat/EmptyState";
import { HomeView } from "@/components/chat/HomeView";
import { MessageItem } from "@/components/chat/MessageItem";
import { type ModelSelection } from "@/components/chat/ModelSwitcher";
import { Button } from "@/components/ui/button";
import type { Conversation } from "@/gen/lemma/v1/conversation_pb";
import { useChat } from "@/hooks/useChat";
import { useConversations } from "@/hooks/useConversations";
import { useProviders } from "@/hooks/useProviders";
import type { SessionSummary } from "@/lib/sessionGrouping";
import { cn } from "@/lib/utils";
import type { ChatItem } from "@/stores/chat";

const SIDEBAR_COLLAPSED_KEY = "sidebar-collapsed";
const MODEL_KEY = "lemma.model";

function toSummary(c: Conversation): SessionSummary {
    return {
        id: c.id,
        title: c.title,
        updatedAtMs: c.updatedAt ? timestampDate(c.updatedAt).getTime() : 0,
        messageCount: c.messageCount,
    };
}

export default function ChatPage() {
    const { t } = useTranslation();
    const conversations = useConversations();
    const chat = useChat();
    const providersStore = useProviders();

    const [activeId, setActiveId] = useState<string | null>(null);
    const [sidebarCollapsed, setSidebarCollapsed] = useState(
        () => localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "1",
    );
    const [draft, setDraft] = useState("");

    const scrollRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLTextAreaElement>(null);

    const toggleSidebar = (collapsed: boolean) => {
        setSidebarCollapsed(collapsed);
        localStorage.setItem(SIDEBAR_COLLAPSED_KEY, collapsed ? "1" : "0");
    };

    // ---------- 模型选择（持久化 + 失效回退） ----------

    const enabledProviders = useMemo(
        () =>
            providersStore.list.filter((p) => p.enabled && p.models.length > 0),
        [providersStore.list],
    );

    const [stored, setStored] = useState<ModelSelection | null>(() => {
        try {
            const raw = localStorage.getItem(MODEL_KEY);
            const parsed = raw ? (JSON.parse(raw) as ModelSelection) : null;
            return parsed?.providerId && parsed?.model ? parsed : null;
        } catch {
            return null;
        }
    });

    const model = useMemo(() => {
        if (
            stored &&
            enabledProviders.some(
                (p) =>
                    p.id === stored.providerId &&
                    p.models.includes(stored.model),
            )
        ) {
            return stored;
        }
        const first = enabledProviders[0];
        return first ? { providerId: first.id, model: first.models[0] } : null;
    }, [stored, enabledProviders]);

    const selectModel = (selection: ModelSelection) => {
        localStorage.setItem(MODEL_KEY, JSON.stringify(selection));
        setStored(selection);
    };

    // ---------- 会话与消息 ----------

    // 打开会话：chat store 缓存优先，离线也能开
    useEffect(() => {
        if (activeId) void chat.open(activeId);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [activeId]);

    useEffect(() => {
        const el = scrollRef.current;
        if (el) el.scrollTop = el.scrollHeight;
    }, [chat.items, activeId]);

    const sendText = async (text: string) => {
        if (!model || chat.streaming) return;
        try {
            let cid = activeId;
            if (!cid) {
                cid = await conversations.create();
                setActiveId(cid);
                await chat.open(cid);
            }
            await chat.send(model.providerId, model.model, text);
        } catch {
            // create/open 的网络错误静默；流内错误由 store 转 error 项
        }
    };

    const handleSend = () => {
        const text = draft.trim();
        if (!text) return;
        setDraft("");
        void sendText(text);
    };

    const handleStop = () => {
        void chat.abort();
    };

    // 重新生成 = 用当前模型重发该回复前面最近一条用户消息
    const handleRegenerate = (messageId: string) => {
        if (chat.streaming || !model) return;
        const idx = chat.items.findIndex((m) => m.id === messageId);
        if (idx < 0) return;
        for (let i = idx - 1; i >= 0; i--) {
            const prev = chat.items[i];
            if (prev.role === "user") {
                void chat.send(model.providerId, model.model, prev.content);
                return;
            }
        }
    };

    const handleArchive = async (id: string) => {
        await conversations.archive(id);
        if (id === activeId) setActiveId(null);
    };

    const handleRename = async (id: string, title: string) => {
        await conversations.rename(id, title);
    };

    const handleRestore = async (id: string) => {
        await conversations.restore(id);
    };

    // 彻底删除，二次确认
    const handleDelete = async (id: string) => {
        if (window.confirm(t("sessions.deleteConfirm"))) {
            await conversations.deleteArchived(id);
        }
    };

    const handlePickSuggestion = (text: string) => {
        setDraft(text);
        inputRef.current?.focus();
    };

    // ---------- 派生 ----------

    const summaries = useMemo(
        () => conversations.list.map(toSummary),
        [conversations.list],
    );
    const archivedSummaries = useMemo(
        () => conversations.archived.map(toSummary),
        [conversations.archived],
    );

    const providerNameById = useMemo(() => {
        const map = new Map<string, string>();
        for (const p of providersStore.list) map.set(p.id, p.name);
        return map;
    }, [providersStore.list]);

    const sourceOf = (m: ChatItem) => {
        if (!m.model) return undefined;
        const name = providerNameById.get(m.providerId);
        return name ? `${name} · ${m.model}` : m.model;
    };

    const lastAssistantId = useMemo(() => {
        for (let i = chat.items.length - 1; i >= 0; i--) {
            if (chat.items[i].role === "assistant") return chat.items[i].id;
        }
        return null;
    }, [chat.items]);

    return (
        <div className="flex h-dvh bg-sidebar text-foreground">
            {/* 侧边栏：折叠时宽度/透明度动画 */}
            <div
                aria-hidden={sidebarCollapsed}
                className={cn(
                    "shrink-0 overflow-hidden transition-all duration-200 ease-out",
                    sidebarCollapsed
                        ? "w-0 opacity-0"
                        : "w-[260px] opacity-100",
                )}
            >
                <AppSidebar
                    sessions={summaries}
                    archived={archivedSummaries}
                    activeSessionId={activeId}
                    onGoHome={() => setActiveId(null)}
                    onOpenSession={setActiveId}
                    onArchiveSession={(id) => void handleArchive(id)}
                    onRenameSession={(id, title) =>
                        void handleRename(id, title)
                    }
                    onRestoreSession={(id) => void handleRestore(id)}
                    onDeleteSession={(id) => void handleDelete(id)}
                    onCollapse={() => toggleSidebar(true)}
                />
            </div>

            <main
                className={cn(
                    "min-w-0 flex-1 transition-[padding] duration-200 ease-out",
                    sidebarCollapsed ? "p-2" : "py-2 pr-2",
                )}
            >
                <div className="relative flex h-full flex-col overflow-hidden rounded-xl border border-border app-canvas">
                    {sidebarCollapsed && (
                        <div className="absolute left-3 top-3 z-10">
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                className="size-8 rounded-lg border border-border bg-background text-muted-foreground"
                                onClick={() => toggleSidebar(false)}
                                aria-label={t("sidebar.expand")}
                                title={t("sidebar.expand")}
                            >
                                <PanelLeft className="size-4" />
                            </Button>
                        </div>
                    )}
                    {activeId === null ? (
                        <HomeView
                            onSubmit={(text) => void sendText(text)}
                            model={model}
                            onModelChange={selectModel}
                        />
                    ) : (
                        <>
                            <div
                                ref={scrollRef}
                                className="flex-1 overflow-y-auto"
                            >
                                {chat.items.length === 0 ? (
                                    <EmptyState
                                        onPickSuggestion={handlePickSuggestion}
                                    />
                                ) : (
                                    <div className="max-w-3xl mx-auto w-full px-6 pt-10 pb-6 space-y-8">
                                        {chat.items.map((m) => (
                                            <MessageItem
                                                key={m.id}
                                                message={m}
                                                source={sourceOf(m)}
                                                canRegenerate={
                                                    m.id === lastAssistantId
                                                }
                                                onRegenerate={handleRegenerate}
                                            />
                                        ))}
                                    </div>
                                )}
                            </div>
                            <ChatComposer
                                value={draft}
                                onChange={setDraft}
                                onSend={handleSend}
                                onStop={handleStop}
                                streaming={chat.streaming}
                                model={model}
                                onModelChange={selectModel}
                                inputRef={inputRef}
                            />
                        </>
                    )}
                </div>
            </main>
        </div>
    );
}
