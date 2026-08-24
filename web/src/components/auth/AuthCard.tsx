import { Bot } from "lucide-react";
import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";

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
import { useAuth } from "@/stores/auth";

export function AuthCard() {
    const { t } = useTranslation();
    const login = useAuth((s) => s.login);
    const signup = useAuth((s) => s.signup);

    const [identifier, setIdentifier] = useState("");
    const [password, setPassword] = useState("");
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [confirm, setConfirm] = useState("");
    const [error, setError] = useState("");
    const [busy, setBusy] = useState(false);

    const handleLogin = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setBusy(true);
        setError("");
        try {
            await login(identifier.trim(), password);
        } catch {
            setError(t("auth.loginFailed"));
        } finally {
            setBusy(false);
        }
    };

    const handleSignup = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (password !== confirm) {
            setError(t("auth.passwordMismatch"));
            return;
        }
        setBusy(true);
        setError("");
        try {
            await signup(username.trim(), email.trim(), password);
        } catch {
            setError(t("auth.signupFailed"));
        } finally {
            setBusy(false);
        }
    };

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
                            onSubmit={handleLogin}
                        >
                            <div className="grid gap-2">
                                <Label htmlFor="login-identifier">
                                    {t("auth.identifier")}
                                </Label>
                                <Input
                                    id="login-identifier"
                                    type="text"
                                    autoComplete="username"
                                    placeholder={t(
                                        "auth.identifierPlaceholder",
                                    )}
                                    value={identifier}
                                    onChange={(e) =>
                                        setIdentifier(e.target.value)
                                    }
                                    required
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
                                    value={password}
                                    onChange={(e) =>
                                        setPassword(e.target.value)
                                    }
                                    required
                                />
                            </div>
                            {error && (
                                <p className="text-xs text-destructive">
                                    {error}
                                </p>
                            )}
                            <Button
                                type="submit"
                                className="w-full"
                                disabled={busy}
                            >
                                {t("auth.signIn")}
                            </Button>
                        </form>
                    </TabsContent>

                    <TabsContent value="register">
                        <form
                            className="flex flex-col gap-4 pt-4"
                            onSubmit={handleSignup}
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
                                    value={username}
                                    onChange={(e) =>
                                        setUsername(e.target.value)
                                    }
                                    required
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
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    required
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
                                    value={password}
                                    onChange={(e) =>
                                        setPassword(e.target.value)
                                    }
                                    required
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
                                    value={confirm}
                                    onChange={(e) => setConfirm(e.target.value)}
                                    required
                                />
                            </div>
                            {error && (
                                <p className="text-xs text-destructive">
                                    {error}
                                </p>
                            )}
                            <Button
                                type="submit"
                                className="w-full"
                                disabled={busy}
                            >
                                {t("auth.createAccount")}
                            </Button>
                        </form>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}
