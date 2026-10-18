# Sable

A Mac app that shows what uses your disk and moves caches, leftovers, and large downloads to the Trash only after you confirm.

## Install

1. Open the [latest release](https://github.com/h3ma209/Sable/releases/latest).
2. Download the disk image.
   - Apple Silicon: `Sable-*-macos-arm64.dmg`
   - Intel: `Sable-*-macos-x64.dmg`
3. Open the disk image.
4. Drag **Sable** onto **Applications**.
5. Eject the disk image.

The app is not notarized, so macOS blocks the first launch.

1. In Applications, Control-click **Sable**.
2. Choose **Open**, then choose **Open** again.

Later launches are normal.

## What it does

| Screen | What you get |
| --- | --- |
| Overview | Health check of the Mac |
| Free Up Space | Caches and logs, with sizes |
| Apps | Installed apps and files they left behind |
| Tune-Up | Startup items, DNS flush, Spotlight |
| File Finder | Installers and archives. Does not change files |
| Threat Scan | Files compared to known malware hashes |
| Big Downloads | Large files in Downloads |
| Space Lens | Disk map. One scan, then folders open from that map |
| Cloud Storage | Local iCloud and other cloud folder sizes |
| System Monitor | Live CPU and memory |

Anything that removes a file asks first and uses the Trash. Scanning the whole Mac needs Full Disk Access. Desktop, Documents, and Downloads need Files and Folders access.

## Build from source

Rust 1.80 or newer. Run these from this folder.

```bash
make package
open dist/Sable-*.dmg
```

`make install` copies the app into `/Applications`. `make help` lists the other commands.

```text
crates/engine   scan, trash plan, disk map
crates/ui       Dioxus desktop window
```

## Publish a download

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions builds the disk image and zip and attaches them to that release.
