import { useState } from "react";
import type { Provider, ProviderModel } from "@/mocks";
import { mockProviderModels, mockProviders } from "@/mocks";
import {
    SettingsNav,
    type SettingsSection,
} from "@/components/providers/SettingsNav";
import { AppearancePanel } from "@/components/providers/AppearancePanel";
import { ProviderListPane } from "@/components/providers/ProviderListPane";
import { ProviderDetail } from "@/components/providers/ProviderDetail";
import {
    NewProviderForm,
    type NewProviderData,
} from "@/components/providers/NewProviderForm";

export default function ProvidersPage() {
    const [section, setSection] = useState<SettingsSection>("providers");
    const [providers, setProviders] = useState<Provider[]>(mockProviders);
    const [selectedId, setSelectedId] = useState<string | null>(
        mockProviders[0]?.id ?? null,
    );
    const [creating, setCreating] = useState(false);
    const [modelsByProvider, setModelsByProvider] =
        useState<Record<string, ProviderModel[]>>(mockProviderModels);

    const selected = providers.find((p) => p.id === selectedId) ?? null;

    const handleSelect = (id: string) => {
        setCreating(false);
        setSelectedId(id);
    };

    const handleCreate = () => {
        setCreating(true);
    };

    const handleSaveNew = (data: NewProviderData) => {
        const provider: Provider = {
            id: `p-${Date.now()}`,
            name: data.name,
            type: data.type,
            baseUrl: data.baseUrl,
            apiKeyMasked: data.apiKey,
            models: [],
            configured: Boolean(data.apiKey),
            enabled: false,
        };
        setProviders((current) => [...current, provider]);
        setSelectedId(provider.id);
        setCreating(false);
    };

    const handleToggleEnabled = (id: string, enabled: boolean) => {
        setProviders((current) =>
            current.map((p) => (p.id === id ? { ...p, enabled } : p)),
        );
    };

    const handleModelsChange = (
        providerId: string,
        models: ProviderModel[],
    ) => {
        setModelsByProvider((current) => ({
            ...current,
            [providerId]: models,
        }));
    };

    return (
        <div className="flex h-dvh gap-2 bg-sidebar p-2 text-foreground">
            <SettingsNav section={section} onSelect={setSection} />
            {section === "appearance" ? (
                <main className="min-w-0 flex-1 overflow-y-auto rounded-xl border border-border bg-background">
                    <AppearancePanel />
                </main>
            ) : (
                <>
                    <ProviderListPane
                        providers={providers}
                        selectedId={creating ? null : selectedId}
                        onSelect={handleSelect}
                        onCreate={handleCreate}
                    />
                    <main className="min-w-0 flex-1 overflow-y-auto rounded-xl border border-border bg-background">
                        {creating ? (
                            <NewProviderForm
                                onSave={handleSaveNew}
                                onCancel={() => setCreating(false)}
                            />
                        ) : (
                            selected && (
                                <ProviderDetail
                                    key={selected.id}
                                    provider={selected}
                                    models={modelsByProvider[selected.id] ?? []}
                                    onToggleEnabled={(enabled) =>
                                        handleToggleEnabled(
                                            selected.id,
                                            enabled,
                                        )
                                    }
                                    onModelsChange={(models) =>
                                        handleModelsChange(selected.id, models)
                                    }
                                />
                            )
                        )}
                    </main>
                </>
            )}
        </div>
    );
}
