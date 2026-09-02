import { create } from "zustand";

import type { Provider, ProviderKind } from "@/gen/lemma/v1/provider_pb";
import { providerClient } from "@/lib/clients";

export interface ProviderPatch {
    name?: string;
    baseUrl?: string;
    // Left empty to keep the current key. Never prefill the masked display
    // value: the backend would seal it as if it were the real key.
    apiKey?: string;
    enabled?: boolean;
    models?: string[];
    apiPath?: string;
    modelsPath?: string;
}

export interface NewProvider {
    kind: ProviderKind;
    name: string;
    baseUrl: string;
    apiKey: string;
    models: string[];
    apiPath?: string;
    modelsPath?: string;
}

interface ProvidersState {
    list: Provider[];
    loaded: boolean;
    refresh: () => Promise<void>;
    create: (input: NewProvider) => Promise<Provider>;
    update: (id: string, patch: ProviderPatch) => Promise<void>;
    remove: (id: string) => Promise<void>;
    // Accepts either a saved provider id or a draft's raw fields, so the
    // form can test credentials before anything is persisted.
    fetchModels: (req: {
        id?: string;
        kind?: ProviderKind;
        baseUrl?: string;
        apiKey?: string;
        modelsPath?: string;
    }) => Promise<string[]>;
}

export const useProvidersStore = create<ProvidersState>()((set) => ({
    list: [],
    loaded: false,

    refresh: async () => {
        const res = await providerClient.listProviders({});
        set({ list: res.providers, loaded: true });
    },

    create: async (input) => {
        const res = await providerClient.createProvider(input);
        if (!res.provider) throw new Error("no provider in response");
        set((s) => ({ list: [...s.list, res.provider!] }));
        return res.provider;
    },

    update: async (id, patch) => {
        const res = await providerClient.updateProvider({
            id,
            name: patch.name,
            baseUrl: patch.baseUrl,
            apiKey: patch.apiKey,
            enabled: patch.enabled,
            models: patch.models ? { models: patch.models } : undefined,
            apiPath: patch.apiPath,
            modelsPath: patch.modelsPath,
        });
        if (!res.provider) return;
        set((s) => ({
            list: s.list.map((p) => (p.id === id ? res.provider! : p)),
        }));
    },

    remove: async (id) => {
        await providerClient.deleteProvider({ id });
        set((s) => ({ list: s.list.filter((p) => p.id !== id) }));
    },

    fetchModels: async (req) => {
        const res = await providerClient.fetchModels(req);
        return res.models;
    },
}));
