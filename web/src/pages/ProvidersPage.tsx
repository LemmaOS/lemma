import { useState } from "react";
import { useTranslation } from "react-i18next";

import { AppearancePanel } from "@/components/providers/AppearancePanel";
import {
    NewProviderForm,
    type NewProviderData,
} from "@/components/providers/NewProviderForm";
import { ProviderDetail } from "@/components/providers/ProviderDetail";
import { ProviderListPane } from "@/components/providers/ProviderListPane";
import { StoragePanel } from "@/components/providers/StoragePanel";
import {
    SettingsNav,
    type SettingsSection,
} from "@/components/providers/SettingsNav";
import { useProviders } from "@/hooks/useProviders";

export default function ProvidersPage() {
    const { t } = useTranslation();
    const [section, setSection] = useState<SettingsSection>("providers");
    const store = useProviders();
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);

    // 未手选时默认选中第一个（派生值，避免 effect 里 setState）
    const effectiveSelectedId =
        selectedId ?? (store.loaded ? (store.list[0]?.id ?? null) : null);
    const selected =
        store.list.find((p) => p.id === effectiveSelectedId) ?? null;

    const handleSaveNew = async (data: NewProviderData) => {
        const provider = await store.create({
            kind: data.kind,
            name: data.name,
            baseUrl: data.baseUrl,
            apiKey: data.apiKey,
            models: [],
        });
        setSelectedId(provider.id);
        setCreating(false);
    };

    const handleDelete = async (id: string) => {
        await store.remove(id);
        setSelectedId(null);
    };

    return (
        <div className="flex h-dvh gap-2 bg-sidebar p-2 text-foreground">
            <SettingsNav section={section} onSelect={setSection} />
            {section === "appearance" ? (
                <main className="min-w-0 flex-1 overflow-y-auto rounded-lg border border-border app-canvas">
                    <AppearancePanel />
                </main>
            ) : section === "storage" ? (
                <main className="min-w-0 flex-1 overflow-y-auto rounded-lg border border-border app-canvas">
                    <StoragePanel />
                </main>
            ) : (
                <>
                    <ProviderListPane
                        providers={store.list}
                        selectedId={creating ? null : effectiveSelectedId}
                        onSelect={(id) => {
                            setCreating(false);
                            setSelectedId(id);
                        }}
                        onCreate={() => setCreating(true)}
                    />
                    <main className="min-w-0 flex-1 overflow-y-auto rounded-lg border border-border app-canvas">
                        {creating ? (
                            <NewProviderForm
                                onSave={handleSaveNew}
                                onCancel={() => setCreating(false)}
                            />
                        ) : selected ? (
                            <ProviderDetail
                                key={selected.id}
                                provider={selected}
                                onToggleEnabled={(enabled) =>
                                    void store.update(selected.id, { enabled })
                                }
                                onSaveBaseUrl={(baseUrl) =>
                                    store.update(selected.id, { baseUrl })
                                }
                                onSaveApiKey={(apiKey) =>
                                    store.update(selected.id, { apiKey })
                                }
                                onModelsChange={(models) =>
                                    void store.update(selected.id, { models })
                                }
                                onFetchModels={() =>
                                    store.fetchModels({ id: selected.id })
                                }
                                onDelete={() => handleDelete(selected.id)}
                            />
                        ) : (
                            <div className="grid h-full place-items-center text-sm text-muted-foreground">
                                {store.loaded
                                    ? t("providers.emptyList")
                                    : t("common.loading")}
                            </div>
                        )}
                    </main>
                </>
            )}
        </div>
    );
}
