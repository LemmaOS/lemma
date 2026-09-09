import { app, BrowserWindow, ipcMain } from "electron";
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
        const url = new URL(SETUP_VITE_DEV_SERVER_URL);
        if (reason) url.searchParams.set("reason", reason);
        mainWindow.loadURL(url.toString());
    } else {
        mainWindow.loadFile(
            path.join(__dirname, `../renderer/${SETUP_VITE_NAME}/setup.html`),
            reason ? { query: { reason } } : undefined,
        );
    }
};

const loadApp = () => {
    if (!mainWindow) return;
    const serverUrl = getServerUrl();
    if (!serverUrl) {
        loadSetupPage();
        return;
    }
    // Failures surface through did-fail-load below.
    mainWindow.loadURL(serverUrl).catch(() => {});
};

const createWindow = () => {
    const window = new BrowserWindow({
        width: 1280,
        height: 800,
        webPreferences: {
            preload: path.join(__dirname, "preload.js"),
            additionalArguments: [`--lemma-server-url=${getServerUrl() ?? ""}`],
        },
    });
    mainWindow = window;
    window.on("closed", () => {
        mainWindow = null;
    });

    window.webContents.on(
        "did-fail-load",
        (_event, errorCode, _errorDescription, validatedURL) => {
            // ERR_ABORTED fires when a new navigation supersedes this one.
            if (errorCode === -3) return;
            if (validatedURL === getServerUrl()) {
                loadSetupPage(`Cannot reach ${validatedURL}`);
            }
        },
    );

    loadApp();
};

ipcMain.handle("get-server-url", () => getServerUrl());
ipcMain.handle("set-server-url", (_event, url: string) => {
    setServerUrl(url);
    loadApp();
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

    app.on("ready", createWindow);

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
