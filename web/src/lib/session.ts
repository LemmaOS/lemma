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
