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

// 裸 transport 仅用于刷新 token，避免拦截器自递归
const bareTransport = createConnectTransport({ baseUrl: "/" });

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

// Bearer 注入；401 → 刷新并重放一次；刷新失败清空登录态回登录页
const authInterceptor: Interceptor = (next) => async (req) => {
    const token = getAccessToken();
    if (token) req.header.set("Authorization", `Bearer ${token}`);
    try {
        return await next(req);
    } catch (e) {
        // 认证服务自身的 401（如登录密码错）不触发刷新
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
