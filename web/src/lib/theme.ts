import { useCallback, useEffect, useState } from "react";

export type ThemePreference = "light" | "dark" | "system";

// 与 index.html 首帧脚本共用同一个键
export const THEME_STORAGE_KEY = "lemma.theme";

export function readThemePreference(): ThemePreference {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    return stored === "light" || stored === "dark" || stored === "system"
        ? stored
        : "system";
}

export function resolveTheme(pref: ThemePreference): "light" | "dark" {
    if (pref === "system") {
        return window.matchMedia("(prefers-color-scheme: dark)").matches
            ? "dark"
            : "light";
    }
    return pref;
}

export function applyTheme(pref: ThemePreference) {
    document.documentElement.setAttribute("data-theme", resolveTheme(pref));
}

export function saveThemePreference(pref: ThemePreference) {
    localStorage.setItem(THEME_STORAGE_KEY, pref);
}

/** 三态主题偏好：读取/应用/持久化/跟随系统，全在这一个 hook 里 */
export function useThemePreference(): readonly [
    ThemePreference,
    (pref: ThemePreference) => void,
] {
    const [preference, setPreference] =
        useState<ThemePreference>(readThemePreference);

    useEffect(() => {
        applyTheme(preference);
        if (preference !== "system") return;
        const mql = window.matchMedia("(prefers-color-scheme: dark)");
        const onChange = () => applyTheme("system");
        mql.addEventListener("change", onChange);
        return () => mql.removeEventListener("change", onChange);
    }, [preference]);

    const select = useCallback((pref: ThemePreference) => {
        saveThemePreference(pref);
        setPreference(pref);
    }, []);

    return [preference, select] as const;
}
