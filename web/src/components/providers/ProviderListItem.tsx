import { useTranslation } from "react-i18next";

import type { Provider } from "@/gen/lemma/v1/provider_pb";
import { kindLabel } from "@/lib/providerKind";
import { cn } from "@/lib/utils";

interface ProviderListItemProps {
    provider: Provider;
    selected: boolean;
    onSelect: () => void;
}

export function ProviderListItem({
    provider,
    selected,
    onSelect,
}: ProviderListItemProps) {
    const { t } = useTranslation();
    // 服务端返回的 apiKey 是脱敏串，非空即视为已配置
    const configured = provider.apiKey !== "";

    return (
        <button
            type="button"
            onClick={onSelect}
            aria-current={selected ? "true" : undefined}
            className={cn(
                "flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors",
                selected
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-accent/60",
            )}
        >
            <span className="min-w-0 flex-1">
                <span className="block truncate text-sm font-medium">
                    {provider.name}
                </span>
                <span className="block truncate text-xs text-muted-foreground">
                    {kindLabel(provider.kind)}
                </span>
            </span>
            <span
                role="img"
                aria-label={
                    configured
                        ? t("providers.configured")
                        : t("providers.notConfigured")
                }
                className={cn(
                    "size-1.5 shrink-0 rounded-full",
                    configured ? "bg-primary" : "bg-muted-foreground/40",
                )}
            />
        </button>
    );
}
