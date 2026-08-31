import { MessageSquare } from "lucide-react";
import { useTranslation } from "react-i18next";

const SUGGESTION_KEYS = [
    "chat.suggestion1",
    "chat.suggestion2",
    "chat.suggestion3",
] as const;

interface EmptyStateProps {
    /** Fill the composer with the picked suggestion text. */
    onPickSuggestion: (text: string) => void;
}

/** Empty conversation onboarding (design.md §3.4). */
export function EmptyState({ onPickSuggestion }: EmptyStateProps) {
    const { t } = useTranslation();

    return (
        <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
            <MessageSquare
                className="size-8 text-muted-foreground/50"
                strokeWidth={1.5}
            />
            <h1 className="text-card-title font-medium">{t("chat.emptyTitle")}</h1>
            <p className="text-sm text-muted-foreground">
                {t("chat.emptySubtitle")}
            </p>
            <div className="mt-4 grid w-full max-w-2xl gap-3 sm:grid-cols-3">
                {SUGGESTION_KEYS.map((key) => (
                    <button
                        key={key}
                        type="button"
                        onClick={() => onPickSuggestion(t(key))}
                        className="rounded-md border p-3 text-sm text-left hover:bg-accent cursor-pointer transition-colors"
                    >
                        {t(key)}
                    </button>
                ))}
            </div>
        </div>
    );
}
