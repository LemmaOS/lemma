import { useEffect } from "react";

import { useProvidersStore } from "@/stores/providers";

export function useProviders() {
    const store = useProvidersStore();
    const loaded = useProvidersStore((s) => s.loaded);
    const refresh = useProvidersStore((s) => s.refresh);
    useEffect(() => {
        if (!loaded) void refresh();
    }, [loaded, refresh]);
    return store;
}
