import { ArrowUp, ChevronDown, Square } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Textarea } from "@/components/ui/textarea";

export interface ModelOption {
    key: string; // `${providerId}:${model}`
    providerId: string;
    model: string;
    label: string; // "ProviderName · model"
}

interface ChatComposerProps {
    value: string;
    onChange: (value: string) => void;
    onSend: () => void;
    onStop: () => void;
    streaming: boolean;
    options: ModelOption[];
    selectedKey: string | null;
    onSelect: (key: string) => void;
    inputRef?: React.RefObject<HTMLTextAreaElement | null>;
}

function autosize(el: HTMLTextAreaElement | null) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
}

/** 底部输入区（设计稿 §4.2）：卡片容器、自动增高、模型选择、发送/停止 */
export function ChatComposer({
    value,
    onChange,
    onSend,
    onStop,
    streaming,
    options,
    selectedKey,
    onSelect,
    inputRef,
}: ChatComposerProps) {
    const { t } = useTranslation();
    const innerRef = useRef<HTMLTextAreaElement>(null);
    const textareaRef = inputRef ?? innerRef;

    useEffect(() => {
        autosize(textareaRef.current);
    }, [value, textareaRef]);

    const selected = options.find((o) => o.key === selectedKey) ?? null;
    const canSend = value.trim().length > 0 && selected !== null;

    return (
        <div className="px-6 pb-6">
            <div className="mx-auto max-w-3xl rounded-xl border border-input bg-card shadow-xs">
                <Textarea
                    ref={textareaRef}
                    value={value}
                    rows={1}
                    onChange={(e) => onChange(e.target.value)}
                    onInput={(e) => autosize(e.currentTarget)}
                    onKeyDown={(e) => {
                        if (
                            e.key === "Enter" &&
                            !e.shiftKey &&
                            !e.nativeEvent.isComposing
                        ) {
                            e.preventDefault();
                            if (canSend && !streaming) onSend();
                        }
                    }}
                    placeholder={t("chat.inputPlaceholder")}
                    aria-label={t("chat.inputPlaceholder")}
                    className="min-h-[52px] max-h-40 resize-none border-0 bg-transparent px-4 pt-3.5 shadow-none focus-visible:ring-0"
                />
                <div className="flex items-center justify-between px-3 pb-2.5">
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button
                                variant="ghost"
                                size="sm"
                                className="h-7 gap-1 px-2 text-xs text-muted-foreground"
                                aria-label={t("chat.chooseModel")}
                            >
                                {selected?.model ?? t("chat.noProvider")}
                                <ChevronDown className="size-3.5" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="start">
                            {options.map((o) => (
                                <DropdownMenuItem
                                    key={o.key}
                                    onSelect={() => onSelect(o.key)}
                                >
                                    {o.label}
                                </DropdownMenuItem>
                            ))}
                            {options.length === 0 && (
                                <DropdownMenuItem asChild>
                                    <Link to="/settings/providers">
                                        {t("chat.noProvider")}
                                    </Link>
                                </DropdownMenuItem>
                            )}
                        </DropdownMenuContent>
                    </DropdownMenu>
                    {streaming ? (
                        <Button
                            variant="outline"
                            size="icon"
                            className="size-8 rounded-md"
                            onClick={onStop}
                            aria-label={t("chat.stop")}
                        >
                            <Square className="size-3.5" />
                        </Button>
                    ) : (
                        <Button
                            size="icon"
                            className="size-8 rounded-md bg-primary text-primary-foreground"
                            onClick={onSend}
                            disabled={!canSend}
                            aria-label={t("chat.send")}
                        >
                            <ArrowUp className="size-4" />
                        </Button>
                    )}
                </div>
            </div>
            <p className="mx-auto max-w-3xl pt-2 text-center text-xs text-muted-foreground">
                {t("chat.disclaimer")}
            </p>
        </div>
    );
}
