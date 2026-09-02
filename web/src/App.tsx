import { lazy, Suspense, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BrowserRouter, Navigate, Outlet, Route, Routes } from "react-router";

import { TooltipProvider } from "@/components/ui/tooltip";
import { useAuth } from "@/stores/auth";
import { useChat } from "@/stores/chat";
import { closeDb, openDb } from "./lib/db";
import { onSynced, startSync, stopSync } from "./lib/sync";
import { useConversationsStore } from "./stores/conversations";

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

    useEffect(() => {
        if (!userId) return;
        // Wire the sync stack to the signed-in user: open their per-user
        // cache, render it, re-render after every pull, and keep the watch
        // loop alive until logout or an account switch tears it down.
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
