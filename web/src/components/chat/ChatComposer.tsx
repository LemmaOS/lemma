import { ArrowUp, Square } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import {
    ModelSwitcher,
    type ModelSelection,
} from "@/components/chat/ModelSwitcher";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

interface ChatComposerProps {
    value: string;
    onChange: (value: string) => void;
    onSend: () => void;
    onStop: () => void;
    streaming: boolean;
    model: ModelSelection | null;
    onModelChange: (selection: ModelSelection) => void;
    inputRef?: React.RefObject<HTMLTextAreaElement | null>;
}

function autosize(el: HTMLTextAreaElement | null) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
}

export function ChatComposer({
    value,
    onChange,
    onSend,
    onStop,
    streaming,
    model,
    onModelChange,
    inputRef,
}: ChatComposerProps) {
    const { t } = useTranslation();
    const innerRef = useRef<HTMLTextAreaElement>(null);
    const textareaRef = inputRef ?? innerRef;

    useEffect(() => {
        autosize(textareaRef.current);
    }, [value, textareaRef]);

    const canSend = value.trim().length > 0;

    return (
        <div className="px-6 pb-6">
            <div className="max-w-3xl mx-auto rounded-xl border border-input bg-composer">
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
                    className="min-h-[52px] max-h-40 resize-none border-0 bg-transparent focus-visible:ring-0 px-4 pt-3.5 shadow-none"
                />
                <div className="flex items-center justify-between px-3 pb-2.5">
                    <ModelSwitcher selection={model} onSelect={onModelChange} />
                    <div className="flex items-center gap-1">
                        {streaming ? (
                            <Button
                                variant="outline"
                                size="icon"
                                className="size-8 rounded-full"
                                onClick={onStop}
                                aria-label={t("chat.stop")}
                            >
                                <Square className="size-3.5" />
                            </Button>
                        ) : (
                            <Button
                                size="icon"
                                className="size-8 rounded-full bg-foreground text-background hover:bg-foreground/85"
                                onClick={onSend}
                                disabled={!canSend}
                                aria-label={t("chat.send")}
                            >
                                <ArrowUp className="size-4" />
                            </Button>
                        )}
                    </div>
                </div>
            </div>
            <p className="max-w-3xl mx-auto text-center text-xs text-muted-foreground pt-2">
                {model ? model.model : t("chat.noProvider")} ·{" "}
                {t("chat.disclaimer")}
            </p>
        </div>
    );
}
