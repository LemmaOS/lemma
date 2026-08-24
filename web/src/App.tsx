import { lazy, Suspense, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BrowserRouter, Navigate, Outlet, Route, Routes } from "react-router";

import { TooltipProvider } from "@/components/ui/tooltip";
import { useAuth } from "@/stores/auth";
import { useChat } from "@/stores/chat";
import { closeDb, openDb } from "./lib/db";
import { onSynced, startSync, stopSync } from "./lib/sync";
import { useConversationsStore } from "./stores/conversations";

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

// 等会话恢复完成再决定放行还是跳登录
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

    const userId = useAuth((s) => s.user?.id ?? null);

    // 登录后：开用户专属缓存 → 缓存铺数据 → 启动同步引擎；登出/切号时全部关闭（缓存按用户保留）
    useEffect(() => {
        if (!userId) return;
        openDb(userId);
        void useConversationsStore.getState().hydrateFromCache();
        const off = onSynced(() => {
            void useConversationsStore.getState().hydrateFromCache();
            void useChat.getState().syncFromCache();
        });
        startSync();
        return () => {
            off();
            stopSync();
            closeDb();
        };
    }, [userId]);

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
