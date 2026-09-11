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
