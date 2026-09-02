import { Languages } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";

export function LanguageToggle() {
    const { i18n, t } = useTranslation();
    return (
        <DropdownMenu>
            <Tooltip>
                <TooltipTrigger asChild>
                    <DropdownMenuTrigger asChild>
                        <Button
                            variant="ghost"
                            size="icon"
                            aria-label={t("language.toggle")}
                        >
                            <Languages className="size-4" />
                        </Button>
                    </DropdownMenuTrigger>
                </TooltipTrigger>
                <TooltipContent>{t("language.toggle")}</TooltipContent>
            </Tooltip>
            <DropdownMenuContent align="end">
                <DropdownMenuItem
                    onSelect={() => void i18n.changeLanguage("zh")}
                >
                    中文
                </DropdownMenuItem>
                <DropdownMenuItem
                    onSelect={() => void i18n.changeLanguage("en")}
                >
                    English
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
