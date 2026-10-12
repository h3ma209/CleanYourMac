# Ghidra install for REA (required for Track 2 decompile)

**Status (2026-10-12):** Exhaustive `ghidraRun` search found **no Ghidra installation** on this Mac (`mdfind`, `/Applications`, `~/Applications`, `GHIDRA_INSTALL_DIR` unset).

## Install (manual — approve download)

1. Download **Ghidra 12.1.x PUBLIC** from [NSA Ghidra releases](https://github.com/NationalSecurityAgency/ghidra/releases) (zip, not brew cask on this host).

2. Extract, e.g.:
   ```bash
   unzip ghidra_*_PUBLIC_*.zip -d ~/Applications
   export GHIDRA_INSTALL_DIR=~/Applications/ghidra_12.1_PUBLIC
   ```

3. JDK **21+** (Homebrew):
   ```bash
   brew install openjdk@21
   export JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"
   export PATH="$JAVA_HOME/bin:$PATH"
   ```

4. Persist in shell profile:
   ```bash
   export GHIDRA_INSTALL_DIR="$HOME/Applications/ghidra_12.1_PUBLIC"
   export JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"
   ```

5. Verify:
   ```bash
   rea doctor | rg -i ghidra
   rea decompile /tmp/CleanMyMac_5_MAS_arm64 0x100004678 --provider ghidra --format md | head
   ```

6. **Restart Cursor** so REA MCP inherits `GHIDRA_INSTALL_DIR` and `JAVA_HOME`.

## First feature target (queued)

First bundle in `workspace_assets/Features_bundles_paths.txt`:

`Features_CarouselFlow.bundle` — likely **assets + plist**; init hooks may live in main binary / `PurchaseFlow` / UI frameworks. Decompile those after Ghidra opens arm64 slices.
