import { Link } from "react-router";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Bot, Palette } from "lucide-react";
import { cn } from "@/lib/utils";

export type SettingsSection = "appearance" | "providers";

interface SettingsNavProps {
    section: SettingsSection;
    onSelect: (section: SettingsSection) => void;
}

interface NavItemProps {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    selected: boolean;
    onClick: () => void;
}

function NavItem({ icon: Icon, label, selected, onClick }: NavItemProps) {
    return (
        <button
            type="button"
            onClick={onClick}
            aria-pressed={selected}
            className={cn(
                "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
                selected
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground hover:bg-accent/60",
            )}
        >
            <Icon className="size-4 shrink-0" />
            <span className="truncate">{label}</span>
        </button>
    );
}

function NavGroup({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div>
            <p className="px-2 pt-4 pb-1 text-xs text-muted-foreground">
                {label}
            </p>
            <div className="flex flex-col gap-0.5">{children}</div>
        </div>
    );
}

export function SettingsNav({ section, onSelect }: SettingsNavProps) {
    const { t } = useTranslation();
    return (
        <aside className="flex w-[220px] shrink-0 flex-col bg-transparent p-3">
            <div className="flex items-center gap-2 px-1 pb-2">
                <Link
                    to="/"
                    aria-label={t("settings.backToHome")}
                    className="grid size-7 place-items-center rounded-md text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground"
                >
                    <ArrowLeft className="size-4" />
                </Link>
                <h1 className="text-sm font-semibold text-sidebar-foreground">
                    {t("settings.title")}
                </h1>
            </div>
            <nav className="flex flex-col gap-0.5">
                <NavGroup label={t("settings.groupGeneral")}>
                    <NavItem
                        icon={Palette}
                        label={t("settings.appearance")}
                        selected={section === "appearance"}
                        onClick={() => onSelect("appearance")}
                    />
                </NavGroup>
                <NavGroup label={t("settings.groupAgent")}>
                    <NavItem
                        icon={Bot}
                        label={t("settings.aiProviders")}
                        selected={section === "providers"}
                        onClick={() => onSelect("providers")}
                    />
                </NavGroup>
            </nav>
        </aside>
    );
}
