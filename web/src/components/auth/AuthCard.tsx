import { useTranslation } from "react-i18next";
import { Bot } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export function AuthCard() {
    const { t } = useTranslation();

    return (
        <Card className="w-full max-w-[380px]">
            <CardHeader className="flex flex-col items-center gap-2 text-center">
                <div className="size-9 rounded-lg bg-primary text-primary-foreground grid place-items-center">
                    <Bot className="size-5" />
                </div>
                <CardTitle className="text-lg">
                    {t("auth.productName")}
                </CardTitle>
                <CardDescription>{t("auth.subtitle")}</CardDescription>
            </CardHeader>
            <CardContent>
                <Tabs defaultValue="login">
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="login">
                            {t("auth.loginTab")}
                        </TabsTrigger>
                        <TabsTrigger value="register">
                            {t("auth.registerTab")}
                        </TabsTrigger>
                    </TabsList>

                    <TabsContent value="login">
                        <form
                            className="flex flex-col gap-4 pt-4"
                            onSubmit={(event) => event.preventDefault()}
                        >
                            <div className="grid gap-2">
                                <Label htmlFor="login-email">
                                    {t("auth.email")}
                                </Label>
                                <Input
                                    id="login-email"
                                    type="email"
                                    autoComplete="email"
                                    placeholder={t("auth.emailPlaceholder")}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="login-password">
                                    {t("auth.password")}
                                </Label>
                                <Input
                                    id="login-password"
                                    type="password"
                                    autoComplete="current-password"
                                    placeholder={t("auth.passwordPlaceholder")}
                                />
                            </div>
                            <button
                                type="button"
                                className="self-start text-xs text-muted-foreground transition-colors hover:text-foreground"
                            >
                                {t("auth.forgotPassword")}
                            </button>
                            <Button type="submit" className="w-full">
                                {t("auth.signIn")}
                            </Button>
                        </form>
                    </TabsContent>

                    <TabsContent value="register">
                        <form
                            className="flex flex-col gap-4 pt-4"
                            onSubmit={(event) => event.preventDefault()}
                        >
                            <div className="grid gap-2">
                                <Label htmlFor="register-username">
                                    {t("auth.username")}
                                </Label>
                                <Input
                                    id="register-username"
                                    type="text"
                                    autoComplete="username"
                                    placeholder={t("auth.usernamePlaceholder")}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="register-email">
                                    {t("auth.email")}
                                </Label>
                                <Input
                                    id="register-email"
                                    type="email"
                                    autoComplete="email"
                                    placeholder={t("auth.emailPlaceholder")}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="register-password">
                                    {t("auth.password")}
                                </Label>
                                <Input
                                    id="register-password"
                                    type="password"
                                    autoComplete="new-password"
                                    placeholder={t("auth.passwordPlaceholder")}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="register-confirm">
                                    {t("auth.confirmPassword")}
                                </Label>
                                <Input
                                    id="register-confirm"
                                    type="password"
                                    autoComplete="new-password"
                                    placeholder={t(
                                        "auth.confirmPasswordPlaceholder",
                                    )}
                                />
                            </div>
                            <Button type="submit" className="w-full">
                                {t("auth.createAccount")}
                            </Button>
                        </form>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}
