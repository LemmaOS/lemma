import {
    Archive,
    ChevronDown,
    ChevronRight,
    LogOut,
    PanelLeft,
    Pencil,
    Plus,
    RotateCcw,
    Settings,
    Trash2,
} from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { LanguageToggle } from "@/components/LanguageToggle";
import { SyncIndicator } from "@/components/SyncIndicator";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Button } from "@/components/ui/button";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { groupSessions, type SessionSummary } from "@/lib/sessionGrouping";
import { cn } from "@/lib/utils";
import { useAuth } from "@/stores/auth";

interface AppSidebarProps {
    sessions: SessionSummary[];
    archived: SessionSummary[];
    activeSessionId: string | null;
    onGoHome: () => void;
    onOpenSession: (id: string) => void;
    onArchiveSession: (id: string) => void;
    onRenameSession: (id: string, title: string) => void;
    onRestoreSession: (id: string) => void;
    onDeleteSession: (id: string) => void;
    onCollapse: () => void;
}

/** 会话行：悬停出现 重命名/归档；重命名为非受控输入（defaultValue + 提交时取值） */
function SessionRow({
    session,
    active,
    onOpen,
    onArchive,
    onRename,
}: {
    session: SessionSummary;
    active: boolean;
    onOpen: (id: string) => void;
    onArchive: (id: string) => void;
    onRename: (id: string, title: string) => void;
}) {
    const { t } = useTranslation();
    const [editing, setEditing] = useState(false);
    const inputRef = useRef<HTMLInputElement | null>(null);
    const focusRef = useCallback((el: HTMLInputElement | null) => {
        inputRef.current = el;
        el?.select();
    }, []);

    const commit = () => {
        const title = inputRef.current?.value.trim() ?? "";
        setEditing(false);
        if (title && title !== session.title) onRename(session.id, title);
    };

    if (editing) {
        return (
            <Input
                ref={focusRef}
                defaultValue={session.title}
                onBlur={commit}
                onKeyDown={(e) => {
                    if (e.key === "Enter") {
                        e.preventDefault();
                        commit();
                    } else if (e.key === "Escape") {
                        setEditing(false);
                    }
                }}
                className="h-7 rounded-md px-3 text-sm"
            />
        );
    }

    return (
        <div
            role="button"
            tabIndex={0}
            onClick={() => onOpen(session.id)}
            onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onOpen(session.id);
                }
            }}
            className={cn(
                "group flex w-full cursor-pointer items-center rounded-md px-3 py-1.5 text-sm truncate hover:bg-accent/60 transition-colors",
                active && "bg-sidebar-accent font-medium",
            )}
        >
            <span className="flex-1 truncate text-left">
                {session.title || t("sidebar.newChat")}
            </span>
            <span className="flex shrink-0 opacity-0 transition-opacity group-hover:opacity-100">
                <button
                    type="button"
                    aria-label={t("sessions.rename")}
                    title={t("sessions.rename")}
                    onClick={(e) => {
                        e.stopPropagation();
                        setEditing(true);
                    }}
                    className="grid size-5 place-items-center rounded text-muted-foreground hover:text-foreground"
                >
                    <Pencil className="size-3" />
                </button>
                <button
                    type="button"
                    aria-label={t("sessions.archive")}
                    title={t("sessions.archive")}
                    onClick={(e) => {
                        e.stopPropagation();
                        onArchive(session.id);
                    }}
                    className="grid size-5 place-items-center rounded text-muted-foreground hover:text-foreground"
                >
                    <Archive className="size-3.5" />
                </button>
            </span>
        </div>
    );
}

/** 归档行：悬停出现 恢复/彻底删除 */
function ArchivedRow({
    session,
    onRestore,
    onDelete,
}: {
    session: SessionSummary;
    onRestore: (id: string) => void;
    onDelete: (id: string) => void;
}) {
    const { t } = useTranslation();
    return (
        <div className="group flex w-full items-center rounded-md px-3 py-1.5 text-sm truncate hover:bg-accent/60 transition-colors">
            <span className="flex-1 truncate text-left text-muted-foreground">
                {session.title || t("sidebar.newChat")}
            </span>
            <span className="flex shrink-0 opacity-0 transition-opacity group-hover:opacity-100">
                <button
                    type="button"
                    aria-label={t("sessions.restore")}
                    title={t("sessions.restore")}
                    onClick={() => onRestore(session.id)}
                    className="grid size-5 place-items-center rounded text-muted-foreground hover:text-foreground"
                >
                    <RotateCcw className="size-3" />
                </button>
                <button
                    type="button"
                    aria-label={t("sessions.delete")}
                    title={t("sessions.delete")}
                    onClick={() => onDelete(session.id)}
                    className="grid size-5 place-items-center rounded text-muted-foreground hover:text-destructive"
                >
                    <Trash2 className="size-3.5" />
                </button>
            </span>
        </div>
    );
}

