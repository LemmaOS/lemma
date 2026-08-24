import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";
import { mockProviders } from "@/mocks";
import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface ModelSwitcherProps {
    /** Currently selected model id. */
    model: string;
    /** Called when the user picks a different model. */
    onModelChange: (model: string) => void;
}

/** Composer tool-row model picker: lists models of all configured providers. */
export function ModelSwitcher({ model, onModelChange }: ModelSwitcherProps) {
    const { t } = useTranslation();
    const models = mockProviders
        .filter((p) => p.configured)
        .flatMap((p) => p.models);

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    className="h-7 px-2 text-xs text-muted-foreground"
                    aria-label={t("chat.selectModel")}
                    title={t("chat.selectModel")}
                >
                    <span className="font-mono">{model}</span>
                    <ChevronDown className="size-3" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
                {models.map((m) => (
                    <DropdownMenuItem
                        key={m}
                        onClick={() => onModelChange(m)}
                        className="font-mono text-xs"
                    >
                        {m}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
