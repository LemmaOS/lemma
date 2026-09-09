import { contextBridge, ipcRenderer } from "electron";

const serverUrl = ipcRenderer.sendSync("get-server-url-sync");

contextBridge.exposeInMainWorld("__LEMMA_SERVER_URL__", serverUrl);
contextBridge.exposeInMainWorld("lemmaDesktop", {
    getServerUrl: (): Promise<string | undefined> =>
        ipcRenderer.invoke("get-server-url"),
    setServerUrl: (url: string): Promise<void> =>
        ipcRenderer.invoke("set-server-url", url),
});
