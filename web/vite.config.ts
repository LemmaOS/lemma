import { codecovVitePlugin } from "@codecov/vite-plugin";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vitest/config";

export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
        // Uploads bundle stats to Codecov during `vite build`; without a
        // token (local builds) it stays a no-op.
        codecovVitePlugin({
            enableBundleAnalysis: process.env.CODECOV_TOKEN !== undefined,
            bundleName: "lemma-web",
            uploadToken: process.env.CODECOV_TOKEN,
        }),
    ],
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
        coverage: {
            provider: "v8",
            reporter: ["text", "lcov"],
            // Generated proto code and locale dictionaries carry no logic.
            exclude: ["src/gen/**", "src/i18n/locales/**"],
            thresholds: {
                lines: 90,
            },
        },
    },
});
