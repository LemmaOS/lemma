import { Monitor, Moon, Sun } from "lucide-react";
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
import { useThemePreference } from "@/lib/theme";

export function ThemeToggle() {
    const { t } = useTranslation();
    const [preference, select] = useThemePreference();

    const Icon =
        preference === "light" ? Sun : preference === "dark" ? Moon : Monitor;

    return (
        <DropdownMenu>
            <Tooltip>
                <TooltipTrigger asChild>
                    <DropdownMenuTrigger asChild>
                        <Button
                            variant="ghost"
                            size="icon"
                            aria-label={t("theme.toggle")}
                        >
                            <Icon className="size-4" />
                        </Button>
                    </DropdownMenuTrigger>
                </TooltipTrigger>
                <TooltipContent>{t("theme.toggle")}</TooltipContent>
            </Tooltip>
            <DropdownMenuContent align="end">
                <DropdownMenuItem onSelect={() => select("light")}>
                    <Sun className="size-4" />
                    {t("theme.light")}
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={() => select("dark")}>
                    <Moon className="size-4" />
                    {t("theme.dark")}
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={() => select("system")}>
                    <Monitor className="size-4" />
                    {t("theme.system")}
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
