import { useChat as useChatStore } from "@/stores/chat";

// 直接透传：流式状态全在 store 里，hook 只是统一入口
export function useChat() {
    return useChatStore();
}
