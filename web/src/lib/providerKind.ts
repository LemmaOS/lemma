import { ProviderKind } from "@/gen/lemma/v1/provider_pb";

export function kindLabel(kind: ProviderKind): string {
    switch (kind) {
        case ProviderKind.OPENAI:
            return "openai";
        case ProviderKind.ANTHROPIC:
            return "anthropic";
        case ProviderKind.GEMINI:
            return "gemini";
        default:
            return "unknown";
    }
}
