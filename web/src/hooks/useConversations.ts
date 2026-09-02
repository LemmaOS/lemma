import { useEffect } from "react";

import { useConversationsStore } from "@/stores/conversations";

export function useConversations() {
    const store = useConversationsStore();
    const loaded = useConversationsStore((s) => s.loaded);
    const refresh = useConversationsStore((s) => s.refresh);
    useEffect(() => {
        if (!loaded) void refresh();
    }, [loaded, refresh]);
    return store;
}
