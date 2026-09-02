import { WifiOff } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useSyncStatus } from "@/stores/sync";

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
