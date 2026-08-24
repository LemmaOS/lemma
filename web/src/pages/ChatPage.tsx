import { useEffect, useMemo, useRef, useState } from "react";
import { PanelLeft } from "lucide-react";
import { useTranslation } from "react-i18next";
import { mockMessages, mockProviders, mockSessions } from "@/mocks";
import type { ChatMessage, Session } from "@/mocks";
import { AppSidebar } from "@/components/chat/AppSidebar";
import { ChatComposer } from "@/components/chat/ChatComposer";
import { EmptyState } from "@/components/chat/EmptyState";
import { HomeView } from "@/components/chat/HomeView";
import { MessageItem } from "@/components/chat/MessageItem";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const STREAM_DURATION_MS = 2500;
const TITLE_MAX_LENGTH = 40;
const SIDEBAR_COLLAPSED_KEY = "sidebar-collapsed";

/** Streaming sample blocks reused for demo replies. */
const streamingSample = mockMessages.find((m) => m.streaming)?.blocks ?? [];

export default function ChatPage() {
    const { t } = useTranslation();
    const [sessions, setSessions] = useState<Session[]>(mockSessions);
    const [activeId, setActiveId] = useState<string | null>(null);
    const [sidebarCollapsed, setSidebarCollapsed] = useState(
        () => localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "1",
    );

    const toggleSidebar = (collapsed: boolean) => {
        setSidebarCollapsed(collapsed);
        localStorage.setItem(SIDEBAR_COLLAPSED_KEY, collapsed ? "1" : "0");
    };

    const [messagesById, setMessagesById] = useState<
        Record<string, ChatMessage[]>
    >(() =>
        Object.fromEntries(
            mockSessions
                .filter((s) => !s.archived)
                .slice(0, 3)
                .map((s) => [s.id, mockMessages]),
        ),
    );
    const [draft, setDraft] = useState("");
    const [model, setModel] = useState(mockProviders[0].models[0]);

    const scrollRef = useRef<HTMLDivElement>(null);
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const streamTimer = useRef<number | null>(null);

    const messages = useMemo(
        () => (activeId ? (messagesById[activeId] ?? []) : []),
        [messagesById, activeId],
    );
    const streaming = messages.some((m) => m.streaming);

    useEffect(() => {
        const el = scrollRef.current;
        if (el) el.scrollTop = el.scrollHeight;
    }, [messages, activeId]);

    useEffect(
        () => () => {
            if (streamTimer.current !== null)
                window.clearTimeout(streamTimer.current);
        },
        [],
    );

    const patchMessages = (
        sessionId: string,
        updater: (list: ChatMessage[]) => ChatMessage[],
    ) => {
        setMessagesById((prev) => ({
            ...prev,
            [sessionId]: updater(prev[sessionId] ?? []),
        }));
    };

    const finishStreaming = (sessionId: string, messageId: string) => {
        patchMessages(sessionId, (list) =>
            list.map((m) =>
                m.id === messageId ? { ...m, streaming: false } : m,
            ),
        );
        streamTimer.current = null;
    };

    const scheduleFinish = (sessionId: string, messageId: string) => {
        if (streamTimer.current !== null)
            window.clearTimeout(streamTimer.current);
        streamTimer.current = window.setTimeout(
            () => finishStreaming(sessionId, messageId),
            STREAM_DURATION_MS,
        );
    };

    const sourceForModel = (modelId: string) => {
        const provider = mockProviders.find(
            (p) => p.configured && p.models.includes(modelId),
        );
        return provider ? `${provider.type} · ${modelId}` : modelId;
    };

    /** Append a user message + a streaming demo reply to a session. */
    const sendToSession = (sessionId: string, text: string) => {
        const now = Date.now();
        const userMessage: ChatMessage = {
            id: `m-${now}-u`,
            role: "user",
            blocks: [{ type: "paragraph", segments: [{ type: "text", text }] }],
        };
        const aiMessage: ChatMessage = {
            id: `m-${now}-a`,
            role: "assistant",
            source: sourceForModel(model),
            streaming: true,
            blocks: streamingSample,
        };
        patchMessages(sessionId, (list) => [...list, userMessage, aiMessage]);
        setSessions((prev) =>
            prev.map((s) =>
                s.id === sessionId
                    ? {
                          ...s,
                          updatedAt: new Date(now).toISOString(),
                          messageCount: s.messageCount + 2,
                      }
                    : s,
            ),
        );
        scheduleFinish(sessionId, aiMessage.id);
    };

    const handleSend = () => {
        const text = draft.trim();
        if (!text || streaming || !activeId) return;
        sendToSession(activeId, text);
        setDraft("");
    };

    const handleStop = () => {
        if (streamTimer.current !== null) {
            window.clearTimeout(streamTimer.current);
            streamTimer.current = null;
        }
        if (!activeId) return;
        patchMessages(activeId, (list) =>
            list.map((m) => (m.streaming ? { ...m, streaming: false } : m)),
        );
    };

    const handleRegenerate = (messageId: string) => {
        if (!activeId) return;
        patchMessages(activeId, (list) =>
            list.map((m) =>
                m.id === messageId ? { ...m, streaming: true } : m,
            ),
        );
        scheduleFinish(activeId, messageId);
    };

    /** Home input submit: create a new session with the first exchange. */
    const handleHomeSubmit = (text: string) => {
        const id = `s-${Date.now()}`;
        const session: Session = {
            id,
            title:
                text.length > TITLE_MAX_LENGTH
                    ? `${text.slice(0, TITLE_MAX_LENGTH)}…`
                    : text,
            updatedAt: new Date().toISOString(),
            messageCount: 0,
        };
        setSessions((prev) => [session, ...prev]);
        setActiveId(id);
        sendToSession(id, text);
    };

    /** Sidebar hover action: archive a session (front-end state only). */
    const handleArchiveSession = (id: string) => {
        setSessions((prev) =>
            prev.map((s) => (s.id === id ? { ...s, archived: true } : s)),
        );
        if (activeId === id) setActiveId(null);
    };

    const handlePickSuggestion = (text: string) => {
        setDraft(text);
        inputRef.current?.focus();
    };

    return (
        <div className="flex h-dvh bg-sidebar text-foreground">
            {/* Sidebar wrapper: animated width/opacity for smooth collapse */}
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
                    sessions={sessions.filter((s) => !s.archived)}
                    activeSessionId={activeId}
                    onGoHome={() => setActiveId(null)}
                    onOpenSession={setActiveId}
                    onArchiveSession={handleArchiveSession}
                    onCollapse={() => toggleSidebar(true)}
                />
            </div>

            <main
                className={cn(
                    "min-w-0 flex-1 transition-[padding] duration-200 ease-out",
                    sidebarCollapsed ? "p-2" : "py-2 pr-2",
                )}
            >
                <div className="relative flex h-full flex-col overflow-hidden rounded-xl border border-border bg-background">
                    {sidebarCollapsed && (
                        <div className="absolute left-3 top-3 z-10">
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                className="size-8 rounded-lg border border-border bg-background text-muted-foreground shadow-xs"
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
                            onSubmit={handleHomeSubmit}
                            model={model}
                            onModelChange={setModel}
                        />
                    ) : (
                        <>
                            <div
                                ref={scrollRef}
                                className="flex-1 overflow-y-auto"
                            >
                                {messages.length === 0 ? (
                                    <EmptyState
                                        onPickSuggestion={handlePickSuggestion}
                                    />
                                ) : (
                                    <div className="max-w-3xl mx-auto w-full px-6 pt-10 pb-6 space-y-8">
                                        {messages.map((message) => (
                                            <MessageItem
                                                key={message.id}
                                                message={message}
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
                                streaming={streaming}
                                model={model}
                                onModelChange={setModel}
                                inputRef={inputRef}
                            />
                        </>
                    )}
                </div>
            </main>
        </div>
    );
}
