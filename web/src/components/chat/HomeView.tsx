import { useEffect, useRef, useState } from "react";
import { ArrowUp } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ModelSwitcher } from "@/components/chat/ModelSwitcher";

interface HomeViewProps {
    /** Create a new session from the home input and enter the chat view. */
    onSubmit: (text: string) => void;
    /** Currently selected model id. */
    model: string;
    /** Called when the user picks a different model. */
    onModelChange: (model: string) => void;
}

function autosize(el: HTMLTextAreaElement | null) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
}

/** Home view: a single input card centered in the main area. */
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
                {/* Brand */}
                <p className="text-center text-2xl font-semibold tracking-tight mb-8">
                    {t("common.appName")}
                </p>
                {/* Large input card */}
                <div className="rounded-2xl border border-input bg-card p-4 shadow-xs">
                    <Textarea
                        ref={textareaRef}
                        value={value}
                        rows={2}
                        onChange={(e) => setValue(e.target.value)}
                        onInput={(e) => autosize(e.currentTarget)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter" && !e.shiftKey) {
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
                            model={model}
                            onModelChange={onModelChange}
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
