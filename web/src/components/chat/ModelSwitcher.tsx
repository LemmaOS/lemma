import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useProviders } from "@/hooks/useProviders";
import { cn } from "@/lib/utils";

export interface ModelSelection {
    providerId: string;
    model: string;
}

interface ModelSwitcherProps {
    selection: ModelSelection | null;
    onSelect: (selection: ModelSelection) => void;
}

/** 输入区模型选择：列出所有已启用供应商的模型 */
export function ModelSwitcher({ selection, onSelect }: ModelSwitcherProps) {
    const { t } = useTranslation();
    const providers = useProviders();
    const enabled = providers.list.filter(
        (p) => p.enabled && p.models.length > 0,
    );

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    className="h-7 px-2 text-xs text-muted-foreground"
                    aria-label={t("chat.selectModel")}
                    title={t("chat.selectModel")}
                    disabled={enabled.length === 0}
                >
                    <span className="font-mono truncate max-w-56">
                        {selection ? selection.model : t("chat.noProvider")}
                    </span>
                    <ChevronDown className="size-3" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
                {enabled.flatMap((p) =>
                    p.models.map((m) => (
                        <DropdownMenuItem
                            key={`${p.id}/${m}`}
                            onClick={() =>
                                onSelect({ providerId: p.id, model: m })
                            }
                            className={cn(
                                "font-mono text-xs",
                                selection?.providerId === p.id &&
                                    selection.model === m &&
                                    "bg-accent",
                            )}
                        >
                            {p.name} · {m}
                        </DropdownMenuItem>
                    )),
                )}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
