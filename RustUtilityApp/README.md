# Rust Utility

Pure-Rust desktop scanner. The engine walks a directory in chunks, checks each file through a detector trait, and reports progress. The window is egui. Nothing in this workspace deletes files or ships malware signatures.

The control flow is a clean-room of the scan pipeline notes under `../CleanMyMac_RE/` (enumerate, chunk, check, separate removal plan).

## Requirements

- Rust 1.80 or newer (`rustup update stable`)
- A desktop session (the `ui` crate opens a native window)

## Build and run

From this directory:

```bash
cargo test -p engine
cargo run -p ui
```

`cargo test -p engine` covers cancel, chunked walk, an extension watch hit, and a removal plan that does not unlink.

`cargo run -p ui` opens the Sable window. The sidebar switches features:

- Smart Care runs cleanup, access, launch agents, and applications together
- Protection walks a folder and flags watched extensions
- Cleanup measures cache and log roots and can move one of those roots to the Trash
- Performance lists LaunchAgents, flushes the user DNS cache, and asks Spotlight for status or a reindex
- Applications lists bundle names and identifiers
- Space sizes the children of a folder

Hits and surveys do not unlink files. Trash goes through Finder and only accepts the surveyed cache or log roots. Spotlight reindex may ask for an administrator.

## Layout

```text
crates/engine   walk, execution control, detector, scan task, removal plan
crates/ui       eframe shell
```

Swap the UI later without changing `engine`. Slint can replace `crates/ui`. Tauri is not used here because the UI is Rust.
