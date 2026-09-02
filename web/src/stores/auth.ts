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
        // A stored token restores the session via me(); any failure means
        // the session is gone and the tokens are dropped.
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
        // The login target is an email when it contains @, else a username.
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
            // Revoking the refresh token is best-effort: local logout must
            // succeed even when the server is unreachable.
        }
        clearTokens();
        set({ user: null });
    },
}));
