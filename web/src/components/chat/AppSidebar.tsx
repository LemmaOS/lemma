import { useMemo } from "react";
import { Link } from "react-router";
import {
    Archive,
    ChevronDown,
    LogOut,
    PanelLeft,
    Plus,
    Settings,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { mockUser } from "@/mocks";
import type { Session } from "@/mocks";
import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ThemeToggle } from "@/components/ThemeToggle";
import { cn } from "@/lib/utils";

interface AppSidebarProps {
    /** Non-archived sessions shown in the sidebar list. */
    sessions: Session[];
    /** Currently open session (null = home view). */
    activeSessionId: string | null;
    /** Navigate to the home view (also used by "New chat"). */
    onGoHome: () => void;
    /** Open an existing session. */
    onOpenSession: (id: string) => void;
    /** Archive a session (removes it from the list). */
    onArchiveSession: (id: string) => void;
    /** Collapse the sidebar. */
    onCollapse: () => void;
}

type GroupKey = "today" | "yesterday" | "last7Days" | "earlier";

const GROUP_ORDER: GroupKey[] = ["today", "yesterday", "last7Days", "earlier"];
const DAY_MS = 24 * 60 * 60 * 1000;

function startOfDay(date: Date) {
    return new Date(
        date.getFullYear(),
        date.getMonth(),
        date.getDate(),
    ).getTime();
}

/** Application sidebar: product name, new chat, grouped sessions, user footer. */
export function AppSidebar({
    sessions,
    activeSessionId,
    onGoHome,
    onOpenSession,
    onArchiveSession,
    onCollapse,
}: AppSidebarProps) {
    const { t } = useTranslation();

    const stats = [
        { label: t("stats.sessions"), value: mockUser.stats.sessions },
        { label: t("stats.topics"), value: mockUser.stats.topics },
        { label: t("stats.messages"), value: mockUser.stats.messages },
    ];

    const sorted = useMemo(
        () =>
            [...sessions].sort(
                (a, b) => Date.parse(b.updatedAt) - Date.parse(a.updatedAt),
            ),
        [sessions],
    );

    // The mock data uses static dates, so "today" is anchored to the newest
    // updatedAt in the session list (not the wall clock); each group is a day
    // offset from that anchor: 0 = today, 1 = yesterday, 2–6 = last 7 days.
    const groups = useMemo(() => {
        if (sessions.length === 0)
            return [] as { key: GroupKey; items: Session[] }[];
        const anchor = startOfDay(
            new Date(Math.max(...sessions.map((s) => Date.parse(s.updatedAt)))),
        );
        const byGroup = new Map<GroupKey, Session[]>();
        for (const session of sorted) {
            const diff = Math.round(
                (anchor - startOfDay(new Date(session.updatedAt))) / DAY_MS,
            );
            const key: GroupKey =
                diff <= 0
                    ? "today"
                    : diff === 1
                      ? "yesterday"
                      : diff <= 6
                        ? "last7Days"
                        : "earlier";
            const items = byGroup.get(key) ?? [];
            items.push(session);
            byGroup.set(key, items);
        }
        return GROUP_ORDER.filter((key) => byGroup.has(key)).map((key) => ({
            key,
            items: byGroup.get(key)!,
        }));
    }, [sessions, sorted]);

    return (
        <aside className="h-full w-[260px] shrink-0 bg-transparent text-sidebar-foreground flex flex-col">
            {/* Product name + collapse */}
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

            {/* New chat */}
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

            {/* Sessions grouped by recency */}
            <div className="flex-1 overflow-y-auto pb-3">
                {groups.map((group) => (
                    <div key={group.key}>
                        <p className="text-xs text-muted-foreground px-3 pt-4 pb-1">
                            {t(`sessions.${group.key}`)}
                        </p>
                        <div className="space-y-0.5">
                            {group.items.map((session) => (
                                <div
                                    key={session.id}
                                    role="button"
                                    tabIndex={0}
                                    onClick={() => onOpenSession(session.id)}
                                    onKeyDown={(e) => {
                                        if (
                                            e.key === "Enter" ||
                                            e.key === " "
                                        ) {
                                            e.preventDefault();
                                            onOpenSession(session.id);
                                        }
                                    }}
                                    className={cn(
                                        "group flex w-full cursor-pointer items-center rounded-md px-3 py-1.5 text-sm truncate hover:bg-accent/60 transition-colors",
                                        session.id === activeSessionId &&
                                            "bg-sidebar-accent font-medium",
                                    )}
                                >
                                    <span className="flex-1 truncate text-left">
                                        {session.title}
                                    </span>
                                    <button
                                        type="button"
                                        aria-label={t("sessions.archive")}
                                        title={t("sessions.archive")}
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            onArchiveSession(session.id);
                                        }}
                                        className="grid size-5 shrink-0 place-items-center rounded text-muted-foreground opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100"
                                    >
                                        <Archive className="size-3.5" />
                                    </button>
                                </div>
                            ))}
                        </div>
                    </div>
                ))}
            </div>

            {/* User area */}
            <div className="border-t border-sidebar-border p-3 flex items-center gap-2">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <button
                            type="button"
                            className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-1 py-1 hover:bg-sidebar-accent transition-colors"
                        >
                            <span className="size-7 shrink-0 rounded-full bg-muted grid place-items-center text-xs">
                                {mockUser.name.charAt(0)}
                            </span>
                            <span className="flex-1 truncate text-left text-sm">
                                {mockUser.name}
                            </span>
                            <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />
                        </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent
                        align="start"
                        side="top"
                        className="w-64"
                    >
                        <div className="px-2 py-2">
                            <p className="text-sm font-medium">
                                {mockUser.name}
                            </p>
                            <div className="mt-3 grid grid-cols-3 gap-2">
                                {stats.map((stat) => (
                                    <div key={stat.label}>
                                        <p className="text-sm font-semibold tabular-nums">
                                            {stat.value}
                                        </p>
                                        <p className="text-xs text-muted-foreground">
                                            {stat.label}
                                        </p>
                                    </div>
                                ))}
                            </div>
                        </div>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem asChild>
                            <Link to="/settings/providers">
                                <Settings className="size-4" />
                                {t("sidebar.appSettings")}
                            </Link>
                        </DropdownMenuItem>
                        <DropdownMenuItem asChild>
                            <Link to="/login">
                                <LogOut className="size-4" />
                                {t("sidebar.signOut")}
                            </Link>
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
                <ThemeToggle />
            </div>
        </aside>
    );
}
