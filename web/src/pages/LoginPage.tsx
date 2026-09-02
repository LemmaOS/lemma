import { useTranslation } from "react-i18next";
import { Navigate } from "react-router";

import { AuthCard } from "@/components/auth/AuthCard";
import { LanguageToggle } from "@/components/LanguageToggle";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useAuth } from "@/stores/auth";

export default function LoginPage() {
    const { t } = useTranslation();
    const user = useAuth((s) => s.user);

    if (user) return <Navigate to="/" replace />;

    return (
        <div className="relative min-h-dvh grid place-items-center bg-background px-4">
            <div className="absolute right-4 top-4 flex items-center gap-1">
                <LanguageToggle />
                <ThemeToggle />
            </div>
            <div className="flex w-full max-w-[380px] flex-col items-center gap-4">
                <AuthCard />
                <p className="text-center text-xs text-muted-foreground">
                    {t("auth.selfHostedNote")}
                </p>
            </div>
        </div>
    );
}
