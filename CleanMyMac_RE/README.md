# CleanMyMac 5 MAS — Reverse Engineering Workspace

**Target:** `/Applications/CleanMyMac_5_MAS.app`  
**Main binary:** `Contents/MacOS/CleanMyMac_5_MAS` (universal fat; arm64 analysis slice: `/tmp/CleanMyMac_5_MAS_arm64`)  
**Version:** 5.7.1 (build 50701.0.2609301754) · `com.macpaw.CleanMyMac-mas`
## Index

| Track | Document | Purpose |
|-------|----------|---------|
| Overview | [00_initial_findings.md](./00_initial_findings.md) | Executive structural summary |
| Track 1 — UI & assets | [workspace_assets/UI_SCREEN_MAP.md](./workspace_assets/UI_SCREEN_MAP.md) | Screens/modules from bundles, menus, catalogs |
| Track 1 — UI REA | [workspace_assets/UI_REA.md](./workspace_assets/UI_REA.md) | Nib decode, window shell, module view controllers |
| Track 1 — manifest | [workspace_assets/manifest.json](./workspace_assets/manifest.json) | Resource counts + Features_* bundles |
| Track 1 — hashes | [workspace_assets/manifest_hashes.txt](./workspace_assets/manifest_hashes.txt) | SHA-256 for `.car` / `.nib` paths |
| Track 1 — catalogs | [workspace_assets/main_asset_catalog_p0.json](./workspace_assets/main_asset_catalog_p0.json) | REA `inspect-asset-catalog` (catalog list) |
| Track 1 — plist | [workspace_assets/Info_parsed.json](./workspace_assets/Info_parsed.json) | REA `inspect-plist` evidence |
| Track 2 — entry | [binary_logic/00_entry_and_composition.md](./binary_logic/00_entry_and_composition.md) | LC_MAIN, ComposedApplication, bootstrap |
| Track 2 — junk | [binary_logic/01_junk_cleanup.md](./binary_logic/01_junk_cleanup.md) | System junk / mail / AI clutter flows |
| Track 2 — protection | [binary_logic/02_protection_malware.md](./binary_logic/02_protection_malware.md) | Malware scan, Moonlock, Panther engine |
| Track 2 — space & care | [binary_logic/03_space_lens_smart_care.md](./binary_logic/03_space_lens_smart_care.md) | Space Lens, Smart Care orchestration |
| Track 2 — perf & assistant | [binary_logic/04_performance_assistant.md](./binary_logic/04_performance_assistant.md) | Performance module, Assistant recommendations |
| Track 2 — activation | [binary_logic/05_activation_routing_health.md](./binary_logic/05_activation_routing_health.md) | MAS activation, routing, login items |
| Track 1 — login items | [workspace_assets/06_login_items_satellites.md](./workspace_assets/06_login_items_satellites.md) | HealthMonitor + Menu agents (follow-up) |
| Track 1 — Features bundles | [workspace_assets/Features_bundles_paths.txt](./workspace_assets/Features_bundles_paths.txt) | Onboarding / purchase / FDA bundles |
| Ops — REA unblock | [REA_PROVIDER_RECOVERY.md](./REA_PROVIDER_RECOVERY.md) | Clear Hopper stuck state; Ghidra path |
| Ops — Ghidra install | [GHIDRA_INSTALL.md](./GHIDRA_INSTALL.md) | Homebrew paths + MCP env |
| Ops — recovery log | [RECOVERY_LOG.md](./RECOVERY_LOG.md) | `/tmp/rea-*` purge + provider status |
| Track 2 — Carousel (Ghidra) | [binary_logic/features_decompile/01_Features_CarouselFlow.md](./binary_logic/features_decompile/01_Features_CarouselFlow.md) | First `Features_*.bundle` + decompile |
| Track 1 — RTP UI | [workspace_assets/RTP_ui_map.md](./workspace_assets/RTP_ui_map.md) | Moonlock vs CMM status modal |
| Track 3 — RTP blueprint | [binary_logic/features_decompile/02_RealTimeProtection_ARCHITECTURE.md](./binary_logic/features_decompile/02_RealTimeProtection_ARCHITECTURE.md) | Scan walk in MalwareScanningRoutines |
| Track 3 — RTP replica | [binary_logic/features_decompile/BLUEPRINT_RealTimeProtection.md](./binary_logic/features_decompile/BLUEPRINT_RealTimeProtection.md) | Standalone replica guide |
| Track 2 — RTP logic | [binary_logic/features_decompile/02_RealTimeProtection_logic.md](./binary_logic/features_decompile/02_RealTimeProtection_logic.md) | Monitor glue + modal init |
| Track 2 — scan pipeline | [binary_logic/features_decompile/03_SCAN_PIPELINE_CLOSED.md](./binary_logic/features_decompile/03_SCAN_PIPELINE_CLOSED.md) | startScanning through checkURL call |
| Domain 1 — Smart Care | [binary_logic/features_decompile/04_DOMAIN1_SMART_CARE.md](./binary_logic/features_decompile/04_DOMAIN1_SMART_CARE.md) | Orchestrator start/pause/aggregate, proven states |
| Domain 2 — cleanup engine | [binary_logic/features_decompile/05_DOMAIN2_CLEANUP_ENGINE.md](./binary_logic/features_decompile/05_DOMAIN2_CLEANUP_ENGINE.md) | Junk paths, enumerator, trash vs lock-aware remove |
| Domain 3 — protection | [binary_logic/features_decompile/06_DOMAIN3_PROTECTION.md](./binary_logic/features_decompile/06_DOMAIN3_PROTECTION.md) | XPC health-monitor client and permission tags |
| Domain 4 — performance | [binary_logic/features_decompile/07_DOMAIN4_PERFORMANCE.md](./binary_logic/features_decompile/07_DOMAIN4_PERFORMANCE.md) | Login items, DNS flush, Spotlight reindex |
| Domain 5 — applications | [binary_logic/features_decompile/08_DOMAIN5_APPLICATIONS.md](./binary_logic/features_decompile/08_DOMAIN5_APPLICATIONS.md) | Bundle id, leftover roots, reset vs uninstall |
| Domain 6 — Space Lens | [binary_logic/features_decompile/09_DOMAIN6_SPACE_LENS.md](./binary_logic/features_decompile/09_DOMAIN6_SPACE_LENS.md) | Size tree node and scan task factory |

## Prior session

Extended REA report: `../CleanMyMac_5_MAS_REA_Structural_Report.md`

## Tooling notes

- **Ghidra:** Homebrew `ghidra` 12.1.4 — `GHIDRA_INSTALL_DIR=/opt/homebrew/opt/ghidra/libexec`, JDK 21 via `openjdk@21`. Env set in `~/.cursor/mcp.json` for REA MCP.
- **REA MCP:** `open_binary` + `provider_id: ghidra` on `/tmp/CleanMyMac_5_MAS_arm64` **works** after MCP reload. Prefer Ghidra over Hopper (demo bridge unreliable). Long `procedure_pseudo_code` calls may timeout — use `rea decompile` CLI or `--snapshot` for reuse.
- **REA CLI:** `rea analyze`, `rea search`, `rea decompile`, `rea xrefs` with same env exports.
