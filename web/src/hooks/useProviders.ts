import { useEffect } from "react";

import { useProvidersStore } from "@/stores/providers";

export function useProviders() {
    const store = useProvidersStore();
    useEffect(() => {
        if (!store.loaded) void store.refresh();
    }, [store.loaded]);
    return store;
}
