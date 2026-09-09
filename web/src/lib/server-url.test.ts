import { beforeEach, describe, expect, it, vi } from "vitest";

import { resolveBaseUrl } from "@/lib/server-url";

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
