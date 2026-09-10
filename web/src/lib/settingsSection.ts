export const SETTINGS_SECTIONS = [
    "appearance",
    "providers",
    "storage",
] as const;

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

export function parseSettingsSection(
    raw: string | undefined,
): SettingsSection | null {
    return (SETTINGS_SECTIONS as readonly string[]).includes(raw ?? "")
        ? (raw as SettingsSection)
        : null;
}
