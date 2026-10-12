export {};

interface EngineEvent {
  event: string;
  filesScanned?: number;
  filesPerSecond?: number;
  threatCount?: number;
  path?: string;
  message?: string;
  paused?: boolean;
}

interface EngineApi {
  startScan: (root: string) => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  cancel: () => Promise<void>;
  onStatus: (callback: (event: EngineEvent) => void) => () => void;
}

declare global {
  interface Window {
    engine: EngineApi;
  }
}

const filesEl = document.querySelector("#files") as HTMLElement;
const rateEl = document.querySelector("#rate") as HTMLElement;
const threatsEl = document.querySelector("#threats") as HTMLElement;
const statusEl = document.querySelector("#status") as HTMLElement;
const listEl = document.querySelector("#list") as HTMLUListElement;
const rootEl = document.querySelector("#root") as HTMLInputElement;

function setStatus(text: string): void {
  statusEl.textContent = text;
}

window.engine.onStatus((event) => {
  if (typeof event.filesScanned === "number") {
    filesEl.textContent = String(event.filesScanned);
  }
  if (typeof event.filesPerSecond === "number") {
    rateEl.textContent = event.filesPerSecond.toFixed(1);
  }
  if (typeof event.threatCount === "number") {
    threatsEl.textContent = String(event.threatCount);
  }

  if (event.event === "ready") {
    setStatus("Engine online. Choose a folder and start.");
  } else if (event.event === "threat" && event.path) {
    const item = document.createElement("li");
    item.textContent = event.path;
    listEl.prepend(item);
    setStatus("Threat flagged (queued, not deleted).");
  } else if (event.event === "error") {
    setStatus(event.message ?? "Engine error");
  } else if (event.event === "done") {
    setStatus(event.message === "canceled" ? "Scan canceled." : "Scan finished.");
  } else if (event.paused) {
    setStatus("Paused.");
  } else if (event.message === "resumed") {
    setStatus("Scanning.");
  } else if (event.message === "started") {
    listEl.replaceChildren();
    setStatus("Scanning.");
  }
});

document.querySelector("#start")?.addEventListener("click", () => {
  const root = rootEl.value.trim();
  if (!root) {
    setStatus("Enter a folder path.");
    return;
  }
  void window.engine.startScan(root);
});

document.querySelector("#pause")?.addEventListener("click", () => {
  void window.engine.pause();
});

document.querySelector("#resume")?.addEventListener("click", () => {
  void window.engine.resume();
});

document.querySelector("#cancel")?.addEventListener("click", () => {
  void window.engine.cancel();
});
