import { useTranslation } from "react-i18next";
import { Navigate } from "react-router";

import { AuthCard } from "@/components/auth/AuthCard";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useAuth } from "@/stores/auth";

export default function LoginPage() {
    const { t } = useTranslation();
    const user = useAuth((s) => s.user);
    const ready = useAuth((s) => s.ready);

    // 已登录则没必要再看登录页
    if (ready && user) return <Navigate to="/" replace />;

    return (
        <div className="relative grid min-h-dvh place-items-center bg-background px-4">
            <div className="absolute right-4 top-4">
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
