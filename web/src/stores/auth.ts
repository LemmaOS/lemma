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
        try {
            const refreshToken = getRefreshToken();
            if (refreshToken) await authClient.logout({ refreshToken });
        } catch {
            // Intentionally empty.
        }
        clearTokens();
        set({ user: null });
    },
}));
