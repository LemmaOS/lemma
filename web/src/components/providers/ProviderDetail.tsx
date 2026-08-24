import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
    Check,
    Eye,
    EyeOff,
    Lock,
    MoreHorizontal,
    Plus,
    RefreshCw,
    Settings,
    Undo2,
    Video,
    Wrench,
    X,
} from "lucide-react";
import type {
    ModelCapability,
    ModelKind,
    Provider,
    ProviderModel,
} from "@/mocks";
import { mockRemoteModels } from "@/mocks";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";

/* ------------------------------- helpers ------------------------------- */

function formatContext(contextK: number): string {
    return contextK >= 1000 ? `${contextK / 1000}M` : `${contextK}K`;
}

type ModelTab = "all" | ModelKind;

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

function CapabilityIcon({
    icon: Icon,
    label,
    active,
}: {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    active: boolean;
}) {
    return (
        <span title={label} aria-label={label}>
            <Icon
                className={cn(
                    "size-4",
                    active
                        ? "text-muted-foreground"
                        : "text-muted-foreground/30",
                )}
            />
        </span>
    );
}

const CAPABILITIES: {
    key: ModelCapability;
    icon: React.ComponentType<{ className?: string }>;
    labelKey: string;
}[] = [
    { key: "vision", icon: Eye, labelKey: "providers.capVision" },
    { key: "video", icon: Video, labelKey: "providers.capVideo" },
    { key: "tools", icon: Wrench, labelKey: "providers.capTools" },
];

/* --------------------------- model list block --------------------------- */

interface ModelListProps {
    models: ProviderModel[];
    onChange: (models: ProviderModel[]) => void;
}

