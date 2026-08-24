// token 持久化：纯模块，拦截器与 auth store 共用
const ACCESS_KEY = "lemma.access_token";
const REFRESH_KEY = "lemma.refresh_token";

export function getAccessToken(): string | null {
    return localStorage.getItem(ACCESS_KEY);
}

export function getRefreshToken(): string | null {
    return localStorage.getItem(REFRESH_KEY);
}

export function setTokens(accessToken: string, refreshToken: string): void {
    localStorage.setItem(ACCESS_KEY, accessToken);
    localStorage.setItem(REFRESH_KEY, refreshToken);
}

export function clearTokens(): void {
    localStorage.removeItem(ACCESS_KEY);
    localStorage.removeItem(REFRESH_KEY);
}

// token 载荷里的用户 id（仅做身份变化检测，不验签）
function userIdOf(token: string): string | null {
    try {
        // JWT 载荷是 base64url 编码，先转成标准 base64 再解码
        const raw = token.split(".")[1] ?? "";
        const b64 = raw.replace(/-/g, "+").replace(/_/g, "/");
        const payload = JSON.parse(
            atob(b64 + "=".repeat((4 - (b64.length % 4)) % 4)),
        ) as { sub?: string };
        return payload.sub ?? null;
    } catch {
        return null;
    }
}

// 多标签页共用 localStorage：别的标签页换了账号时，本页强制重载，避免拿错 token 写脏缓存
export function installCrossTabGuard(): void {
    window.addEventListener("storage", (event) => {
        if (event.key !== ACCESS_KEY) return;
        const prev = event.oldValue ? userIdOf(event.oldValue) : null;
        const next = event.newValue ? userIdOf(event.newValue) : null;
        if (prev && next && prev !== next) window.location.reload();
    });
}
