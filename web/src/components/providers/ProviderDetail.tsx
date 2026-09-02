import {
    Check,
    Eye,
    EyeOff,
    Lock,
    Plus,
    RefreshCw,
    Trash2,
    X,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import type { Provider } from "@/gen/lemma/v1/provider_pb";
import { errorText } from "@/lib/errors";
import { cn } from "@/lib/utils";

function FieldRow({
    label,
    description,
    children,
}: {
    label: string;
    description: string;
    children: React.ReactNode;
}) {
    return (
        <div className="flex items-center justify-between gap-6 border-b border-border/60 py-4">
            <div className="min-w-0">
                <p className="text-sm font-medium">{label}</p>
                <p className="mt-0.5 text-xs text-muted-foreground">
                    {description}
                </p>
            </div>
            <div className="flex w-[380px] shrink-0 items-center justify-end gap-2">
                {children}
            </div>
        </div>
    );
}

interface ModelListProps {
    models: string[];
    onChange: (models: string[]) => void;
    onFetch: () => Promise<string[]>;
}

function ModelList({ models, onChange, onFetch }: ModelListProps) {
    const { t } = useTranslation();
    const [query, setQuery] = useState("");
    const [fetching, setFetching] = useState(false);
    const [fetchMsg, setFetchMsg] = useState<
        { ok: true; text: string } | { ok: false; text: string } | null
    >(null);
    const [adding, setAdding] = useState(false);
    const [newModel, setNewModel] = useState("");

    const visible = useMemo(() => {
        const q = query.trim().toLowerCase();
        return q ? models.filter((m) => m.toLowerCase().includes(q)) : models;
    }, [models, query]);

    const fetchRemote = async () => {
        if (fetching) return;
        setFetching(true);
        setFetchMsg(null);
        try {
            const remote = await onFetch();
            onChange(Array.from(new Set([...models, ...remote])));
            setFetchMsg({ ok: true, text: t("providers.modelsFetched") });
        } catch (e) {
            setFetchMsg({ ok: false, text: errorText(e, t) });
        } finally {
            setFetching(false);
        }
    };

    const confirmAdd = () => {
        const id = newModel.trim();
        setNewModel("");
        setAdding(false);
        if (!id || models.includes(id)) return;
        onChange([id, ...models]);
    };

    const remove = (id: string) => onChange(models.filter((m) => m !== id));

    return (
        <section className="pt-6">
            <div className="flex items-center gap-2">
                <h3 className="text-sm font-semibold">
                    {t("providers.modelList")}
                </h3>
                <div className="ml-auto flex items-center gap-2">
                    <Input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder={t("providers.modelSearchPlaceholder")}
                        className="h-8 w-48"
                    />
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => void fetchRemote()}
                        disabled={fetching}
                    >
                        <RefreshCw
                            className={cn(
                                "size-3.5",
                                fetching && "animate-spin",
                            )}
                        />
                        {fetching
                            ? t("providers.fetchingModels")
                            : t("providers.fetchModelList")}
                    </Button>
                    <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        className="size-8"
                        aria-label={t("providers.addModel")}
                        onClick={() => setAdding((v) => !v)}
                    >
                        <Plus className="size-4" />
                    </Button>
                </div>
            </div>

            {fetchMsg && (
                <p
                    className={cn(
                        "pt-2 text-xs",
                        fetchMsg.ok
                            ? "text-muted-foreground"
                            : "text-destructive",
                    )}
                >
                    {fetchMsg.text}
                </p>
            )}

            {adding && (
                <div className="flex items-center gap-2 border-b border-border/60 py-3">
                    <Input
                        value={newModel}
                        onChange={(e) => setNewModel(e.target.value)}
                        placeholder={t("providers.modelPlaceholder")}
                        className="h-8 flex-1 font-mono"
                        onKeyDown={(e) => {
                            if (e.key === "Enter") {
                                e.preventDefault();
                                confirmAdd();
                            } else if (e.key === "Escape") {
                                setNewModel("");
                                setAdding(false);
                            }
                        }}
                    />
                    <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        className="size-8 shrink-0"
                        aria-label={t("providers.addModel")}
                        onClick={confirmAdd}
                        disabled={!newModel.trim()}
                    >
                        <Check className="size-4" />
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="size-8 shrink-0"
                        aria-label={t("common.cancel")}
                        onClick={() => {
                            setNewModel("");
                            setAdding(false);
                        }}
                    >
                        <X className="size-4" />
                    </Button>
                </div>
            )}

            {visible.length > 0 ? (
                visible.map((model) => (
                    <div
                        key={model}
                        className="flex items-center gap-3 border-b border-border/60 py-3"
                    >
                        <span className="truncate font-mono text-sm">
                            {model}
                        </span>
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            className="ml-auto size-7 shrink-0 text-muted-foreground hover:text-destructive"
                            aria-label={t("providers.removeModel")}
                            onClick={() => remove(model)}
                        >
                            <X className="size-3.5" />
                        </Button>
                    </div>
                ))
            ) : (
                <p className="py-8 text-center text-sm text-muted-foreground">
                    {t("providers.noModels")}
                </p>
            )}
        </section>
    );
}

