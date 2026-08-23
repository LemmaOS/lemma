import { Check, Copy } from "lucide-react";
import { lazy, Suspense, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import type { ChatItem } from "@/stores/chat";

// react-markdown + highlight.js 是最大的依赖块，单独拆包按需加载
const MessageContent = lazy(() =>
    import("@/components/chat/MessageContent").then((m) => ({
        default: m.MessageContent,
    })),
);

interface MessageItemProps {
    message: ChatItem;
    /** 形如 "OpenAI · gpt-4o" 的来源标注，仅 assistant 有 */
    source?: string;
}

/** 消息行（设计稿 §3.2）：user 为色块，assistant 为纯排版流 */
export function MessageItem({ message, source }: MessageItemProps) {
    const { t } = useTranslation();
    const [copied, setCopied] = useState(false);

    if (message.role === "user") {
        return (
            <div className="rounded-lg bg-secondary/60 px-4 py-3">
                <div className="text-sm leading-relaxed whitespace-pre-wrap">
                    {message.content}
                </div>
            </div>
        );
    }

    const providerInitial = (source?.trim().charAt(0) || "A").toUpperCase();

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(message.content);
        } catch {
            // 剪贴板不可用（非安全上下文）时静默
        }
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
    };

    return (
        <div className="group">
            <div className="flex items-start gap-3">
                <div className="mt-1 grid size-4 shrink-0 place-items-center rounded-full bg-primary text-[10px] leading-none font-semibold text-primary-foreground">
                    {providerInitial}
                </div>
                <div className="min-w-0 flex-1">
                    {message.content && (
                        <Suspense
                            fallback={
                                <div className="text-sm leading-relaxed whitespace-pre-wrap">
                                    {message.content}
                                </div>
                            }
                        >
                            <MessageContent content={message.content} />
                        </Suspense>
                    )}
                    {message.status === "streaming" && (
                        <span className="mt-1 inline-block h-4 w-[7px] rounded-[1px] bg-foreground/70 align-text-bottom animate-pulse" />
                    )}
                    {message.status === "error" && (
                        <p className="mt-2 text-xs text-destructive">
                            {message.error}
                        </p>
                    )}
                    {message.status === "aborted" && (
                        <p className="mt-2 text-xs text-muted-foreground">
                            {t("chat.abortedNotice")}
                        </p>
                    )}
                    {source && (
                        <p className="mt-3 text-xs text-muted-foreground">
                            {source}
                        </p>
                    )}
                    <div className="mt-2 flex items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
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
                    </div>
                </div>
            </div>
        </div>
    );
}
