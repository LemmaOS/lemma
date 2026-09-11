import { app, BrowserWindow, ipcMain, Menu, nativeTheme } from "electron";
import started from "electron-squirrel-startup";
import path from "node:path";
import { getServerUrl, setServerUrl } from "./settings";

// Quit when launched by the Squirrel installer/updater hooks.
if (started) {
    app.quit();
}

let mainWindow: BrowserWindow | null = null;

const loadSetupPage = (reason?: string) => {
    if (!mainWindow) return;
    if (SETUP_VITE_DEV_SERVER_URL) {
        // The define is the dev server's bare origin; the setup page lives
        // at /setup.html (no index.html exists at the project root).
        const url = new URL(SETUP_VITE_DEV_SERVER_URL);
        url.pathname = "/setup.html";
        if (reason) url.searchParams.set("reason", reason);
        mainWindow.loadURL(url.toString());
    } else {
        mainWindow.loadFile(
            path.join(__dirname, `../renderer/${SETUP_VITE_NAME}/setup.html`),
            reason ? { query: { reason } } : undefined,
        );
    }
};

const SETUP_LOAD_RETRIES = 3;
let setupLoadAttempts = 0;

const loadFatalPage = (detail: string) => {
    if (!mainWindow) return;
    const html = `<!doctype html><html><head><meta charset="utf-8"><title>Lemma</title></head>
<body style="margin:0;min-height:100vh;display:flex;align-items:center;justify-content:center;background:#151615;color:#e6e6e4;font:14px system-ui,-apple-system,'Segoe UI',sans-serif;-webkit-app-region:drag">
<main style="max-width:420px;padding:32px;-webkit-app-region:no-drag">
<h1 style="margin:0 0 12px;font-size:20px">Lemma failed to load its interface</h1>
<p style="margin:0 0 8px;color:#8f918c">${detail}</p>
<p style="margin:0;color:#8f918c">Fully quit the app and start it again. In a dev session, also check for a leftover dev-server process still holding the renderer port.</p>
</main></body></html>`;
    mainWindow.loadURL(
        `data:text/html;charset=utf-8,${encodeURIComponent(html)}`,
    );
};

const loadApp = () => {
    if (!mainWindow) return;
    const serverUrl = getServerUrl();
    if (!serverUrl) {
        loadSetupPage();
        return;
    }
    if (app.isPackaged) {
        mainWindow.loadFile(path.join(__dirname, "../../web-dist/index.html"));
    } else {
        // Dev mode loads the app from the server itself; failures surface
        // through did-fail-load below.
        mainWindow.loadURL(serverUrl).catch(() => {});
    }
};

const createWindow = () => {
    const window = new BrowserWindow({
        width: 1280,
        height: 800,
        backgroundColor: "#151615",
        titleBarStyle: "hidden",
        titleBarOverlay: {
            color: nativeTheme.shouldUseDarkColors ? "#151615" : "#ffffff",
            symbolColor: nativeTheme.shouldUseDarkColors
                ? "#e6e6e4"
                : "#1f1f1f",
            height: 40,
        },
        webPreferences: {
            preload: path.join(__dirname, "preload.js"),
        },
    });
    mainWindow = window;
    window.on("closed", () => {
        mainWindow = null;
    });

    window.webContents.on("before-input-event", (_event, input) => {
        if (input.type !== "keyDown") return;
        const devtoolsKey =
            input.key === "F12" ||
            ((input.control || input.meta) &&
                input.shift &&
                input.key.toLowerCase() === "i");
        if (devtoolsKey) {
            window.webContents.toggleDevTools();
        }
    });

    window.webContents.on("did-finish-load", () => {
        setupLoadAttempts = 0;
    });

    window.webContents.on(
        "did-fail-load",
        (_event, errorCode, errorDescription, validatedURL, isMainFrame) => {
            if (!isMainFrame) return;
            // ERR_ABORTED fires when a new navigation supersedes this one.
            if (errorCode === -3) return;
            if (validatedURL.startsWith("data:")) return;
            if (validatedURL === getServerUrl()) {
                setupLoadAttempts = 0;
                loadSetupPage(`Cannot reach ${validatedURL}`);
                return;
            }
            // The shell UI itself failed to load (in dev, usually a stale
            // dev-server process holding the renderer port). Retry a few
            // times, then show an error page instead of a black window.
            setupLoadAttempts += 1;
            if (setupLoadAttempts <= SETUP_LOAD_RETRIES) {
                setTimeout(() => loadSetupPage(), 1000);
            } else {
                loadFatalPage(`${errorDescription} (${errorCode})`);
            }
        },
    );

    loadApp();
};

ipcMain.handle("get-server-url", () => getServerUrl());
ipcMain.handle("set-server-url", (_event, url: string) => {
    setServerUrl(url);
    setupLoadAttempts = 0;
    loadApp();
});
ipcMain.on("get-server-url-sync", (event) => {
    event.returnValue = getServerUrl() ?? "";
});
ipcMain.on("toggle-maximize", () => {
    if (!mainWindow) return;
    if (mainWindow.isMaximized()) {
        mainWindow.unmaximize();
    } else {
        mainWindow.maximize();
    }
});

const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
    app.quit();
} else {
    app.on("second-instance", () => {
        if (mainWindow) {
            if (mainWindow.isMinimized()) {
                mainWindow.restore();
            }
            mainWindow.focus();
        }
    });

    app.on("ready", () => {
        Menu.setApplicationMenu(null);
        createWindow();
    });

    app.on("window-all-closed", () => {
        if (process.platform !== "darwin") {
            app.quit();
        }
    });

    app.on("activate", () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow();
        }
    });
}
