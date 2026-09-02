import { Check, Copy, RefreshCw, Sparkles } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { MessageContent } from "@/components/chat/MessageContent";
import { Button } from "@/components/ui/button";
import type { ChatItem } from "@/stores/chat";

interface MessageItemProps {
    message: ChatItem;
    source?: string;
    canRegenerate?: boolean;
    onRegenerate?: (id: string) => void;
}

export function MessageItem({
    message,
    source,
    canRegenerate,
    onRegenerate,
}: MessageItemProps) {
    const { t } = useTranslation();
    const [copied, setCopied] = useState(false);

    const streaming = message.status === "streaming";

    if (message.role === "user") {
        return (
            <div className="flex justify-end">
                <div className="max-w-[75%] rounded-xl bg-muted px-4 py-2.5 text-sm whitespace-pre-wrap break-words">
                    {message.content}
                </div>
            </div>
        );
    }

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(message.content);
        } catch {
            // Intentionally empty.
        }
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
    };

    return (
        <div className="flex gap-3">
            <div className="size-7 shrink-0 rounded-full bg-foreground text-background grid place-items-center">
                <Sparkles className="size-3.5" />
            </div>
            <div className="min-w-0 flex-1">
                <MessageContent content={message.content} />
                {streaming && (
                    <span className="mt-1 inline-block h-4 w-[7px] animate-pulse rounded-[1px] bg-foreground/70 align-text-bottom" />
                )}
                {message.status === "error" && message.error && (
                    <p className="mt-2 text-sm text-destructive">
                        {message.error}
                    </p>
                )}
                {message.status === "aborted" && (
                    <p className="mt-2 text-xs text-muted-foreground">
                        {t("chat.abortedNotice")}
                    </p>
                )}
                {!streaming && source && (
                    <p className="mt-3 text-xs text-muted-foreground">
                        {source}
                    </p>
                )}
                {!streaming && (
                    <div className="mt-2 flex gap-1">
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            className="size-7 text-muted-foreground"
                            onClick={handleCopy}
                            aria-label={
                                copied ? t("chat.copied") : t("chat.copy")
                            }
                        >
                            {copied ? (
                                <Check className="size-3.5" />
                            ) : (
                                <Copy className="size-3.5" />
                            )}
                        </Button>
                        {canRegenerate && onRegenerate && (
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                className="size-7 text-muted-foreground"
                                onClick={() => onRegenerate(message.id)}
                                aria-label={t("chat.regenerate")}
                            >
                                <RefreshCw className="size-3.5" />
                            </Button>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
