import { WifiOff } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useSyncStatus } from "@/stores/sync";

// 断线时提示"展示的是本地缓存"；在线（含同步中）不渲染
export function SyncIndicator() {
    const { t } = useTranslation();
    const online = useSyncStatus((s) => s.online);
    if (online) return null;
    return (
        <span className="flex items-center gap-1 text-xs text-muted-foreground">
            <WifiOff className="size-3.5" />
            {t("sync.offline")}
        </span>
    );
}
