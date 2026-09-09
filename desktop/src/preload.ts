import { contextBridge, ipcRenderer } from "electron";

const serverUrlArg = process.argv.find((arg) =>
    arg.startsWith("--lemma-server-url="),
);
const serverUrl = serverUrlArg
    ? serverUrlArg.slice("--lemma-server-url=".length)
    : "";

contextBridge.exposeInMainWorld("__LEMMA_SERVER_URL__", serverUrl);
contextBridge.exposeInMainWorld("lemmaDesktop", {
    getServerUrl: (): Promise<string | undefined> =>
        ipcRenderer.invoke("get-server-url"),
    setServerUrl: (url: string): Promise<void> =>
        ipcRenderer.invoke("set-server-url", url),
});