interface ProviderDetailProps {
    provider: Provider;
    onToggleEnabled: (enabled: boolean) => void;
    onSaveBaseUrl: (baseUrl: string) => Promise<void>;
    onSaveApiKey: (apiKey: string) => Promise<void>;
    onModelsChange: (models: string[]) => void;
    onFetchModels: () => Promise<string[]>;
    onDelete: () => void | Promise<void>;
}

export function ProviderDetail({
    provider,
    onToggleEnabled,
    onSaveBaseUrl,
    onSaveApiKey,
    onModelsChange,
    onFetchModels,
    onDelete,
}: ProviderDetailProps) {
    const { t } = useTranslation();
    const [apiKey, setApiKey] = useState("");
    const [showKey, setShowKey] = useState(false);
    const [baseUrl, setBaseUrl] = useState(provider.baseUrl);
    const [savingKey, setSavingKey] = useState(false);
    const [savingUrl, setSavingUrl] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const apiKeyDirty = apiKey.trim().length > 0;
    const baseUrlDirty =
        baseUrl.trim() !== provider.baseUrl && baseUrl.trim().length > 0;

    const saveApiKey = async () => {
        setSavingKey(true);
        setError(null);
        try {
            await onSaveApiKey(apiKey.trim());
            setApiKey("");
        } catch (e) {
            setError(errorText(e, t));
        } finally {
            setSavingKey(false);
        }
    };

    const saveBaseUrl = async () => {
        setSavingUrl(true);
        setError(null);
        try {
            await onSaveBaseUrl(baseUrl.trim());
        } catch (e) {
            setBaseUrl(provider.baseUrl);
            setError(errorText(e, t));
        } finally {
            setSavingUrl(false);
        }
    };

    const handleDelete = async () => {
        if (!window.confirm(t("providers.deleteConfirm"))) return;
        setError(null);
        try {
            await onDelete();
        } catch (e) {
            setError(errorText(e, t));
        }
    };

    return (
        <div className="max-w-3xl px-8 py-6">
            <header className="flex items-center gap-3">
                <span className="grid size-9 shrink-0 place-items-center rounded-md border border-border bg-background text-sm font-semibold">
                    {provider.name.charAt(0).toUpperCase()}
                </span>
                <h2 className="text-base font-semibold">{provider.name}</h2>
                <div className="ml-auto flex items-center gap-3">
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="text-muted-foreground hover:text-destructive"
                        aria-label={t("providers.deleteProvider")}
                        onClick={() => void handleDelete()}
                    >
                        <Trash2 className="size-4" />
                    </Button>
                    <Switch
                        checked={provider.enabled}
                        onCheckedChange={onToggleEnabled}
                        aria-label={t("providers.enableProvider")}
                    />
                </div>
            </header>

            <section className="mt-4">
                <FieldRow
                    label={t("providers.apiKey")}
                    description={t("providers.apiKeyKeepHint")}
                >
                    <div className="relative flex-1">
                        <Input
                            type={showKey ? "text" : "password"}
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder={
                                provider.apiKey ||
                                t("providers.apiKeyPlaceholder")
                            }
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
                    <Button
                        type="button"
                        size="sm"
                        disabled={!apiKeyDirty || savingKey}
                        onClick={() => void saveApiKey()}
                    >
                        {t("common.save")}
                    </Button>
                </FieldRow>

                <FieldRow
                    label={t("providers.baseUrl")}
                    description={t("providers.baseUrlDesc")}
                >
                    <Input
                        value={baseUrl}
                        onChange={(e) => setBaseUrl(e.target.value)}
                        placeholder={t("providers.baseUrlPlaceholder")}
                        className="flex-1 font-mono text-xs"
                    />
                    <Button
                        type="button"
                        size="sm"
                        disabled={!baseUrlDirty || savingUrl}
                        onClick={() => void saveBaseUrl()}
                    >
                        {t("common.save")}
                    </Button>
                </FieldRow>

                {error && (
                    <p className="pt-2 text-xs text-destructive">{error}</p>
                )}

                <p className="flex items-center gap-1.5 pt-4 text-xs text-muted-foreground">
                    <Lock className="size-3.5" />
                    {t("providers.securityNote")}
                </p>
            </section>

            <ModelList
                models={provider.models}
                onChange={onModelsChange}
                onFetch={onFetchModels}
            />
        </div>
    );
}
