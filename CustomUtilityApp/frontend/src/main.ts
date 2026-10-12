import { app, BrowserWindow, ipcMain } from "electron";
import { ChildProcessWithoutNullStreams, spawn } from "child_process";
import path from "path";
import readline from "readline";

interface EngineEvent {
  event: string;
  filesScanned?: number;
  filesPerSecond?: number;
  threatCount?: number;
  path?: string;
  message?: string;
  paused?: boolean;
}

let mainWindow: BrowserWindow | null = null;
let engine: ChildProcessWithoutNullStreams | null = null;

function engineBinary(): string {
  if (process.env.ENGINE_BIN) {
    return process.env.ENGINE_BIN;
  }
  return path.resolve(__dirname, "../../backend/custom-utility-engine");
}

function sendStatus(payload: EngineEvent): void {
  mainWindow?.webContents.send("engine:status", payload);
}

function writeCommand(cmd: { cmd: string; root?: string }): void {
  if (!engine || !engine.stdin.writable) {
    sendStatus({ event: "error", message: "engine is not running" });
    return;
  }
  engine.stdin.write(JSON.stringify(cmd) + "\n");
}

function startEngine(): void {
  if (engine) {
    return;
  }
  const bin = engineBinary();
  engine = spawn(bin, [], { stdio: ["pipe", "pipe", "pipe"] });

  const out = readline.createInterface({ input: engine.stdout });
  out.on("line", (line) => {
    try {
      sendStatus(JSON.parse(line) as EngineEvent);
    } catch {
      sendStatus({ event: "error", message: "bad engine json" });
    }
  });

  engine.stderr.on("data", (chunk: Buffer) => {
    const text = chunk.toString().trim();
    if (text) {
      sendStatus({ event: "error", message: text });
    }
  });

  engine.on("exit", (code) => {
    sendStatus({ event: "error", message: `engine exited (${code ?? "null"})` });
    engine = null;
  });

  engine.on("error", (err) => {
    sendStatus({
      event: "error",
      message: `failed to start engine at ${bin}: ${err.message}`,
    });
    engine = null;
  });
}

function stopEngine(): void {
  if (!engine) {
    return;
  }
  engine.kill();
  engine = null;
}

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 880,
    height: 640,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  mainWindow.loadFile(path.join(__dirname, "../src/renderer/index.html"));
  mainWindow.on("closed", () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  startEngine();
  createWindow();
});

app.on("window-all-closed", () => {
  stopEngine();
  app.quit();
});

ipcMain.handle("engine:start", (_event, root: string) => {
  writeCommand({ cmd: "start", root });
});

ipcMain.handle("engine:pause", () => {
  writeCommand({ cmd: "pause" });
});

ipcMain.handle("engine:resume", () => {
  writeCommand({ cmd: "resume" });
});

ipcMain.handle("engine:cancel", () => {
  writeCommand({ cmd: "cancel" });
});
