# REA recovery log — 2026-10-12

## Step 1 — System recovery

| Check | Result |
|-------|--------|
| Hopper running (`pgrep hopper`) | **No** |
| `/tmp/rea-*` before purge | **None** (glob empty) |
| `/tmp/rea-*` after `rm -rf /tmp/rea-*` | **None** |
| Stale dir from prior incident (`/tmp/rea-F1fjcg`) | Removed explicitly; still absent after purge |

### MCP lifecycle

- `close_binary`: **cleanup_incomplete** — Hopper provider still bound in MCP process; reported `hopper-document` + `/tmp/rea-F1fjcg` (filesystem already clean).
- `binary_session`: Session still references prior Hopper target until MCP reload.
- **Action taken:** Updated `~/.cursor/mcp.json` with Ghidra env (see below). **Cursor MCP reload required** for `open_binary` with `provider_id: ghidra`.

## Step 2 — Ghidra for REA

| Item | Value |
|------|--------|
| `GHIDRA_INSTALL_DIR` | `/opt/homebrew/opt/ghidra/libexec` (Homebrew ghidra 12.1.4) |
| `JAVA_HOME` | `/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home` (21.0.12.1) |
| Alt install | `/Users/hema/Library/ghidra/ghidra_12.1.4_PUBLIC` |
| CLI `rea doctor` (with env) | Ghidra checks **healthy** (headless, native decompiler, JDK 21) |
| CLI `rea analyze … --provider ghidra` | **Success** on `/tmp/CleanMyMac_5_MAS_arm64` |
| MCP `open_binary` + ghidra | **Blocked** — MCP process lacks `GHIDRA_INSTALL_DIR` (JDK 11 fallback) until reload |

## Step 3 — First feature bundle

- **Bundle:** `Features_CarouselFlow.bundle` (first line in `Features_bundles_paths.txt`)
- **Mach-O in bundle:** None — resource bundle only (`Info.plist`, `Assets.car`, localized `Localizable.strings`)
- **Analysis target:** Main exe arm64 slice; Swift `CarouselFlow` module linked in binary; bundle loaded via `CarouselFlowResources::bundle`

## Next for parent

1. Reload Cursor (or REA MCP) after `mcp.json` change.
2. Re-run `close_binary` then `open_binary` with `provider_id: ghidra` on `/tmp/CleanMyMac_5_MAS_arm64`.
3. Continue Track 2 decompile via MCP or CLI (`rea decompile` / `rea function` with `--provider ghidra`).

---

## Follow-up (post [REA recovery + Ghidra feature](ead219d0-34a7-4e8d-bc1e-8eba651ffdbb))

| Check | Result |
|-------|--------|
| `~/.cursor/mcp.json` | `GHIDRA_INSTALL_DIR` + `JAVA_HOME` present |
| MCP `open_binary` (`ghidra`, `/tmp/CleanMyMac_5_MAS_arm64`) | **Success** (arm64, sha256 `aae6adcc…`) |
| MCP `procedure_pseudo_code` (Carousel `bundle`) | **Timeout** — use CLI or snapshot; see `binary_logic/features_decompile/01_Features_CarouselFlow.md` |
| `/tmp/rea-*` | Still empty unless a new Ghidra session creates `/tmp/rea-*` (normal) |
| Track 2 artifact | `binary_logic/features_decompile/01_Features_CarouselFlow.md` |
