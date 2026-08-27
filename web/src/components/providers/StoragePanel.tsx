import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Code, ConnectError } from "@connectrpc/connect";
import { Eye, EyeOff } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { storageClient } from "@/lib/clients";

type Progress = { done: number; total: number; skipped: number };

interface Field {
    id: string;
    label: string;
    desc?: string;
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    secret?: boolean;
}

function FieldRow({
    field,
    show,
    onToggleShow,
}: {
    field: Field;
    show: boolean;
    onToggleShow: () => void;
}) {
    return (
        <div className="border-b border-border/60 py-4">
            <div className="flex items-center gap-4">
                <div className="min-w-0 flex-1">
                    <Label htmlFor={field.id}>{field.label}</Label>
                    {field.desc && (
                        <p className="mt-0.5 text-xs text-muted-foreground">
                            {field.desc}
                        </p>
                    )}
                </div>
                <div className="flex w-80 items-center gap-1">
                    <Input
                        id={field.id}
                        type={field.secret && !show ? "password" : "text"}
                        value={field.value}
                        onChange={(e) => field.onChange(e.target.value)}
                        placeholder={field.placeholder}
                    />
                    {field.secret && (
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            onClick={onToggleShow}
                            aria-label="toggle visibility"
                        >
                            {show ? (
                                <EyeOff className="size-4" />
                            ) : (
                                <Eye className="size-4" />
                            )}
                        </Button>
                    )}
                </div>
            </div>
        </div>
    );
}

