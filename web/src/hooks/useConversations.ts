import { useEffect } from "react";

import { useConversationsStore } from "@/stores/conversations";

export function useConversations() {
    const store = useConversationsStore();
    useEffect(() => {
        if (!store.loaded) void store.refresh();
    }, [store.loaded]);
    return store;
}
