import { create } from "zustand";

import type { User } from "@/gen/lemma/v1/auth_pb";
import { authClient } from "@/lib/clients";
import {
    clearTokens,
    getAccessToken,
    getRefreshToken,
    setTokens,
} from "@/lib/session";

interface AuthState {
    user: User | null;
    // 启动时的会话恢复是否结束（路由守卫据此等待）
    ready: boolean;
    bootstrap: () => Promise<void>;
    login: (identifier: string, password: string) => Promise<void>;
    signup: (
        username: string,
        email: string,
        password: string,
    ) => Promise<void>;
    logout: () => Promise<void>;
}

export const useAuth = create<AuthState>()((set) => ({
    user: null,
    ready: false,

    // 应用启动：有旧 token 就调 me 恢复会话；失败（含 refresh 失败）视为未登录
    bootstrap: async () => {
        if (getAccessToken()) {
            try {
                const res = await authClient.me({});
                set({ user: res.user ?? null });
            } catch {
                clearTokens();
            }
        }
        set({ ready: true });
    },

    // 契约里 username 与 email 二选一：含 @ 视为邮箱
    login: async (identifier, password) => {
        const req = identifier.includes("@")
            ? { email: identifier, password }
            : { username: identifier, password };
        const res = await authClient.login(req);
        if (!res.tokens) throw new Error("no tokens in response");
        setTokens(res.tokens.accessToken, res.tokens.refreshToken);
        set({ user: res.user ?? null });
    },

    signup: async (username, email, password) => {
        const res = await authClient.signUp({ username, email, password });
        if (!res.tokens) throw new Error("no tokens in response");
        setTokens(res.tokens.accessToken, res.tokens.refreshToken);
        set({ user: res.user ?? null });
    },

    logout: async () => {
        // 尽力通知服务端吊销 refresh token，本地登出不依赖它成功
        try {
            const refreshToken = getRefreshToken();
            if (refreshToken) await authClient.logout({ refreshToken });
        } catch {
            // 忽略：本地登出优先
        }
        clearTokens();
        set({ user: null });
    },
}));
