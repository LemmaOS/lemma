import { Code, ConnectError } from "@connectrpc/connect";
import type { TFunction } from "i18next";
import { expect, it } from "vitest";

import { ErrorInfoSchema, ErrorReason } from "@/gen/lemma/v1/errors_pb";
import { errorText } from "./errors";

// The stub echoes the key so assertions pin the reason→key mapping itself,
// independent of locale wording.
const t = ((key: string) => key) as unknown as TFunction;

function withReason(reason: ErrorReason): ConnectError {
    return new ConnectError("raw message", Code.NotFound, undefined, [
        { desc: ErrorInfoSchema, value: { reason, attrs: {} } },
    ]);
}

it("业务错误按 reason 映射到 i18n key", () => {
    expect(errorText(withReason(ErrorReason.PROVIDER_NOT_FOUND), t)).toBe(
        "errors.providerNotFound",
    );
});

// ConnectError.message carries a "[code] " prefix by design.
it("未知 reason 值没有映射，回退原始英文消息", () => {
    expect(errorText(withReason(999 as ErrorReason), t)).toBe(
        "[not_found] raw message",
    );
});

it("无 ErrorInfo 详情的 ConnectError 原样显示消息", () => {
    const e = new ConnectError("network unreachable", Code.Unavailable);
    expect(errorText(e, t)).toBe("[unavailable] network unreachable");
});

it("非 ConnectError 一律 String 化", () => {
    expect(errorText(new Error("boom"), t)).toBe("Error: boom");
    expect(errorText("plain", t)).toBe("plain");
});
