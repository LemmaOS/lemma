import { ArrowLeft, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";

import {
    ProviderForm,
    type ProviderFormValues,
} from "@/components/providers/ProviderForm";
import { ProviderListItem } from "@/components/providers/ProviderListItem";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Button } from "@/components/ui/button";
import type { Provider } from "@/gen/lemma/v1/provider_pb";
import { useProviders } from "@/hooks/useProviders";

export default function ProvidersPage() {
    const { t } = useTranslation();
    const store = useProviders();
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);

    // 未显式选中时默认第一个（渲染期派生，不用 effect 写回 state）
    const effectiveSelectedId =
        selectedId ?? (store.loaded ? (store.list[0]?.id ?? null) : null);
    const selected = store.list.find((p) => p.id === effectiveSelectedId);

    const handleSave = async (values: ProviderFormValues) => {
        if (creating) {
            const created = await store.create({
                kind: values.kind,
                name: values.name,
                baseUrl: values.baseUrl,
                apiKey: values.apiKey,
                models: values.models,
            });
            setSelectedId(created.id);
            setCreating(false);
            return;
        }
        if (!selected) return;
        // 只提交变化过的字段；apiKey 留空 = 不变更
        await store.update(selected.id, {
            name: values.name !== selected.name ? values.name : undefined,
            baseUrl:
                values.baseUrl !== selected.baseUrl
                    ? values.baseUrl
                    : undefined,
            apiKey: values.apiKey !== "" ? values.apiKey : undefined,
            models: values.models,
        });
    };

    // 已存供应商且密钥/地址没动 → 走 id（用服务端存的密钥）；否则用表单临时凭证
    const handleFetchModels = (provider?: Provider) => {
        return (values: {
            kind: Provider["kind"];
            baseUrl: string;
            apiKey: string;
        }) => {
            if (
                provider &&
                values.apiKey === "" &&
                values.baseUrl === provider.baseUrl
            ) {
                return store.fetchModels({ id: provider.id });
            }
            return store.fetchModels({
                kind: values.kind,
                baseUrl: values.baseUrl,
                apiKey: values.apiKey,
            });
        };
    };

    return (
        <div className="min-h-dvh bg-background">
            <header className="border-b border-border">
                <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-6">
                    <div className="flex items-center gap-3">
                        <Link
                            to="/"
                            className="flex items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
                        >
                            <ArrowLeft className="size-4" />
                            {t("common.back")}
                        </Link>
                        <h1 className="text-sm font-semibold">
                            {t("providers.title")}
                        </h1>
                    </div>
                    <ThemeToggle />
                </div>
            </header>

            <main className="mx-auto flex max-w-5xl items-start gap-8 px-6 py-8">
                <aside className="w-[240px] shrink-0">
                    <Button
                        type="button"
                        variant="outline"
                        className="w-full"
                        onClick={() => setCreating(true)}
                    >
                        <Plus className="size-4" />
                        {t("providers.newProvider")}
                    </Button>
                    <nav className="mt-4 flex flex-col gap-1">
                        {store.list.map((provider) => (
                            <ProviderListItem
                                key={provider.id}
                                provider={provider}
                                selected={
                                    !creating &&
                                    provider.id === effectiveSelectedId
                                }
                                onSelect={() => {
                                    setCreating(false);
                                    setSelectedId(provider.id);
                                }}
                            />
                        ))}
                        {store.loaded && store.list.length === 0 && (
                            <p className="px-3 py-2 text-xs text-muted-foreground">
                                {t("providers.emptyList")}
                            </p>
                        )}
                    </nav>
                </aside>

                <section className="min-w-0 flex-1">
                    {creating ? (
                        <ProviderForm
                            key="new"
                            onSave={handleSave}
                            onCancel={() => setCreating(false)}
                            onFetchModels={handleFetchModels()}
                        />
                    ) : (
                        selected && (
                            <ProviderForm
                                key={selected.id}
                                provider={selected}
                                onSave={handleSave}
                                onDelete={async () => {
                                    await store.remove(selected.id);
                                    setSelectedId(null);
                                }}
                                onFetchModels={handleFetchModels(selected)}
                            />
                        )
                    )}
                </section>
            </main>
        </div>
    );
}
