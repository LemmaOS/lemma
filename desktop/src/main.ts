import { app, BrowserWindow } from "electron";
import started from "electron-squirrel-startup";
import path from "node:path";

// Quit when launched by the Squirrel installer/updater hooks.
if (started) {
    app.quit();
}

const SERVER_URL = "http://127.0.0.1:1025";

const createWindow = () => {
    const mainWindow = new BrowserWindow({
        width: 1280,
        height: 800,
        webPreferences: {
            preload: path.join(__dirname, "preload.js"),
        },
    });

    mainWindow.loadURL(SERVER_URL);
};

const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
    app.quit();
} else {
    app.on("second-instance", () => {
        const mainWindow = BrowserWindow.getAllWindows()[0];
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
