import { timestampDate } from "@bufbuild/protobuf/wkt";
import { ChevronRight } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import {
    SessionListItem,
    type SessionView,
} from "@/components/chat/SessionListItem";
import type { Conversation } from "@/gen/lemma/v1/conversation_pb";
import { cn } from "@/lib/utils";

interface SessionListProps {
    conversations: Conversation[];
    archived: Conversation[];
    activeId: string | null;
    editingId: string | null;
    onSelect: (id: string) => void;
    onStartRename: (id: string) => void;
    onCommitRename: (id: string, title: string) => void;
    onCancelRename: () => void;
    onArchive: (id: string) => void;
    onRestore: (id: string) => void;
    onDelete: (id: string) => void;
}

function isToday(c: Conversation) {
    if (!c.updatedAt) return false;
    const d = timestampDate(c.updatedAt);
    const now = new Date();
    return (
        d.getFullYear() === now.getFullYear() &&
        d.getMonth() === now.getMonth() &&
        d.getDate() === now.getDate()
    );
}

function GroupLabel({ children }: { children: React.ReactNode }) {
    return (
        <p className="px-2 pt-4 pb-1 text-xs text-muted-foreground">
            {children}
        </p>
    );
}

/** 会话列表：今天 / 更早 两组 + 可折叠的归档组（设计稿 §4.2） */
export function SessionList({
    conversations,
    archived,
    activeId,
    editingId,
    onSelect,
    onStartRename,
    onCommitRename,
    onCancelRename,
    onArchive,
    onRestore,
    onDelete,
}: SessionListProps) {
    const { t } = useTranslation();
    const [archivedOpen, setArchivedOpen] = useState(false);

    const toView = (c: Conversation): SessionView => ({
        id: c.id,
        // 未起过标题的会话用"新对话"占位
        title: c.title || t("chat.newChat"),
        messageCount: c.messageCount,
    });

    const today = conversations.filter(isToday);
    const earlier = conversations.filter((c) => !isToday(c));

    const renderItem = (c: Conversation, archivedView: boolean) => (
        <SessionListItem
            key={c.id}
            session={toView(c)}
            active={!archivedView && c.id === activeId}
            editing={c.id === editingId}
            archivedView={archivedView}
            onSelect={() => onSelect(c.id)}
            onStartRename={() => onStartRename(c.id)}
            onCommitRename={(title) => onCommitRename(c.id, title)}
            onCancelRename={onCancelRename}
            onArchive={() => onArchive(c.id)}
            onRestore={() => onRestore(c.id)}
            onDelete={() => onDelete(c.id)}
        />
    );

    return (
        <nav className="flex-1 overflow-y-auto px-2 pb-2">
            {today.length > 0 && (
                <>
                    <GroupLabel>{t("sessions.today")}</GroupLabel>
                    <div className="space-y-0.5">
                        {today.map((c) => renderItem(c, false))}
                    </div>
                </>
            )}
            {earlier.length > 0 && (
                <>
                    <GroupLabel>{t("sessions.earlier")}</GroupLabel>
                    <div className="space-y-0.5">
                        {earlier.map((c) => renderItem(c, false))}
                    </div>
                </>
            )}
            {archived.length > 0 && (
                <>
                    <button
                        type="button"
                        onClick={() => setArchivedOpen((v) => !v)}
                        className="flex w-full items-center gap-1 px-2 pt-4 pb-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
                    >
                        <ChevronRight
                            className={cn(
                                "size-3 transition-transform",
                                archivedOpen && "rotate-90",
                            )}
                        />
                        {t("sessions.archived")}
                        <span>({archived.length})</span>
                    </button>
                    {archivedOpen && (
                        <div className="space-y-0.5">
                            {archived.map((c) => renderItem(c, true))}
                        </div>
                    )}
                </>
            )}
        </nav>
    );
}
