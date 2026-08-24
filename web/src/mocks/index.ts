/**
 * Mock data — single source of truth for all demo content (see design.md §5).
 * Content strings here are DATA, not UI copy, so they stay in English and do
 * not go through t(). Components must receive everything via props.
 */

/* ------------------------------- Sessions ------------------------------- */

export interface Session {
    id: string;
    title: string;
    /** ISO 8601 timestamp */
    updatedAt: string;
    messageCount: number;
    archived?: boolean;
}

export const mockSessions: Session[] = [
    {
        id: "s-01",
        title: "Tailwind v4 migration plan",
        updatedAt: "2025-01-20T09:42:00Z",
        messageCount: 12,
    },
    {
        id: "s-02",
        title: "Debug streaming SSE parser",
        updatedAt: "2025-01-20T08:15:00Z",
        messageCount: 7,
    },
    {
        id: "s-03",
        title: "OKLCH color palette review",
        updatedAt: "2025-01-19T17:03:00Z",
        messageCount: 21,
    },
    {
        id: "s-04",
        title: "Draft README for self-hosting",
        updatedAt: "2025-01-19T11:26:00Z",
        messageCount: 4,
    },
    {
        id: "s-05",
        title: "Compare provider rate limits",
        updatedAt: "2025-01-18T15:48:00Z",
        messageCount: 9,
    },
    {
        id: "s-06",
        title: "React 19 concurrent features",
        updatedAt: "2025-01-17T10:12:00Z",
        messageCount: 15,
    },
    {
        id: "s-07",
        title: "i18n key naming convention",
        updatedAt: "2025-01-16T14:37:00Z",
        messageCount: 6,
    },
    {
        id: "s-08",
        title: "SQLite vs Postgres for local store",
        updatedAt: "2025-01-15T09:05:00Z",
        messageCount: 11,
    },
    {
        id: "s-09",
        title: "Old experiments with prompts",
        updatedAt: "2025-01-02T13:20:00Z",
        messageCount: 30,
        archived: true,
    },
    {
        id: "s-10",
        title: "Deprecated gateway config",
        updatedAt: "2024-12-28T16:44:00Z",
        messageCount: 5,
        archived: true,
    },
];

/* ------------------------------- Messages ------------------------------- */

/** Structured content block — rendered by type, no markdown parser needed. */
export type MessageBlock =
    | { type: "paragraph"; segments: InlineSegment[] }
    | { type: "heading"; level: 2 | 3; text: string }
    | { type: "list"; ordered?: boolean; items: InlineSegment[][] }
    | { type: "quote"; segments: InlineSegment[] }
    | { type: "code"; language: string; code: string };

export type InlineSegment =
    | { type: "text"; text: string }
    | { type: "code"; text: string }
    | { type: "strong"; text: string };

export interface ChatMessage {
    id: string;
    role: "user" | "assistant";
    blocks: MessageBlock[];
    /** e.g. "openai · gpt-4o" — assistant messages only */
    source?: string;
    /** true while the assistant response is still streaming in */
    streaming?: boolean;
}

export const mockMessages: ChatMessage[] = [
    {
        id: "m-01",
        role: "user",
        blocks: [
            {
                type: "paragraph",
                segments: [
                    {
                        type: "text",
                        text: "How do I stream responses from my self-hosted gateway into a React chat UI without a heavy state library?",
                    },
                ],
            },
        ],
    },
    {
        id: "m-02",
        role: "assistant",
        source: "openai · gpt-4o",
        blocks: [
            {
                type: "paragraph",
                segments: [
                    {
                        type: "text",
                        text: "The simplest reliable approach is ",
                    },
                    { type: "code", text: "fetch" },
                    { type: "text", text: " with a readable stream. Unlike " },
                    { type: "code", text: "EventSource" },
                    {
                        type: "text",
                        text: ", it supports POST bodies and custom headers, which most providers require.",
                    },
                ],
            },
            { type: "heading", level: 2, text: "Recommended approach" },
            {
                type: "list",
                items: [
                    [
                        { type: "strong", text: "Open one request" },
                        {
                            type: "text",
                            text: " per message and read the body as a stream.",
                        },
                    ],
                    [
                        {
                            type: "text",
                            text: "Buffer chunks, split on newlines, and parse ",
                        },
                        { type: "code", text: "data:" },
                        { type: "text", text: " frames." },
                    ],
                    [
                        {
                            type: "text",
                            text: "Append deltas to local state — a single ",
                        },
                        { type: "code", text: "useState" },
                        { type: "text", text: " is enough." },
                    ],
                ],
            },
            {
                type: "quote",
                segments: [
                    {
                        type: "text",
                        text: "Keep the parser dumb: transport framing in, plain text deltas out.",
                    },
                ],
            },
            { type: "heading", level: 3, text: "Client example" },
            {
                type: "code",
                language: "tsx",
                code: [
                    'const res = await fetch("/v1/chat", { method: "POST", body });',
                    "const reader = res.body!.getReader();",
                    "const decoder = new TextDecoder();",
                    "while (true) {",
                    "  const { done, value } = await reader.read();",
                    "  if (done) break;",
                    "  appendDelta(decoder.decode(value, { stream: true }));",
                    "}",
                ].join("\n"),
            },
            {
                type: "code",
                language: "bash",
                code: 'curl -N -X POST http://localhost:8787/v1/chat \\\n  -H "Authorization: Bearer $TOKEN" \\\n  -d \'{"model":"gpt-4o","stream":true}\'',
            },
            {
                type: "paragraph",
                segments: [
                    {
                        type: "text",
                        text: "This keeps your bundle small and the data flow obvious. Add retries only where the connection actually drops.",
                    },
                ],
            },
        ],
    },
    {
        id: "m-03",
        role: "user",
        blocks: [
            {
                type: "paragraph",
                segments: [
                    {
                        type: "text",
                        text: "Great — and how should I handle aborting mid-stream?",
                    },
                ],
            },
        ],
    },
    {
        id: "m-04",
        role: "assistant",
        source: "anthropic · claude-sonnet-4",
        streaming: true,
        blocks: [
            {
                type: "paragraph",
                segments: [
                    { type: "text", text: "Use an " },
                    { type: "code", text: "AbortController" },
                    {
                        type: "text",
                        text: " per request and pass its signal to fetch. When the user hits stop…",
                    },
                ],
            },
        ],
    },
];

