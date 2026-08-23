import { lazy, Suspense, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BrowserRouter, Navigate, Outlet, Route, Routes } from "react-router";

import { TooltipProvider } from "@/components/ui/tooltip";
import { useAuth } from "@/stores/auth";

// 三个页面各自拆成独立 chunk，访问对应路由时才下载
const ChatPage = lazy(() => import("@/pages/ChatPage"));
const LoginPage = lazy(() => import("@/pages/LoginPage"));
const ProvidersPage = lazy(() => import("@/pages/ProvidersPage"));

function Loading() {
    const { t } = useTranslation();
    return (
        <div className="grid min-h-dvh place-items-center bg-background text-sm text-muted-foreground">
            {t("common.loading")}
        </div>
    );
}

// 等待会话恢复完成再决定放行还是跳登录
function RequireAuth() {
    const user = useAuth((s) => s.user);
    const ready = useAuth((s) => s.ready);

    if (!ready) return <Loading />;
    if (!user) return <Navigate to="/login" replace />;
    return <Outlet />;
}

export default function App() {
    const bootstrap = useAuth((s) => s.bootstrap);

    useEffect(() => {
        void bootstrap();
    }, [bootstrap]);

    return (
        <TooltipProvider>
            <BrowserRouter>
                <Suspense fallback={<Loading />}>
                    <Routes>
                        <Route path="/login" element={<LoginPage />} />
                        <Route element={<RequireAuth />}>
                            <Route path="/" element={<ChatPage />} />
                            <Route
                                path="/settings/providers"
                                element={<ProvidersPage />}
                            />
                        </Route>
                        <Route path="*" element={<Navigate to="/" replace />} />
                    </Routes>
                </Suspense>
            </BrowserRouter>
        </TooltipProvider>
    );
}
