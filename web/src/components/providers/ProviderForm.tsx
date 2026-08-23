import { Check, Eye, EyeOff, RefreshCw, Trash2 } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";

import { ModelTagsInput } from "@/components/providers/ModelTagsInput";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { ProviderKind, type Provider } from "@/gen/lemma/v1/provider_pb";

const KIND_OPTIONS = [
    { value: "openai", kind: ProviderKind.OPENAI },
    { value: "anthropic", kind: ProviderKind.ANTHROPIC },
    { value: "gemini", kind: ProviderKind.GEMINI },
] as const;

function kindToValue(kind: ProviderKind): string {
    return KIND_OPTIONS.find((o) => o.kind === kind)?.value ?? "openai";
}

export interface ProviderFormValues {
    kind: ProviderKind;
    name: string;
    baseUrl: string;
    /** 编辑时留空表示不变更密钥 */
    apiKey: string;
    models: string[];
}

interface ProviderFormProps {
    /** 传入为编辑；不传为新建 */
    provider?: Provider;
    onSave: (values: ProviderFormValues) => Promise<void>;
    onCancel?: () => void;
    onDelete?: () => Promise<void>;
    onFetchModels: (values: {
        kind: ProviderKind;
        baseUrl: string;
        apiKey: string;
    }) => Promise<string[]>;
}

export function ProviderForm({
    provider,
    onSave,
    onCancel,
    onDelete,
    onFetchModels,
}: ProviderFormProps) {
    const { t } = useTranslation();
    const isNew = !provider;

    const [name, setName] = useState(provider?.name ?? "");
    const [kindValue, setKindValue] = useState(
        kindToValue(provider?.kind ?? ProviderKind.OPENAI),
    );
    const [baseUrl, setBaseUrl] = useState(provider?.baseUrl ?? "");
    // 编辑时密钥不回填（服务端只给脱敏串），留空 = 不变更
    const [apiKey, setApiKey] = useState("");
    const [models, setModels] = useState<string[]>(provider?.models ?? []);

    const [showApiKey, setShowApiKey] = useState(false);
    const [fetching, setFetching] = useState(false);
    const [fetchState, setFetchState] = useState<"idle" | "ok" | "fail">(
        "idle",
    );
    const [saving, setSaving] = useState(false);
    const [saveFailed, setSaveFailed] = useState(false);
    const [confirmingDelete, setConfirmingDelete] = useState(false);

    const handleFetch = async () => {
        setFetching(true);
        setFetchState("idle");
        try {
            const kind =
                KIND_OPTIONS.find((o) => o.value === kindValue)?.kind ??
                ProviderKind.OPENAI;
            const remote = await onFetchModels({
                kind,
                baseUrl: baseUrl.trim(),
                apiKey,
            });
            setModels((current) =>
                Array.from(new Set([...current, ...remote])),
            );
            setFetchState("ok");
        } catch {
            setFetchState("fail");
        } finally {
            setFetching(false);
        }
    };

    const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setSaving(true);
        setSaveFailed(false);
        try {
            await onSave({
                kind:
                    KIND_OPTIONS.find((o) => o.value === kindValue)?.kind ??
                    ProviderKind.OPENAI,
                name: name.trim(),
                baseUrl: baseUrl.trim(),
                apiKey,
                models,
            });
        } catch {
            setSaveFailed(true);
        } finally {
            setSaving(false);
        }
    };

    const handleDelete = async () => {
        if (!onDelete) return;
        if (!confirmingDelete) {
            setConfirmingDelete(true);
            return;
        }
        await onDelete();
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-base">
                    {isNew ? t("providers.newProvider") : provider.name}
                </CardTitle>
            </CardHeader>
            <CardContent>
                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="grid gap-2">
                        <Label htmlFor="provider-name">
                            {t("providers.name")}
                        </Label>
                        <Input
                            id="provider-name"
                            value={name}
                            placeholder={t("providers.namePlaceholder")}
                            onChange={(event) => setName(event.target.value)}
                            required
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="provider-type">
                            {t("providers.type")}
                        </Label>
                        <Select
                            value={kindValue}
                            onValueChange={setKindValue}
                            disabled={!isNew}
                        >
                            <SelectTrigger id="provider-type" className="w-full">
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

                    <div className="grid gap-2">
                        <Label htmlFor="provider-base-url">
                            {t("providers.baseUrl")}
                        </Label>
                        <Input
                            id="provider-base-url"
                            value={baseUrl}
                            placeholder={t("providers.baseUrlPlaceholder")}
                            onChange={(event) => setBaseUrl(event.target.value)}
                            required
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="provider-api-key">
                            {t("providers.apiKey")}
                        </Label>
                        <div className="relative">
                            <Input
                                id="provider-api-key"
                                type={showApiKey ? "text" : "password"}
                                value={apiKey}
                                placeholder={t("providers.apiKeyPlaceholder")}
                                onChange={(event) =>
                                    setApiKey(event.target.value)
                                }
                                className="pr-9"
                                required={isNew}
                            />
                            <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                aria-label={
                                    showApiKey
                                        ? t("providers.hideApiKey")
                                        : t("providers.showApiKey")
                                }
                                onClick={() =>
                                    setShowApiKey((visible) => !visible)
                                }
                                className="absolute top-1/2 right-1 size-7 -translate-y-1/2"
                            >
                                {showApiKey ? (
                                    <EyeOff className="size-4" />
                                ) : (
                                    <Eye className="size-4" />
                                )}
                            </Button>
                        </div>
                        {!isNew && (
                            <p className="text-xs text-muted-foreground">
                                {t("providers.apiKeyKeepHint")}
                            </p>
                        )}
                    </div>

                    <ModelTagsInput models={models} onChange={setModels} />

                    <div className="flex items-center gap-3">
                        <Button
                            type="button"
                            variant="outline"
                            onClick={handleFetch}
                            disabled={fetching}
                        >
                            <RefreshCw
                                className={
                                    fetching ? "size-4 animate-spin" : "size-4"
                                }
                            />
                            {fetching
                                ? t("providers.fetchingModels")
                                : t("providers.fetchModels")}
                        </Button>
                        {fetchState === "ok" && (
                            <span className="flex items-center gap-1 text-xs text-muted-foreground">
                                <Check className="size-3.5" />
                                {t("providers.modelsFetched")}
                            </span>
                        )}
                        {fetchState === "fail" && (
                            <span className="text-xs text-destructive">
                                {t("providers.fetchFailed")}
                            </span>
                        )}
                    </div>

                    <div className="flex items-center gap-2 border-t border-border pt-4">
                        {!isNew && onDelete && (
                            <Button
                                type="button"
                                variant="ghost"
                                className="text-destructive"
                                onClick={handleDelete}
                            >
                                <Trash2 className="size-4" />
                                {confirmingDelete
                                    ? t("providers.deleteConfirm")
                                    : t("providers.deleteProvider")}
                            </Button>
                        )}
                        {saveFailed && (
                            <span className="text-xs text-destructive">
                                {t("providers.saveFailed")}
                            </span>
                        )}
                        <div className="ml-auto flex items-center gap-2">
                            {isNew && onCancel && (
                                <Button
                                    type="button"
                                    variant="ghost"
                                    onClick={onCancel}
                                >
                                    {t("common.cancel")}
                                </Button>
                            )}
                            <Button type="submit" disabled={saving}>
                                {t("common.save")}
                            </Button>
                        </div>
                    </div>
                </form>
            </CardContent>
        </Card>
    );
}