/* ------------------------------- Providers ------------------------------ */

export type ProviderType = "openai" | "anthropic" | "gemini";

export interface Provider {
    id: string;
    name: string;
    type: ProviderType;
    baseUrl: string;
    /** never a real key — masked display value or empty when unconfigured */
    apiKeyMasked: string;
    models: string[];
    configured: boolean;
    /** shown in the enabled/disabled groups and the header switch */
    enabled: boolean;
}

export const mockProviders: Provider[] = [
    {
        id: "p-01",
        name: "OpenAI",
        type: "openai",
        baseUrl: "https://api.openai.com/v1",
        apiKeyMasked: "sk-••••••••••••4f2a",
        models: ["gpt-4o", "gpt-4o-mini", "o3-mini"],
        configured: true,
        enabled: true,
    },
    {
        id: "p-02",
        name: "Anthropic",
        type: "anthropic",
        baseUrl: "https://api.anthropic.com/v1",
        apiKeyMasked: "sk-ant-••••••••9c1d",
        models: ["claude-opus-4", "claude-sonnet-4"],
        configured: true,
        enabled: true,
    },
    {
        id: "p-03",
        name: "Gemini",
        type: "gemini",
        baseUrl: "https://generativelanguage.googleapis.com/v1beta",
        apiKeyMasked: "",
        models: [],
        configured: false,
        enabled: false,
    },
];

/** Returned by the mocked "fetch from remote" action in the provider form. */
export const mockRemoteModels: string[] = [
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "o3",
    "o3-mini",
    "o4-mini",
];

/* ------------------------------ Empty state ----------------------------- */

/** i18n keys for the suggestion cards on the empty chat state. */
export const emptySuggestions: string[] = [
    "chat.suggestion1",
    "chat.suggestion2",
    "chat.suggestion3",
];

/* --------------------------- v2: home / sidebar ------------------------- */

export interface MockUser {
    name: string;
    stats: { sessions: number; topics: number; messages: number };
}

export const mockUser: MockUser = {
    name: "Demo User",
    stats: { sessions: 8, topics: 134, messages: 2964 },
};

/* --------------------- v2: extended provider models --------------------- */

export type ModelCapability = "vision" | "video" | "tools";
export type ModelKind = "chat" | "image" | "embedding";

export interface ProviderModel {
    id: string;
    name: string;
    /** raw model id sent to the API, shown as a mono badge */
    modelId: string;
    kind: ModelKind;
    capabilities: ModelCapability[];
    /** context window in K tokens, e.g. 256 → "256K" */
    contextK: number;
    enabled: boolean;
}

/** Detailed model lists keyed by provider id (drives the model table). */
export const mockProviderModels: Record<string, ProviderModel[]> = {
    "p-01": [
        {
            id: "m-01",
            name: "GPT-4o",
            modelId: "gpt-4o",
            kind: "chat",
            capabilities: ["vision", "tools"],
            contextK: 128,
            enabled: true,
        },
        {
            id: "m-02",
            name: "GPT-4o mini",
            modelId: "gpt-4o-mini",
            kind: "chat",
            capabilities: ["vision", "tools"],
            contextK: 128,
            enabled: true,
        },
        {
            id: "m-03",
            name: "o3 mini",
            modelId: "o3-mini",
            kind: "chat",
            capabilities: ["tools"],
            contextK: 200,
            enabled: true,
        },
        {
            id: "m-04",
            name: "DALL·E 3",
            modelId: "dall-e-3",
            kind: "image",
            capabilities: [],
            contextK: 4,
            enabled: false,
        },
        {
            id: "m-05",
            name: "text-embedding-3-large",
            modelId: "text-embedding-3-large",
            kind: "embedding",
            capabilities: [],
            contextK: 8,
            enabled: false,
        },
    ],
    "p-02": [
        {
            id: "m-06",
            name: "Claude Opus 4",
            modelId: "claude-opus-4",
            kind: "chat",
            capabilities: ["vision", "tools"],
            contextK: 200,
            enabled: true,
        },
        {
            id: "m-07",
            name: "Claude Sonnet 4",
            modelId: "claude-sonnet-4",
            kind: "chat",
            capabilities: ["vision", "tools"],
            contextK: 200,
            enabled: true,
        },
    ],
    "p-03": [],
};
