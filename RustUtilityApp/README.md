# Rust Utility

Pure-Rust desktop scanner. The engine walks a directory in chunks, checks each file through a detector trait, and reports progress. The window is egui. Nothing in this workspace deletes files or ships malware signatures.

The control flow is a clean-room of the scan pipeline notes under `../CleanMyMac_RE/` (enumerate, chunk, check, separate removal plan).

## Install

GitHub Releases is the download. Open the `.dmg` for your Mac, then drag **Sable** onto **Applications**.

The build is not notarized, so the first open is blocked until you allow it:

1. Control-click **Sable** in Applications and choose **Open**.
2. Confirm **Open** in the dialog. Later launches are normal.

Apple Silicon downloads are named `arm64`. Intel downloads are named `x64`. The GitHub Action builds on Apple Silicon.

### Publish a release

This folder has to be the root of its own GitHub repository. Do not publish the parent experiment tree.

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds `Sable-0.1.0-macos-arm64.dmg` and a zip, then attaches both to that tag.

A local installer, without GitHub:

```bash
make package
open dist/Sable-*.dmg
```

`make install` copies `Sable.app` into `/Applications`. `PREFIX=$HOME/Applications make install` uses another folder.

## Requirements

- Rust 1.80 or newer (`rustup update stable`)
- A desktop session (the `ui` crate opens a native window)

## Build and run

From this directory, `make help` lists the targets.

```bash
make test
make run
make release
make package
```

`cargo test -p engine` covers cancel, chunked walk, an extension watch hit, and a removal plan that does not unlink.

`cargo run -p ui` opens the Sable window. The sidebar switches features:

Each screen opens with a plain-language explanation and a badge: read-only, moves to Trash (undoable), or may ask for a password. Screens fill themselves on first open. Anything that moves a file shows a confirmation first.

- Overview: how much space you can free, app count, startup items, disk access
- Free Up Space: caches and logs, sized, each group can go to the Trash
- Apps: searchable list; click an app to see files it left in your Library
- Tune-Up: startup items, flush DNS, check or rebuild Spotlight
- File Finder: pick a folder and file types (installers, archives...); lists matches, never changes files
- Big Downloads: files of 50 MB or more in Downloads, one at a time to the Trash
- Disk Usage: folder sizes with drill-down (Open / Up one level)
- Cloud Folders: iCloud and CloudStorage sizes
- Live Stats: CPU and memory with a plain verdict

Hits and surveys do not unlink files. Trash goes through Finder and only accepts the surveyed cache or log roots. Spotlight reindex may ask for an administrator.

## Layout

```text
crates/engine   walk, execution control, detector, scan task, removal plan
crates/ui       eframe shell
```

Swap the UI later without changing `engine`. Slint can replace `crates/ui`. Tauri is not used here because the UI is Rust.
