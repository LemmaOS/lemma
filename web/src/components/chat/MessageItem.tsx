import { useState } from "react";
import {
    Check,
    Copy,
    RefreshCw,
    Sparkles,
    ThumbsDown,
    ThumbsUp,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ChatMessage } from "@/mocks";
import { Button } from "@/components/ui/button";
import {
    MessageBlocks,
    blocksToPlainText,
} from "@/components/chat/MessageBlocks";
import { cn } from "@/lib/utils";

interface MessageItemProps {
    message: ChatMessage;
    onRegenerate?: (id: string) => void;
}

/** User messages are right-aligned bubbles; assistant messages have an avatar and always-visible actions. */
export function MessageItem({ message, onRegenerate }: MessageItemProps) {
    const { t } = useTranslation();
    const [copied, setCopied] = useState(false);
    const [feedback, setFeedback] = useState<"up" | "down" | null>(null);

    if (message.role === "user") {
        return (
            <div className="flex justify-end">
                <div className="max-w-[75%] rounded-2xl bg-muted px-4 py-2.5 text-sm">
                    <MessageBlocks blocks={message.blocks} />
                </div>
            </div>
        );
    }

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(
                blocksToPlainText(message.blocks),
            );
        } catch {
            /* clipboard unavailable in demo */
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
                <MessageBlocks blocks={message.blocks} />
                {message.streaming && (
                    <span className="inline-block h-4 w-[7px] rounded-[1px] bg-foreground/70 animate-pulse align-text-bottom mt-1" />
                )}
                {message.source && (
                    <p className="text-xs text-muted-foreground mt-3">
                        {message.source}
                    </p>
                )}
                <div className="flex gap-1 mt-2">
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        className="size-7 text-muted-foreground"
                        onClick={handleCopy}
                        aria-label={copied ? t("chat.copied") : t("chat.copy")}
                    >
                        {copied ? (
                            <Check className="size-3.5" />
                        ) : (
                            <Copy className="size-3.5" />
                        )}
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        className={cn(
                            "size-7 text-muted-foreground",
                            feedback === "up" && "text-foreground",
                        )}
                        onClick={() =>
                            setFeedback(feedback === "up" ? null : "up")
                        }
                        aria-label={t("chat.like")}
                    >
                        <ThumbsUp className="size-3.5" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        className={cn(
                            "size-7 text-muted-foreground",
                            feedback === "down" && "text-foreground",
                        )}
                        onClick={() =>
                            setFeedback(feedback === "down" ? null : "down")
                        }
                        aria-label={t("chat.dislike")}
                    >
                        <ThumbsDown className="size-3.5" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        className="size-7 text-muted-foreground"
                        onClick={() => onRegenerate?.(message.id)}
                        aria-label={t("chat.regenerate")}
                    >
                        <RefreshCw className="size-3.5" />
                    </Button>
                </div>
            </div>
        </div>
    );
}
