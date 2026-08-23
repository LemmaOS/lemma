import { Monitor, Moon, Sun } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
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

type ThemePreference = "light" | "dark" | "system";

// 与 index.html 首帧脚本共用同一个键
const STORAGE_KEY = "lemma.theme";

function resolveTheme(pref: ThemePreference): "light" | "dark" {
    if (pref === "system") {
        return window.matchMedia("(prefers-color-scheme: dark)").matches
            ? "dark"
            : "light";
    }
    return pref;
}

function applyTheme(pref: ThemePreference) {
    document.documentElement.setAttribute("data-theme", resolveTheme(pref));
}

export function ThemeToggle() {
    const { t } = useTranslation();
    const [preference, setPreference] = useState<ThemePreference>(() => {
        const stored = localStorage.getItem(STORAGE_KEY);
        return stored === "light" || stored === "dark" || stored === "system"
            ? stored
            : "system";
    });

    useEffect(() => {
        applyTheme(preference);
        if (preference !== "system") return;
        const mql = window.matchMedia("(prefers-color-scheme: dark)");
        const onChange = () => applyTheme("system");
        mql.addEventListener("change", onChange);
        return () => mql.removeEventListener("change", onChange);
    }, [preference]);

    const select = useCallback((pref: ThemePreference) => {
        localStorage.setItem(STORAGE_KEY, pref);
        setPreference(pref);
    }, []);

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
