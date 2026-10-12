# Custom Utility App

Original filesystem scanner dashboard. The Go process walks directories in chunks and checks each file through an injected function. The Electron window only displays status. Nothing in this app is copied from CleanMyMac, and nothing deletes files.

## Layout

- `backend/` — Go engine (`main.go`, `scanner.go`, `ipc.go`)
- `frontend/` — Electron + TypeScript dashboard

## Pipeline

1. The UI sends newline-delimited JSON on the engine stdin: `start` (with `root`), `pause`, `resume`, `cancel`.
2. The walker groups regular files into chunks of 64.
3. Worker goroutines call `checkURL` (`PlaceholderCheck` today, always “not a threat”).
4. A true result is appended to `RemovalQueue` and emitted as a `threat` event. The queue is not a deleter.
5. Progress events carry files scanned, files per second, and threat count.

Pause blocks workers on a gate. Cancel cancels the scan context. The walker keeps draining so a cancel cannot deadlock the pipeline.

## Prerequisites

- Go 1.22 or newer
- Node.js 20 or newer

## Build and run

```bash
cd backend
go build -o custom-utility-engine .

cd ../frontend
npm install
npm start
```

`npm start` compiles TypeScript and launches Electron. The main process spawns `backend/custom-utility-engine`. Override the binary with `ENGINE_BIN`.

## Engine protocol

Stdin:

```json
{"cmd":"start","root":"/path/to/folder"}
{"cmd":"pause"}
{"cmd":"resume"}
{"cmd":"cancel"}
```

Stdout (one JSON object per line): `ready`, `progress`, `threat`, `done`, `error`.

## Replacing detection

`PlaceholderCheck` in `backend/scanner.go` is the stand-in for a detection service. Change that function, or pass another `Checker` into `Scan`. Do not delete files from the checker. Flagged paths belong on `RemovalQueue` until a separate, explicit removal step exists.