/** S3 归档存储设置：表单 + 连通性测试 + 后端迁移进度 */
export function StoragePanel() {
    const { t } = useTranslation();
    const [endpoint, setEndpoint] = useState("");
    const [region, setRegion] = useState("");
    const [bucket, setBucket] = useState("");
    const [accessKey, setAccessKey] = useState("");
    const [secretKey, setSecretKey] = useState("");
    const [showAccess, setShowAccess] = useState(false);
    const [showSecret, setShowSecret] = useState(false);
    const [configured, setConfigured] = useState(false);
    const [loaded, setLoaded] = useState(false);
    const [pending, setPending] = useState(false);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [testMsg, setTestMsg] = useState<string | null>(null);
    const [progress, setProgress] = useState<Progress | null>(null);
    const [migrating, setMigrating] = useState(false);
    const [migrateDone, setMigrateDone] = useState(false);

    // 初次加载：回填已保存配置；密钥不回填（后端只返脱敏串，留空即保持）
    useEffect(() => {
        void (async () => {
            try {
                const resp = await storageClient.getStorageConfig({});
                if (resp.config) {
                    setEndpoint(resp.config.endpoint);
                    setRegion(resp.config.region);
                    setBucket(resp.config.bucket);
                    setConfigured(true);
                    setPending(resp.config.pendingMigration);
                }
            } finally {
                setLoaded(true);
            }
        })();
    }, []);

    const runMigration = async () => {
        setMigrating(true);
        setMigrateDone(false);
        setError(null);
        setProgress(null);
        try {
            for await (const frame of storageClient.migrateArchives({})) {
                setProgress({
                    done: frame.done,
                    total: frame.total,
                    skipped: frame.skipped,
                });
                if (frame.finished) {
                    if (frame.error) {
                        setError(frame.error);
                    } else {
                        setMigrateDone(true);
                        setPending(false);
                    }
                }
            }
        } catch (e) {
            setError(e instanceof ConnectError ? e.message : String(e));
        } finally {
            setMigrating(false);
        }
    };

    const handleSave = async () => {
        setBusy(true);
        setError(null);
        setTestMsg(null);
        try {
            const resp = await storageClient.updateStorageConfig({
                endpoint,
                region,
                bucket,
                accessKey,
                secretKey,
            });
            setConfigured(true);
            setPending(resp.config?.pendingMigration ?? false);
            if (resp.migrationTotal > 0) {
                setProgress({
                    done: 0,
                    total: resp.migrationTotal,
                    skipped: 0,
                });
                void runMigration();
            }
        } catch (e) {
            setError(e instanceof ConnectError ? e.message : String(e));
        } finally {
            setBusy(false);
        }
    };

    const handleTest = async () => {
        setBusy(true);
        setError(null);
        setTestMsg(null);
        try {
            await storageClient.testStorageConfig({
                endpoint,
                region,
                bucket,
                accessKey,
                secretKey,
            });
            setTestMsg(t("storage.testOk"));
        } catch (e) {
            if (e instanceof ConnectError && e.code === Code.NotFound) {
                setError(t("storage.bucketNotFound", { bucket }));
            } else if (e instanceof ConnectError) {
                setError(`${t("storage.testFail")} · ${e.message}`);
            } else {
                setError(String(e));
            }
        } finally {
            setBusy(false);
        }
    };

    const handleDelete = async () => {
        if (!window.confirm(t("storage.deleteConfirm"))) return;
        setBusy(true);
        setError(null);
        try {
            await storageClient.deleteStorageConfig({});
            setConfigured(false);
            setEndpoint("");
            setRegion("");
            setBucket("");
            setAccessKey("");
            setSecretKey("");
        } catch (e) {
            if (e instanceof ConnectError && e.code === Code.FailedPrecondition) {
                setError(t("storage.deleteBlocked"));
            } else if (e instanceof ConnectError) {
                setError(`${t("storage.deleteFail")} · ${e.message}`);
            } else {
                setError(String(e));
            }
        } finally {
            setBusy(false);
        }
    };

    if (!loaded) {
        return (
            <div className="max-w-3xl px-8 py-6 text-sm text-muted-foreground">
                {t("common.loading")}
            </div>
        );
    }

    const fields: Field[] = [
        {
            id: "endpoint",
            label: t("storage.endpoint"),
            desc: t("storage.endpointDesc"),
            value: endpoint,
            onChange: setEndpoint,
        },
        {
            id: "region",
            label: t("storage.region"),
            value: region,
            onChange: setRegion,
        },
        {
            id: "bucket",
            label: t("storage.bucket"),
            desc: t("storage.bucketDesc"),
            value: bucket,
            onChange: setBucket,
        },
        {
            id: "accessKey",
            label: t("storage.accessKey"),
            value: accessKey,
            onChange: setAccessKey,
            placeholder: configured ? t("storage.secretPlaceholder") : undefined,
            secret: true,
        },
        {
            id: "secretKey",
            label: t("storage.secretKey"),
            value: secretKey,
            onChange: setSecretKey,
            placeholder: configured ? t("storage.secretPlaceholder") : undefined,
            secret: true,
        },
    ];

    const pct =
        progress && progress.total > 0
            ? Math.min(100, Math.round((progress.done / progress.total) * 100))
            : 0;

    return (
        <div className="max-w-3xl px-8 py-6">
            <h2 className="text-base font-semibold">{t("storage.title")}</h2>
            <p className="mt-1 text-xs text-muted-foreground">
                {t("storage.desc")}
            </p>

            {pending && !migrating && (
                <div className="mt-4 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900">
                    <span>{t("storage.pendingBanner")}</span>
                    <Button
                        variant="outline"
                        size="sm"
                        className="ml-3"
                        onClick={() => void runMigration()}
                    >
                        {t("storage.resume")}
                    </Button>
                </div>
            )}

            <div className="mt-4">
                {fields.map((f) => (
                    <FieldRow
                        key={f.id}
                        field={f}
                        show={f.id === "accessKey" ? showAccess : showSecret}
                        onToggleShow={() =>
                            f.id === "accessKey"
                                ? setShowAccess(!showAccess)
                                : setShowSecret(!showSecret)
                        }
                    />
                ))}
            </div>

            <div className="mt-4 flex items-center gap-2">
                <Button
                    variant="outline"
                    size="sm"
                    onClick={handleTest}
                    disabled={busy}
                >
                    {t("storage.test")}
                </Button>
                <Button size="sm" onClick={handleSave} disabled={busy}>
                    {t("common.save")}
                </Button>
                {configured && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={handleDelete}
                        disabled={busy}
                    >
                        {t("common.delete")}
                    </Button>
                )}
            </div>

            {(progress || migrating) && (
                <div className="mt-4">
                    <div className="h-2 w-full overflow-hidden rounded-full bg-accent/60">
                        <div
                            className="h-full bg-foreground transition-all"
                            style={{ width: `${pct}%` }}
                        />
                    </div>
                    <p className="mt-1 text-xs text-muted-foreground">
                        {progress
                            ? t("storage.migrateResult", progress)
                            : t("storage.migrating")}
                    </p>
                </div>
            )}
            {migrateDone && (
                <p className="mt-2 text-sm text-foreground">
                    {t("storage.migrated")}
                </p>
            )}

            {error && (
                <p className="mt-4 text-xs text-destructive">{error}</p>
            )}
            {testMsg && (
                <p className="mt-2 text-xs text-muted-foreground">{testMsg}</p>
            )}
        </div>
    );
}
