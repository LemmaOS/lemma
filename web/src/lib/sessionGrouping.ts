export interface SessionSummary {
    id: string;
    title: string;
    updatedAtMs: number;
    messageCount: number;
}

export type GroupKey = "today" | "yesterday" | "last7Days" | "earlier";

const GROUP_ORDER: GroupKey[] = ["today", "yesterday", "last7Days", "earlier"];
const DAY_MS = 24 * 60 * 60 * 1000;

function startOfDayMs(ms: number): number {
    const d = new Date(ms);
    return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

/** 按更新时间倒序分组：今天/昨天/近 7 天/更早；空组不出现 */
export function groupSessions(
    sessions: SessionSummary[],
): { key: GroupKey; items: SessionSummary[] }[] {
    const byGroup = new Map<GroupKey, SessionSummary[]>();
    const today = startOfDayMs(Date.now());
    const sorted = [...sessions].sort((a, b) => b.updatedAtMs - a.updatedAtMs);
    for (const s of sorted) {
        const diff = Math.round((today - startOfDayMs(s.updatedAtMs)) / DAY_MS);
        const key: GroupKey =
            diff <= 0
                ? "today"
                : diff === 1
                  ? "yesterday"
                  : diff <= 6
                    ? "last7Days"
                    : "earlier";
        const items = byGroup.get(key) ?? [];
        items.push(s);
        byGroup.set(key, items);
    }
    return GROUP_ORDER.filter((key) => byGroup.has(key)).map((key) => ({
        key,
        items: byGroup.get(key)!,
    }));
}
