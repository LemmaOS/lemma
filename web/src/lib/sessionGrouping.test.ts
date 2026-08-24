import { describe, expect, it } from "vitest";

import { groupSessions, type SessionSummary } from "./sessionGrouping";

const DAY_MS = 24 * 60 * 60 * 1000;

const mk = (id: string, updatedAtMs: number): SessionSummary => ({
    id,
    title: id,
    updatedAtMs,
    messageCount: 0,
});

describe("groupSessions", () => {
    it("按自然日分组且空组不出现", () => {
        const now = Date.now();
        const groups = groupSessions([
            mk("a", now - 60_000),
            mk("b", now - DAY_MS),
            mk("c", now - 3 * DAY_MS),
            mk("d", now - 30 * DAY_MS),
        ]);
        expect(groups.map((g) => g.key)).toEqual([
            "today",
            "yesterday",
            "last7Days",
            "earlier",
        ]);
        expect(groups[0].items.map((s) => s.id)).toEqual(["a"]);
    });

    it("组内按更新时间倒序", () => {
        const now = Date.now();
        const groups = groupSessions([
            mk("old", now - 2 * 60_000),
            mk("new", now - 60_000),
        ]);
        expect(groups[0].items.map((s) => s.id)).toEqual(["new", "old"]);
    });
});
