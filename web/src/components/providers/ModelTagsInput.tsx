import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface ModelTagsInputProps {
    models: string[];
    onChange: (models: string[]) => void;
}

export function ModelTagsInput({ models, onChange }: ModelTagsInputProps) {
    const { t } = useTranslation();
    const [draft, setDraft] = useState("");

    const addModel = () => {
        const value = draft.trim();
        if (!value) return;
        if (!models.includes(value)) {
            onChange([...models, value]);
        }
        setDraft("");
    };

    const removeModel = (model: string) => {
        onChange(models.filter((item) => item !== model));
    };

    return (
        <div className="grid gap-2">
            <Label htmlFor="provider-model-input">
                {t("providers.models")}
            </Label>
            {models.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                    {models.map((model) => (
                        <Badge
                            key={model}
                            variant="secondary"
                            className="gap-1 pr-1"
                        >
                            <span className="font-mono text-xs">{model}</span>
                            <button
                                type="button"
                                aria-label={t("providers.removeModel")}
                                onClick={() => removeModel(model)}
                                className="rounded-sm text-muted-foreground transition-colors hover:text-foreground"
                            >
                                <X className="size-3" />
                            </button>
                        </Badge>
                    ))}
                </div>
            )}
            <div className="flex items-center gap-2">
                <Input
                    id="provider-model-input"
                    value={draft}
                    placeholder={t("providers.modelPlaceholder")}
                    onChange={(event) => setDraft(event.target.value)}
                    onKeyDown={(event) => {
                        if (event.key === "Enter") {
                            event.preventDefault();
                            addModel();
                        }
                    }}
                />
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addModel}
                    disabled={!draft.trim()}
                >
                    <Plus className="size-4" />
                    {t("common.add")}
                </Button>
            </div>
        </div>
    );
}
