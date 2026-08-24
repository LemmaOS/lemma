import { Archive, Pencil, RotateCcw, Trash2 } from "lucide-react";
import { useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface SessionView {
    id: string;
    title: string;
    messageCount: number;
}

interface SessionListItemProps {
    session: SessionView;
    active: boolean;
    editing: boolean;
    /** 归档分组里的行：显示恢复/删除而不是重命名/归档 */
    archivedView: boolean;
    onSelect: () => void;
    onStartRename: () => void;
    onCommitRename: (title: string) => void;
    onCancelRename: () => void;
    onArchive: () => void;
    onRestore: () => void;
    onDelete: () => void;
}

/** 侧栏会话行（设计稿 §3.3）：截断标题，悬停出现操作按钮 */
export function SessionListItem({
    session,
    active,
    editing,
    archivedView,
    onSelect,
    onStartRename,
    onCommitRename,
    onCancelRename,
    onArchive,
    onRestore,
    onDelete,
}: SessionListItemProps) {
    const { t } = useTranslation();
    const inputRef = useRef<HTMLInputElement>(null);

    // 挂载时聚焦并全选标题；useCallback 保持引用稳定，只跑挂载这一次
    const focusRef = useCallback((el: HTMLInputElement | null) => {
        inputRef.current = el;
        el?.select();
    }, []);

    const commit = () => {
        const title = (inputRef.current?.value ?? "").trim();
        if (title) onCommitRename(title);
        else onCancelRename();
    };

    if (editing) {
        return (
            <div className="px-2 py-1">
                <input
                    ref={focusRef}
                    defaultValue={session.title}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") commit();
                        if (e.key === "Escape") onCancelRename();
                    }}
                    onBlur={commit}
                    className="w-full rounded-md border border-input bg-background px-2 py-1 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    aria-label={t("sessions.rename")}
                />
            </div>
        );
    }

    return (
        <div
            role="button"
            tabIndex={0}
            onClick={onSelect}
            onKeyDown={(e) => {
                if (e.key === "Enter") onSelect();
            }}
            className={cn(
                "group relative flex w-full cursor-pointer items-center rounded-md px-2 py-1.5 text-sm text-sidebar-foreground hover:bg-sidebar-accent/60",
                active && "bg-sidebar-accent text-sidebar-accent-foreground",
            )}
        >
            <span className="truncate pr-8">{session.title}</span>
            <span className="ml-auto shrink-0 text-xs text-muted-foreground transition-opacity group-hover:opacity-0">
                {session.messageCount}
            </span>
            <span className="absolute right-1 flex items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
                {archivedView ? (
                    <>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={(e) => {
                                e.stopPropagation();
                                onRestore();
                            }}
                            aria-label={t("sessions.restore")}
                        >
                            <RotateCcw className="size-3.5" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={(e) => {
                                e.stopPropagation();
                                if (window.confirm(t("sessions.deleteConfirm")))
                                    onDelete();
                            }}
                            aria-label={t("sessions.delete")}
                        >
                            <Trash2 className="size-3.5" />
                        </Button>
                    </>
                ) : (
                    <>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={(e) => {
                                e.stopPropagation();
                                onStartRename();
                            }}
                            aria-label={t("sessions.rename")}
                        >
                            <Pencil className="size-3.5" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={(e) => {
                                e.stopPropagation();
                                onArchive();
                            }}
                            aria-label={t("sessions.archive")}
                        >
                            <Archive className="size-3.5" />
                        </Button>
                    </>
                )}
            </span>
        </div>
    );
}
