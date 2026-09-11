declare global {
    interface Window {
        __LEMMA_SERVER_URL__?: string;
        lemmaDesktop?: {
            getServerUrl(): Promise<string | undefined>;
            setServerUrl(url: string): Promise<void>;
            toggleMaximize(): void;
        };
    }
}

export function isDesktop(): boolean {
    return typeof window !== "undefined" && window.lemmaDesktop !== undefined;
}

export function resolveBaseUrl(): string {
    const injected =
        typeof window !== "undefined" ? window.__LEMMA_SERVER_URL__ : undefined;
    const stored =
        typeof localStorage !== "undefined"
            ? localStorage.getItem("lemma.serverUrl")
            : null;
    return injected || stored || "/";
}

export function appPath(path: string): string {
    return window.location.protocol === "file:" ? `#${path}` : path;
}
