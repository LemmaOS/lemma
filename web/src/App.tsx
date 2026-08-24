import { BrowserRouter, Route, Routes } from "react-router";
import { TooltipProvider } from "@/components/ui/tooltip";
import ChatPage from "@/pages/ChatPage";
import LoginPage from "@/pages/LoginPage";
import ProvidersPage from "@/pages/ProvidersPage";

export default function App() {
    return (
        <TooltipProvider>
            <BrowserRouter>
                <Routes>
                    <Route path="/" element={<ChatPage />} />
                    <Route path="/login" element={<LoginPage />} />
                    <Route
                        path="/settings/providers"
                        element={<ProvidersPage />}
                    />
                </Routes>
            </BrowserRouter>
        </TooltipProvider>
    );
}
