import { Eye, EyeOff } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { ProviderKind } from "@/gen/lemma/v1/provider_pb";

export interface NewProviderData {
    kind: ProviderKind;
    name: string;
    baseUrl: string;
    apiKey: string;
}

interface NewProviderFormProps {
    onSave: (data: NewProviderData) => Promise<void>;
    onCancel: () => void;
}

// 新建时切类型预填官方端点；用户改过就不动
const DEFAULT_BASE_URLS: Partial<Record<ProviderKind, string>> = {
    [ProviderKind.OPENAI]: "https://api.openai.com/v1",
    [ProviderKind.ANTHROPIC]: "https://api.anthropic.com/v1",
    [ProviderKind.GEMINI]: "https://generativelanguage.googleapis.com/v1beta",
};

const KIND_OPTIONS = [
    { value: "openai", kind: ProviderKind.OPENAI },
    { value: "anthropic", kind: ProviderKind.ANTHROPIC },
    { value: "gemini", kind: ProviderKind.GEMINI },
] as const;

export function NewProviderForm({ onSave, onCancel }: NewProviderFormProps) {
    const { t } = useTranslation();
    const [name, setName] = useState("");
    const [kind, setKind] = useState<ProviderKind>(ProviderKind.OPENAI);
    const [baseUrl, setBaseUrl] = useState(
        DEFAULT_BASE_URLS[ProviderKind.OPENAI] ?? "",
    );
    const [apiKey, setApiKey] = useState("");
    const [showKey, setShowKey] = useState(false);
    const [busy, setBusy] = useState(false);
    const [failed, setFailed] = useState(false);

    const handleKindChange = (value: string) => {
        const option = KIND_OPTIONS.find((o) => o.value === value);
        if (!option) return;
        setKind(option.kind);
        setBaseUrl((current) => {
            const untouched =
                current === "" ||
                Object.values(DEFAULT_BASE_URLS).includes(current);
            return untouched
                ? (DEFAULT_BASE_URLS[option.kind] ?? current)
                : current;
        });
    };

    const handleSave = async () => {
        setBusy(true);
        setFailed(false);
        try {
            const kindLabel =
                KIND_OPTIONS.find((o) => o.kind === kind)?.value ?? "provider";
            await onSave({
                kind,
                name: name.trim() || kindLabel,
                baseUrl: baseUrl.trim(),
                apiKey: apiKey.trim(),
            });
        } catch {
            setFailed(true);
        } finally {
            setBusy(false);
        }
    };

    return (
        <div className="max-w-3xl px-8 py-6">
            <h2 className="text-base font-semibold">
                {t("providers.newProvider")}
            </h2>
            <div className="mt-6 flex max-w-md flex-col gap-4">
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-kind">{t("providers.type")}</Label>
                    <Select
                        value={KIND_OPTIONS.find((o) => o.kind === kind)?.value}
                        onValueChange={handleKindChange}
                    >
                        <SelectTrigger id="np-kind" className="w-full">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {KIND_OPTIONS.map((o) => (
                                <SelectItem key={o.value} value={o.value}>
                                    {o.value}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-name">{t("providers.name")}</Label>
                    <Input
                        id="np-name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder={t("providers.namePlaceholder")}
                    />
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-baseurl">{t("providers.baseUrl")}</Label>
                    <Input
                        id="np-baseurl"
                        value={baseUrl}
                        onChange={(e) => setBaseUrl(e.target.value)}
                        placeholder={t("providers.baseUrlPlaceholder")}
                    />
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-apikey">{t("providers.apiKey")}</Label>
                    <div className="relative">
                        <Input
                            id="np-apikey"
                            type={showKey ? "text" : "password"}
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder={t("providers.apiKeyPlaceholder")}
                            className="pr-9"
                        />
                        <button
                            type="button"
                            onClick={() => setShowKey((v) => !v)}
                            aria-label={
                                showKey
                                    ? t("providers.hideApiKey")
                                    : t("providers.showApiKey")
                            }
                            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-foreground"
                        >
                            {showKey ? (
                                <EyeOff className="size-4" />
                            ) : (
                                <Eye className="size-4" />
                            )}
                        </button>
                    </div>
                </div>
                {failed && (
                    <p className="text-xs text-destructive">
                        {t("providers.saveFailed")}
                    </p>
                )}
                <div className="flex items-center gap-2 pt-2">
                    <Button
                        type="button"
                        onClick={() => void handleSave()}
                        disabled={busy}
                    >
                        {t("common.save")}
                    </Button>
                    <Button type="button" variant="outline" onClick={onCancel}>
                        {t("common.cancel")}
                    </Button>
                </div>
            </div>
        </div>
    );
}