function ModelList({ models, onChange }: ModelListProps) {
    const { t } = useTranslation();
    const [query, setQuery] = useState("");
    const [tab, setTab] = useState<ModelTab>("all");
    const [fetching, setFetching] = useState(false);
    const [adding, setAdding] = useState(false);
    const [newName, setNewName] = useState("");
    const [newModelId, setNewModelId] = useState("");

    const counts = useMemo(() => {
        const c: Record<ModelTab, number> = {
            all: models.length,
            chat: 0,
            image: 0,
            embedding: 0,
        };
        for (const m of models) c[m.kind] += 1;
        return c;
    }, [models]);

    const visible = useMemo(() => {
        const q = query.trim().toLowerCase();
        return models.filter((m) => {
            if (tab !== "all" && m.kind !== tab) return false;
            if (
                q &&
                !m.name.toLowerCase().includes(q) &&
                !m.modelId.toLowerCase().includes(q)
            )
                return false;
            return true;
        });
    }, [models, query, tab]);

    const enabledModels = visible.filter((m) => m.enabled);
    const disabledModels = visible.filter((m) => !m.enabled);

    const toggleModel = (id: string, enabled: boolean) => {
        onChange(models.map((m) => (m.id === id ? { ...m, enabled } : m)));
    };

    const fetchRemote = () => {
        if (fetching) return;
        setFetching(true);
        window.setTimeout(() => {
            const existing = new Set(models.map((m) => m.modelId));
            const incoming: ProviderModel[] = mockRemoteModels
                .filter((modelId) => !existing.has(modelId))
                .map((modelId) => ({
                    id: `m-${Date.now()}-${modelId}`,
                    name: modelId,
                    modelId,
                    kind: "chat",
                    capabilities: ["tools"],
                    contextK: 128,
                    enabled: false,
                }));
            onChange([...models, ...incoming]);
            setFetching(false);
        }, 1200);
    };

    const confirmAdd = () => {
        const modelId = newModelId.trim();
        const name = newName.trim() || modelId;
        if (!modelId) return;
        onChange([
            {
                id: `m-${Date.now()}`,
                name,
                modelId,
                kind: "chat",
                capabilities: ["tools"],
                contextK: 128,
                enabled: true,
            },
            ...models,
        ]);
        setNewName("");
        setNewModelId("");
        setAdding(false);
    };

    const cancelAdd = () => {
        setNewName("");
        setNewModelId("");
        setAdding(false);
    };

    const tabs: { key: ModelTab; labelKey: string }[] = [
        { key: "all", labelKey: "providers.tabAll" },
        { key: "chat", labelKey: "providers.tabChat" },
        { key: "image", labelKey: "providers.tabImage" },
        { key: "embedding", labelKey: "providers.tabEmbedding" },
    ];

    const renderRow = (model: ProviderModel) => (
        <div
            key={model.id}
            className="flex items-center gap-3 border-b border-border/60 py-3"
        >
            <span className="grid size-8 shrink-0 place-items-center rounded-full bg-muted text-xs font-semibold">
                {model.name.charAt(0).toUpperCase()}
            </span>
            <span
                className={cn(
                    "truncate text-sm font-medium",
                    !model.enabled && "text-muted-foreground",
                )}
            >
                {model.name}
            </span>
            <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-muted-foreground">
                {model.modelId}
            </span>
            <div className="ml-auto flex shrink-0 items-center gap-3">
                <div className="flex items-center gap-2">
                    {CAPABILITIES.map(({ key, icon, labelKey }) => (
                        <CapabilityIcon
                            key={key}
                            icon={icon}
                            label={t(labelKey)}
                            active={model.capabilities.includes(key)}
                        />
                    ))}
                </div>
                <span className="w-10 text-right text-xs text-muted-foreground">
                    {formatContext(model.contextK)}
                </span>
                <Switch
                    checked={model.enabled}
                    onCheckedChange={(checked) =>
                        toggleModel(model.id, checked)
                    }
                    aria-label={t("providers.enableModel")}
                />
            </div>
        </div>
    );

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
                        onClick={fetchRemote}
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
                        aria-label={t("providers.newModel")}
                        onClick={() => setAdding((v) => !v)}
                    >
                        <Plus className="size-4" />
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="size-8"
                        aria-label={t("providers.moreActions")}
                    >
                        <MoreHorizontal className="size-4" />
                    </Button>
                </div>
            </div>

            <div className="mt-3 flex items-center gap-4 border-b border-border">
                {tabs.map(({ key, labelKey }) => (
                    <button
                        key={key}
                        type="button"
                        onClick={() => setTab(key)}
                        aria-pressed={tab === key}
                        className={cn(
                            "-mb-px border-b-2 border-transparent pb-2 text-sm transition-colors",
                            tab === key
                                ? "border-primary text-foreground"
                                : "text-muted-foreground hover:text-foreground",
                        )}
                    >
                        {t(labelKey)}
                        <span className="ml-1 text-xs text-muted-foreground">
                            {counts[key]}
                        </span>
                    </button>
                ))}
            </div>

            {adding && (
                <div className="flex items-center gap-2 border-b border-border/60 py-3">
                    <Input
                        value={newName}
                        onChange={(e) => setNewName(e.target.value)}
                        placeholder={t("providers.modelNamePlaceholder")}
                        className="h-8 flex-1"
                    />
                    <Input
                        value={newModelId}
                        onChange={(e) => setNewModelId(e.target.value)}
                        placeholder={t("providers.modelIdPlaceholder")}
                        className="h-8 flex-1 font-mono"
                    />
                    <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        className="size-8 shrink-0"
                        aria-label={t("providers.addModel")}
                        onClick={confirmAdd}
                        disabled={!newModelId.trim()}
                    >
                        <Check className="size-4" />
                    </Button>
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="size-8 shrink-0"
                        aria-label={t("common.cancel")}
                        onClick={cancelAdd}
                    >
                        <X className="size-4" />
                    </Button>
                </div>
            )}

            {enabledModels.length > 0 && (
                <>
                    <p className="pt-4 pb-1 text-xs text-muted-foreground">
                        {t("providers.enabledGroup")}
                    </p>
                    {enabledModels.map(renderRow)}
                </>
            )}
            {disabledModels.length > 0 && (
                <>
                    <p className="pt-4 pb-1 text-xs text-muted-foreground">
                        {t("providers.disabledGroup")}
                    </p>
                    {disabledModels.map(renderRow)}
                </>
            )}
            {visible.length === 0 && (
                <p className="py-8 text-center text-sm text-muted-foreground">
                    {t("providers.noModels")}
                </p>
            )}
        </section>
    );
}

