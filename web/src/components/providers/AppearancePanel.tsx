import { Monitor, Moon, Sun } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useThemePreference } from "@/lib/theme";

export function AppearancePanel() {
    const { t } = useTranslation();
    const [preference, select] = useThemePreference();

    return (
        <div className="max-w-3xl px-8 py-6">
            <h2 className="text-base font-semibold">
                {t("settings.appearance")}
            </h2>
            <div className="mt-2 flex items-center justify-between gap-6 py-4">
                <div className="min-w-0">
                    <p className="text-sm font-medium">
                        {t("settings.themeLabel")}
                    </p>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                        {t("settings.themeDesc")}
                    </p>
                </div>
                <ToggleGroup
                    type="single"
                    variant="outline"
                    value={preference}
                    onValueChange={(value) => {
                        if (
                            value === "light" ||
                            value === "dark" ||
                            value === "system"
                        ) {
                            select(value);
                        }
                    }}
                    aria-label={t("settings.themeLabel")}
                >
                    <ToggleGroupItem
                        value="light"
                        aria-label={t("theme.light")}
                    >
                        <Sun className="size-4" />
                        {t("theme.light")}
                    </ToggleGroupItem>
                    <ToggleGroupItem value="dark" aria-label={t("theme.dark")}>
                        <Moon className="size-4" />
                        {t("theme.dark")}
                    </ToggleGroupItem>
                    <ToggleGroupItem
                        value="system"
                        aria-label={t("theme.system")}
                    >
                        <Monitor className="size-4" />
                        {t("theme.system")}
                    </ToggleGroupItem>
                </ToggleGroup>
            </div>
        </div>
    );
}
