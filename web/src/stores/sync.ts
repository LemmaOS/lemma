import { create } from "zustand";

interface SyncStatusState {
    online: boolean;
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