/* ------------------------------ detail panel ---------------------------- */

interface ProviderDetailProps {
    provider: Provider;
    models: ProviderModel[];
    onToggleEnabled: (enabled: boolean) => void;
    onModelsChange: (models: ProviderModel[]) => void;
}

export function ProviderDetail({
    provider,
    models,
    onToggleEnabled,
    onModelsChange,
}: ProviderDetailProps) {
    const { t } = useTranslation();
    const [apiKey, setApiKey] = useState(provider.apiKeyMasked);
    const [showKey, setShowKey] = useState(false);
    const [proxyUrl, setProxyUrl] = useState(provider.baseUrl);
    const [clientMode, setClientMode] = useState(false);
    const [checkModel, setCheckModel] = useState<string | undefined>(undefined);
    const [checkState, setCheckState] = useState<
        "idle" | "checking" | "success"
    >("idle");

    const enabledModels = models.filter((m) => m.enabled);

    const runCheck = () => {
        if (checkState === "checking") return;
        setCheckState("checking");
        window.setTimeout(() => setCheckState("success"), 1000);
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
                        aria-label={t("providers.providerSettings")}
                    >
                        <Settings className="size-4" />
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
                    description={t("providers.apiKeyDesc")}
                >
                    <div className="relative flex-1">
                        <Input
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
                </FieldRow>

                <FieldRow
                    label={t("providers.proxyUrl")}
                    description={t("providers.proxyDesc")}
                >
                    <Input
                        value={proxyUrl}
                        onChange={(e) => setProxyUrl(e.target.value)}
                        placeholder={t("providers.baseUrlPlaceholder")}
                        className="flex-1"
                    />
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        aria-label={t("providers.resetProxy")}
                        onClick={() => setProxyUrl(provider.baseUrl)}
                    >
                        <Undo2 className="size-4" />
                    </Button>
                </FieldRow>

                <FieldRow
                    label={t("providers.clientMode")}
                    description={t("providers.clientModeDesc")}
                >
                    <Switch
                        checked={clientMode}
                        onCheckedChange={setClientMode}
                        aria-label={t("providers.clientMode")}
                    />
                </FieldRow>

                <FieldRow
                    label={t("providers.connectivityCheck")}
                    description={t("providers.checkDesc")}
                >
                    <Select value={checkModel} onValueChange={setCheckModel}>
                        <SelectTrigger
                            className="flex-1"
                            aria-label={t("providers.selectModelPlaceholder")}
                        >
                            <SelectValue
                                placeholder={t(
                                    "providers.selectModelPlaceholder",
                                )}
                            />
                        </SelectTrigger>
                        <SelectContent>
                            {enabledModels.map((m) => (
                                <SelectItem key={m.id} value={m.modelId}>
                                    {m.name}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <Button
                        type="button"
                        variant="outline"
                        onClick={runCheck}
                        disabled={
                            checkState === "checking" ||
                            enabledModels.length === 0
                        }
                    >
                        {checkState === "checking"
                            ? t("providers.checking")
                            : t("providers.check")}
                    </Button>
                </FieldRow>
                {checkState === "success" && (
                    <p className="flex items-center gap-1.5 pt-2 text-xs text-muted-foreground">
                        <Check className="size-3.5 text-primary" />
                        {t("providers.checkSuccess")}
                    </p>
                )}

                <p className="flex items-center gap-1.5 pt-4 text-xs text-muted-foreground">
                    <Lock className="size-3.5" />
                    {t("providers.securityNote")}
                </p>
            </section>

            <ModelList models={models} onChange={onModelsChange} />
        </div>
    );
}
