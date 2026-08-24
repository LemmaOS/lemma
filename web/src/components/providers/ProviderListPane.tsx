import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, Plus, Search } from "lucide-react";
import type { Provider } from "@/mocks";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible";

interface ProviderListPaneProps {
    providers: Provider[];
    selectedId: string | null;
    onSelect: (id: string) => void;
    onCreate: () => void;
}

function ProviderGroup({
    label,
    providers,
    selectedId,
    onSelect,
}: {
    label: string;
    providers: Provider[];
    selectedId: string | null;
    onSelect: (id: string) => void;
}) {
    const [open, setOpen] = useState(true);
    if (providers.length === 0) return null;
    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex w-full items-center gap-1 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent/60">
                <ChevronDown
                    className={cn(
                        "size-3.5 transition-transform",
                        !open && "-rotate-90",
                    )}
                />
                {label}
                <span className="ml-auto text-[11px]">{providers.length}</span>
            </CollapsibleTrigger>
            <CollapsibleContent className="flex flex-col gap-0.5">
                {providers.map((provider) => (
                    <button
                        key={provider.id}
                        type="button"
                        onClick={() => onSelect(provider.id)}
                        aria-pressed={provider.id === selectedId}
                        className={cn(
                            "flex w-full items-center gap-2.5 rounded-md px-2 py-2 text-left text-sm transition-colors",
                            provider.id === selectedId
                                ? "bg-accent"
                                : "hover:bg-accent/60",
                        )}
                    >
                        <span className="grid size-7 shrink-0 place-items-center rounded-md border border-border bg-background text-xs font-semibold">
                            {(provider.name || "?").charAt(0).toUpperCase()}
                        </span>
                        <span className="flex-1 truncate">{provider.name}</span>
                        <span
                            className={cn(
                                "size-1.5 shrink-0 rounded-full",
                                provider.enabled
                                    ? "bg-primary"
                                    : "bg-muted-foreground/40",
                            )}
                        />
                    </button>
                ))}
            </CollapsibleContent>
        </Collapsible>
    );
}

export function ProviderListPane({
    providers,
    selectedId,
    onSelect,
    onCreate,
}: ProviderListPaneProps) {
    const { t } = useTranslation();
    const [query, setQuery] = useState("");

    const filtered = useMemo(() => {
        const q = query.trim().toLowerCase();
        if (!q) return providers;
        return providers.filter((p) => p.name.toLowerCase().includes(q));
    }, [providers, query]);

    const enabledProviders = filtered.filter((p) => p.enabled);
    const disabledProviders = filtered.filter((p) => !p.enabled);

    return (
        <section className="flex w-[260px] shrink-0 flex-col rounded-xl border border-border bg-background">
            <div className="flex items-center gap-2 p-3">
                <div className="relative flex-1">
                    <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder={t("providers.searchPlaceholder")}
                        className="pl-8"
                    />
                </div>
                <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    aria-label={t("providers.newProvider")}
                    onClick={onCreate}
                >
                    <Plus className="size-4" />
                </Button>
            </div>
            <div className="flex-1 overflow-y-auto px-2 pb-3">
                <ProviderGroup
                    label={t("providers.enabledGroup")}
                    providers={enabledProviders}
                    selectedId={selectedId}
                    onSelect={onSelect}
                />
                <ProviderGroup
                    label={t("providers.disabledGroup")}
                    providers={disabledProviders}
                    selectedId={selectedId}
                    onSelect={onSelect}
                />
            </div>
        </section>
    );
}
