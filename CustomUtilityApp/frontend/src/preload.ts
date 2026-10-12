import { contextBridge, ipcRenderer } from "electron";

export interface EngineEvent {
  event: string;
  filesScanned?: number;
  filesPerSecond?: number;
  threatCount?: number;
  path?: string;
  message?: string;
  paused?: boolean;
}

// Narrow API for the renderer. The page cannot require Node or spawn processes.
contextBridge.exposeInMainWorld("engine", {
  startScan: (root: string): Promise<void> => ipcRenderer.invoke("engine:start", root),
  pause: (): Promise<void> => ipcRenderer.invoke("engine:pause"),
  resume: (): Promise<void> => ipcRenderer.invoke("engine:resume"),
  cancel: (): Promise<void> => ipcRenderer.invoke("engine:cancel"),
  onStatus: (callback: (event: EngineEvent) => void): (() => void) => {
    const listener = (_event: Electron.IpcRendererEvent, payload: EngineEvent) => {
      callback(payload);
    };
    ipcRenderer.on("engine:status", listener);
    return () => ipcRenderer.removeListener("engine:status", listener);
  },
});
