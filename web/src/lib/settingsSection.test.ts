import { expect, it } from "vitest";

import { parseSettingsSection, SETTINGS_SECTIONS } from "./settingsSection";

it("合法标签名原样解析", () => {
    for (const section of SETTINGS_SECTIONS) {
        expect(parseSettingsSection(section)).toBe(section);
    }
});

it("非法标签名返回 null", () => {
    expect(parseSettingsSection("general")).toBeNull();
    expect(parseSettingsSection("")).toBeNull();
    expect(parseSettingsSection(undefined)).toBeNull();
});
