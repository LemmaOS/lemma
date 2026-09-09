import { app } from "electron";
import fs from "node:fs";
import path from "node:path";

interface Settings {
    serverUrl?: string;
}

const settingsPath = () => path.join(app.getPath("userData"), "settings.json");

export function getServerUrl(): string | undefined {
    try {
        const settings = JSON.parse(
            fs.readFileSync(settingsPath(), "utf8"),
        ) as Settings;
        return settings.serverUrl || undefined;
    } catch {
        return undefined;
    }
}

export function setServerUrl(serverUrl: string): void {
    const settings: Settings = { serverUrl };
    fs.writeFileSync(settingsPath(), JSON.stringify(settings, null, 2));
}
