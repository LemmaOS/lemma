import { ArrowUp } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import {
    ModelSwitcher,
    type ModelSelection,
} from "@/components/chat/ModelSwitcher";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

interface HomeViewProps {
    onSubmit: (text: string) => void;
    model: ModelSelection | null;
    onModelChange: (selection: ModelSelection) => void;
}

function autosize(el: HTMLTextAreaElement | null) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
}

/** 主区首页：居中的新会话输入卡 */
export function HomeView({ onSubmit, model, onModelChange }: HomeViewProps) {
    const { t } = useTranslation();
    const [value, setValue] = useState("");
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    useEffect(() => {
        autosize(textareaRef.current);
    }, [value]);

    const canSend = value.trim().length > 0;
    const submit = () => {
        if (canSend) onSubmit(value.trim());
    };

    return (
        <div className="flex-1 grid place-items-center px-6">
            <div className="w-full max-w-2xl">
                <p className="text-center text-2xl font-semibold tracking-tight mb-8">
                    {t("common.appName")}
                </p>
                <div className="rounded-xl border border-input bg-composer p-4 shadow-xs">
                    <Textarea
                        ref={textareaRef}
                        value={value}
                        rows={2}
                        onChange={(e) => setValue(e.target.value)}
                        onInput={(e) => autosize(e.currentTarget)}
                        onKeyDown={(e) => {
                            if (
                                e.key === "Enter" &&
                                !e.shiftKey &&
                                !e.nativeEvent.isComposing
                            ) {
                                e.preventDefault();
                                submit();
                            }
                        }}
                        placeholder={t("chat.inputPlaceholder")}
                        aria-label={t("chat.inputPlaceholder")}
                        className="min-h-[64px] max-h-48 resize-none border-0 bg-transparent shadow-none focus-visible:ring-0 px-1"
                    />
                    <div className="mt-2 flex items-center justify-between">
                        <ModelSwitcher
                            selection={model}
                            onSelect={onModelChange}
                        />
                        <Button
                            size="icon"
                            className="size-8 rounded-full bg-foreground text-background hover:bg-foreground/85"
                            onClick={submit}
                            disabled={!canSend}
                            aria-label={t("chat.send")}
                        >
                            <ArrowUp className="size-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}
