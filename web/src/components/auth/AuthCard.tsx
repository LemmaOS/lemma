import { Bot } from "lucide-react";
import { type FormEvent, useState } from "react";
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
    const [loginPassword, setLoginPassword] = useState("");
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirm, setConfirm] = useState("");
    const [pending, setPending] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleLogin = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setPending(true);
        setError(null);
        try {
            await login(identifier.trim(), loginPassword);
        } catch {
            setError(t("auth.loginFailed"));
        } finally {
            setPending(false);
        }
    };

    const handleSignup = async (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (password !== confirm) {
            setError(t("auth.passwordMismatch"));
            return;
        }
        setPending(true);
        setError(null);
        try {
            await signup(username.trim(), email.trim(), password);
        } catch {
            setError(t("auth.signupFailed"));
        } finally {
            setPending(false);
        }
    };

    return (
        <Card className="w-full max-w-[380px]">
            <CardHeader className="flex flex-col items-center gap-2 text-center">
                <div className="grid size-9 place-items-center rounded-lg bg-primary text-primary-foreground">
                    <Bot className="size-5" />
                </div>
                <CardTitle className="text-lg">
                    {t("auth.productName")}
                </CardTitle>
                <CardDescription>{t("auth.subtitle")}</CardDescription>
            </CardHeader>
            <CardContent>
                <Tabs defaultValue="login" onValueChange={() => setError(null)}>
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
                                    value={identifier}
                                    onChange={(e) =>
                                        setIdentifier(e.target.value)
                                    }
                                    placeholder={t("auth.identifierPlaceholder")}
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
                                    value={loginPassword}
                                    onChange={(e) =>
                                        setLoginPassword(e.target.value)
                                    }
                                    placeholder={t("auth.passwordPlaceholder")}
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
                                disabled={pending}
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
                                    value={username}
                                    onChange={(e) =>
                                        setUsername(e.target.value)
                                    }
                                    placeholder={t("auth.usernamePlaceholder")}
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
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    placeholder={t("auth.emailPlaceholder")}
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
                                    value={password}
                                    onChange={(e) =>
                                        setPassword(e.target.value)
                                    }
                                    placeholder={t("auth.passwordPlaceholder")}
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
                                    value={confirm}
                                    onChange={(e) => setConfirm(e.target.value)}
                                    placeholder={t(
                                        "auth.confirmPasswordPlaceholder",
                                    )}
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
                                disabled={pending}
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
