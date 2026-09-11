import { beforeEach, describe, expect, it, vi } from "vitest";

import { isDesktop, resolveBaseUrl } from "@/lib/server-url";

describe("resolveBaseUrl", () => {
    beforeEach(() => {
        vi.unstubAllGlobals();
    });

    it("defaults to same-origin root", () => {
        expect(resolveBaseUrl()).toBe("/");
    });

    it("falls back to localStorage when nothing is injected", () => {
        vi.stubGlobal("localStorage", {
            getItem: () => "http://stored.example.com",
        });
        expect(resolveBaseUrl()).toBe("http://stored.example.com");
    });

    it("prefers the injected window value over localStorage", () => {
        vi.stubGlobal("window", {
            __LEMMA_SERVER_URL__: "http://injected.example.com",
        });
        vi.stubGlobal("localStorage", {
            getItem: () => "http://stored.example.com",
        });
        expect(resolveBaseUrl()).toBe("http://injected.example.com");
    });

    it("treats empty values as unset", () => {
        vi.stubGlobal("window", { __LEMMA_SERVER_URL__: "" });
        vi.stubGlobal("localStorage", { getItem: () => "" });
        expect(resolveBaseUrl()).toBe("/");
    });
});

describe("isDesktop", () => {
    beforeEach(() => {
        vi.unstubAllGlobals();
    });

    it("is false in a plain browser", () => {
        expect(isDesktop()).toBe(false);
    });

    it("is true when the desktop preload bridge is present", () => {
        vi.stubGlobal("window", { lemmaDesktop: {} });
        expect(isDesktop()).toBe(true);
    });
});
