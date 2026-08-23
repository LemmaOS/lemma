import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vitest/config";

export default defineConfig({
    plugins: [react(), tailwindcss()],
    resolve: {
        alias: { "@": path.resolve(import.meta.dirname, "src") },
    },
    server: {
        proxy: {
            "/lemma.v1.": "http://127.0.0.1:1025",
        },
    },
    test: {
        environment: "node",
        include: ["src/**/*.test.ts"],
    },
});
