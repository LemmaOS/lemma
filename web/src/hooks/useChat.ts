import { useChat as useChatStore } from "@/stores/chat";

export function useChat() {
    return useChatStore();
}
