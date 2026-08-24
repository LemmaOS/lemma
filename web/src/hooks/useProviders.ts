import { useEffect } from "react";

import { useProvidersStore } from "@/stores/providers";

export function useProviders() {
    const store = useProvidersStore();
    useEffect(() => {
        if (!store.loaded) void store.refresh();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [store.loaded]);
    return store;
}
