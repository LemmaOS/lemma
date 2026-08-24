import { create } from "zustand";

interface SyncStatusState {
    /** watch 流是否在线（离线时 UI 禁用发送） */
    online: boolean;
    /** 是否正在补拉 */
    syncing: boolean;
    setOnline: (v: boolean) => void;
    setSyncing: (v: boolean) => void;
}

export const useSyncStatus = create<SyncStatusState>()((set) => ({
    online: true,
    syncing: false,
    setOnline: (online) => set({ online }),
    setSyncing: (syncing) => set({ syncing }),
}));
