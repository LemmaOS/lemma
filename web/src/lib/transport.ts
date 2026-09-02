import type { Interceptor } from "@connectrpc/connect";
import { Code, ConnectError, createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";

import { AuthService } from "@/gen/lemma/v1/auth_pb";
import {
    clearTokens,
    getAccessToken,
    getRefreshToken,
    setTokens,
} from "./session";

// The refresh call bypasses the interceptor below, or a 401 from refresh
// itself would trigger another refresh.
const bareTransport = createConnectTransport({ baseUrl: "/" });

// Any failure (missing token, network, server rejection) reads as false;
// the caller then drops the session.
async function tryRefresh(): Promise<boolean> {
    const refreshToken = getRefreshToken();
    if (!refreshToken) return false;
    const auth = createClient(AuthService, bareTransport);
    try {
        const res = await auth.refresh({ refreshToken });
        if (!res.tokens) return false;
        setTokens(res.tokens.accessToken, res.tokens.refreshToken);
        return true;
    } catch {
        return false;
    }
}

const authInterceptor: Interceptor = (next) => async (req) => {
    const token = getAccessToken();
    if (token) req.header.set("Authorization", `Bearer ${token}`);
    try {
        return await next(req);
    } catch (e) {
        // A 401 from AuthService itself (e.g. a wrong password at login)
        // must not trigger a refresh.
        if (
            !(e instanceof ConnectError) ||
            e.code !== Code.Unauthenticated ||
            req.url.includes("AuthService/")
        ) {
            throw e;
        }
        if (!(await tryRefresh())) {
            clearTokens();
            window.location.href = "/login";
            throw e;
        }
        const fresh = getAccessToken();
        if (fresh) req.header.set("Authorization", `Bearer ${fresh}`);
        return await next(req);
    }
};

export const transport = createConnectTransport({
    baseUrl: "/",
    interceptors: [authInterceptor],
});
