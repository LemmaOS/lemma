import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff } from "lucide-react";
import type { ProviderType } from "@/mocks";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";

export interface NewProviderData {
    name: string;
    type: ProviderType;
    baseUrl: string;
    apiKey: string;
}

interface NewProviderFormProps {
    onSave: (data: NewProviderData) => void;
    onCancel: () => void;
}

const PROVIDER_TYPES: ProviderType[] = ["openai", "anthropic", "gemini"];

export function NewProviderForm({ onSave, onCancel }: NewProviderFormProps) {
    const { t } = useTranslation();
    const [name, setName] = useState("");
    const [type, setType] = useState<ProviderType>("openai");
    const [baseUrl, setBaseUrl] = useState("");
    const [apiKey, setApiKey] = useState("");
    const [showKey, setShowKey] = useState(false);

    const handleSave = () => {
        onSave({
            name: name.trim() || type,
            type,
            baseUrl: baseUrl.trim(),
            apiKey: apiKey.trim(),
        });
    };

    return (
        <div className="max-w-3xl px-8 py-6">
            <h2 className="text-base font-semibold">
                {t("providers.newProvider")}
            </h2>
            <div className="mt-6 flex max-w-md flex-col gap-4">
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-type">{t("providers.type")}</Label>
                    <Select
                        value={type}
                        onValueChange={(v) => setType(v as ProviderType)}
                    >
                        <SelectTrigger id="np-type" className="w-full">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            {PROVIDER_TYPES.map((pt) => (
                                <SelectItem key={pt} value={pt}>
                                    {pt}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-name">{t("providers.name")}</Label>
                    <Input
                        id="np-name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder={t("providers.namePlaceholder")}
                    />
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-baseurl">{t("providers.baseUrl")}</Label>
                    <Input
                        id="np-baseurl"
                        value={baseUrl}
                        onChange={(e) => setBaseUrl(e.target.value)}
                        placeholder={t("providers.baseUrlPlaceholder")}
                    />
                </div>
                <div className="flex flex-col gap-1.5">
                    <Label htmlFor="np-apikey">{t("providers.apiKey")}</Label>
                    <div className="relative">
                        <Input
                            id="np-apikey"
                            type={showKey ? "text" : "password"}
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder={t("providers.apiKeyPlaceholder")}
                            className="pr-9"
                        />
                        <button
                            type="button"
                            onClick={() => setShowKey((v) => !v)}
                            aria-label={
                                showKey
                                    ? t("providers.hideApiKey")
                                    : t("providers.showApiKey")
                            }
                            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-foreground"
                        >
                            {showKey ? (
                                <EyeOff className="size-4" />
                            ) : (
                                <Eye className="size-4" />
                            )}
                        </button>
                    </div>
                </div>
                <div className="flex items-center gap-2 pt-2">
                    <Button type="button" onClick={handleSave}>
                        {t("common.save")}
                    </Button>
                    <Button type="button" variant="outline" onClick={onCancel}>
                        {t("common.cancel")}
                    </Button>
                </div>
            </div>
        </div>
    );
}