/** 应用侧边栏：产品名、新会话、分组会话列表、归档区、用户区 */
export function AppSidebar({
    sessions,
    archived,
    activeSessionId,
    onGoHome,
    onOpenSession,
    onArchiveSession,
    onRenameSession,
    onRestoreSession,
    onDeleteSession,
    onCollapse,
}: AppSidebarProps) {
    const { t } = useTranslation();
    const username = useAuth((s) => s.user?.username ?? "");
    const logout = useAuth((s) => s.logout);

    const groups = useMemo(() => groupSessions(sessions), [sessions]);

    return (
        <aside className="h-full w-[260px] shrink-0 bg-transparent text-sidebar-foreground flex flex-col">
            {/* 产品名 + 收起 */}
            <div className="flex items-center justify-between px-3 pt-4">
                <p className="text-sm font-semibold">{t("common.appName")}</p>
                <Button
                    variant="ghost"
                    size="icon-sm"
                    className="size-7 text-muted-foreground"
                    onClick={onCollapse}
                    aria-label={t("sidebar.collapse")}
                    title={t("sidebar.collapse")}
                >
                    <PanelLeft className="size-4" />
                </Button>
            </div>

            {/* 新会话 */}
            <div className="px-3 pt-3">
                <button
                    type="button"
                    onClick={onGoHome}
                    className="flex w-full items-center gap-2 rounded-lg border border-border bg-background px-3.5 h-9 text-sm hover:bg-accent transition-colors"
                >
                    <Plus className="size-4" />
                    {t("sidebar.newChat")}
                </button>
            </div>

            {/* 会话列表 */}
            <div className="flex-1 overflow-y-auto pb-3">
                {groups.map((group) => (
                    <div key={group.key}>
                        <p className="text-xs text-muted-foreground px-3 pt-4 pb-1">
                            {t(`sessions.${group.key}`)}
                        </p>
                        <div className="space-y-0.5">
                            {group.items.map((session) => (
                                <SessionRow
                                    key={session.id}
                                    session={session}
                                    active={session.id === activeSessionId}
                                    onOpen={onOpenSession}
                                    onArchive={onArchiveSession}
                                    onRename={onRenameSession}
                                />
                            ))}
                        </div>
                    </div>
                ))}

                {archived.length > 0 && (
                    <Collapsible>
                        <CollapsibleTrigger className="group flex w-full items-center gap-1 px-3 pt-4 pb-1 text-xs text-muted-foreground">
                            <ChevronRight className="size-3 transition-transform group-data-[state=open]:rotate-90" />
                            {t("sessions.archived")} ({archived.length})
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <div className="space-y-0.5">
                                {archived.map((session) => (
                                    <ArchivedRow
                                        key={session.id}
                                        session={session}
                                        onRestore={onRestoreSession}
                                        onDelete={onDeleteSession}
                                    />
                                ))}
                            </div>
                        </CollapsibleContent>
                    </Collapsible>
                )}
            </div>

            {/* 用户区 */}
            <div className="border-t border-sidebar-border p-3 flex items-center gap-2">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <button
                            type="button"
                            className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-1 py-1 hover:bg-sidebar-accent transition-colors"
                        >
                            <span className="size-7 shrink-0 rounded-full bg-muted grid place-items-center text-xs">
                                {username.charAt(0)}
                            </span>
                            <span className="flex-1 truncate text-left text-sm">
                                {username}
                            </span>
                            <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />
                        </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent
                        align="start"
                        side="top"
                        className="w-56"
                    >
                        <div className="px-2 py-2">
                            <p className="text-sm font-medium">{username}</p>
                        </div>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem asChild>
                            <Link to="/settings/providers">
                                <Settings className="size-4" />
                                {t("sidebar.appSettings")}
                            </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem
                            onClick={() => {
                                void logout();
                            }}
                        >
                            <LogOut className="size-4" />
                            {t("sidebar.signOut")}
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
                <SyncIndicator />
                <LanguageToggle />
                <ThemeToggle />
            </div>
        </aside>
    );
}
