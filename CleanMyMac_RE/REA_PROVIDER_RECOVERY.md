# REA provider recovery (Hopper stuck)

## Ghidra (Homebrew — current)

```bash
export GHIDRA_INSTALL_DIR="/opt/homebrew/opt/ghidra/libexec"
export JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"
```

Set the same in `~/.cursor/mcp.json` under the `rea` server `env`, then **reload Cursor**. Verify: `rea doctor --format json` (ghidra checks green). Default provider for Track 2: **`ghidra`**, not Hopper.

## Symptom

- MCP `close_binary` → `cleanup_incomplete` (Hopper document + `/tmp/rea-F1fjcg`)
- MCP `open_binary` / `procedure_pseudo_code` / target-bound `inspect_plist` fail
- Hopper demo GUI may not support headless bridge (`Cannot launch Hopper! The handler some object is not defined.`)

## Safe cleanup (manual)

1. Quit **Hopper Disassembler** if running.
2. Remove stale REA runtime dirs (only when Hopper is quit):
   ```bash
   rm -rf /tmp/rea-F1fjcg /tmp/rea-hopper-501 /tmp/rea-*
   ```
3. **Restart Cursor** (or reload REA MCP server) so session state resets.
4. Prefer **Ghidra** for deep analysis instead of Hopper:
   - Install Ghidra 12.1.x, set `GHIDRA_INSTALL_DIR`
   - Use **JDK 21+** (`export JAVA_HOME=$(/usr/bin/usr/libexec/java_home -v 21)`)
   - Analyze **thinned arm64** slice: `/tmp/CleanMyMac_5_MAS_arm64` (`lipo -thin arm64` from main exe)

## CLI that still works without Hopper

```bash
rea inspect-macho /tmp/CleanMyMac_5_MAS_arm64
rea inspect-plist /Applications/CleanMyMac_5_MAS.app/Contents/Info.plist --path /Applications/CleanMyMac_5_MAS.app/Contents/Info.plist
rea inspect-asset-catalog <path-to.car>
rea decode-interface-builder <path-to.nib>
```

## After recovery

Re-run Track 2: `open_binary` → `search_procedures` / `analyze_function` on `JunkCleanupModule`, `PantherEngineService`, `ProtectionModule` framework slices.
