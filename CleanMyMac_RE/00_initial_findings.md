# CleanMyMac 5 MAS — Initial Structural Findings

**Date:** 2026-10-12  
**Methods:** REA CLI (`inspect-macho`, `inspect-plist`, `inspect-asset-catalog`, `decode-interface-builder`), `otool`, `swift demangle`, `strings`, filesystem inventory.

---

## 1. Resource inventory (under `Contents/`)

| Type | Count | Notes |
|------|------:|-------|
| `.car` (compiled asset catalogs) | **44** | Main app + per-module frameworks + `Features_*` bundles + login items |
| `.nib` | **13** | Almost all **Sparkle** updater + **CMAnalyticsWindowController** (MacPaw analytics window); **no feature module storyboards/nibs** |
| `.storyboardc` | **0** | UI is programmatic Swift/AppKit (+ SwiftUI in places) |
| `.xib` | **0** | — |
| `.strings` | **735** | Localized copy across modules and bundles |
| `.plist` | **311** | Info, feature bundles, Sparkle, login items, embedded deps |
| `.stringsdict` | **168** | Pluralized strings |
| `.xcassets` (source) | **0** | Shipped as compiled `.car` only |

**Frameworks:** 133 dylibs under `Contents/Frameworks/`.  
**Feature resource bundles:** 8 × `Features_*.bundle` (intro, purchase, settings, FDA resolution, real-time protection, carousel, email subscription, keep menu open).

Extracted artifacts live in `CleanMyMac_RE/workspace_assets/` (manifest, hashes, plist JSON, asset catalog metadata, `main_Assets_car_info.txt`).

---

## 2. Top UI modules / screens (inferred)

Primary shell (**code-driven**, not IB storyboards):

| Layer | Framework / bundle | Role |
|-------|---------------------|------|
| Window chrome | `MainAppUI` | `MainWindowContainerViewController`: navigation bar, sidebar, module canvas, live background |
| Navigation | `NavigationUI` | Sidebar / module routing (linked from main exe) |
| Shared chrome | `SharedUI`, `DesignSystem`, `ModuleUI`, `ModuleCore` | Reusable module scaffolding |
| Settings | `SettingsUI` + `Features_Settings.bundle` | Settings flows + assets |
| Onboarding / monetization | `Features_IntroFlow`, `Features_PurchaseFlow`, `Features_CarouselFlow`, `RateFlow` | First-run, paywall, carousel |
| Permissions | `Features_FDAResolutionUIKit.bundle` | Full Disk Access / TCC resolution UI |
| Protection UX | `Features_RealTimeProtection.bundle`, `ProtectionModule` (116 catalog records) | RTP + malware UI |
| Menu bar | `CleanMyMac_5_MAS_Menu.app` (login item) | Large `Assets.car` (293 records); Assistant/SharedUI assets |
| Health | `CleanMyMac_5_MAS_HealthMonitor.app` (login item) | Background health / notifications |

**Menu-visible modules** (from `en.lproj/Localizable.strings`): Cleanup (junk), Protection, Performance, Smart Care, Space Lens, Cloud Cleanup, Assistant.

**Decoded nibs (REA):** Only `CMAnalyticsWindowController` + Sparkle update UI — confirms MacPaw feature UI is not Interface Builder–storyboard based.

See [workspace_assets/UI_SCREEN_MAP.md](./workspace_assets/UI_SCREEN_MAP.md).

---

## 3. Binary entry & composition pattern

| Property | Value |
|----------|--------|
| Format | Mach-O 64-bit PIE executable, **universal** (x86_64 + arm64) |
| arm64 UUID | `9E01313F-D270-368D-87FD-E30D0B9D0932` |
| **LC_MAIN** | `entryoff` **18040** → **`__text` @ `0x100004678`** |
| Source unit | `CleanMyMac/main.swift` (literal in `__cstring`) |
| Linked libraries | **147** (`rea inspect-macho`, exhaustive dependency list) |

**Pattern:** Thin **composition root** in the main binary:

1. `SharedUtils.ComposedApplication` on stack/heap.
2. Branch on `NSProcessInfo` extensions: `isCommandLineRun`, `isRunningTests`, `isRunningSwiftUIPreviews`.
3. GUI path: `CMIssuesReportingApplication` + `NSApplication.setup(with: ComposedApplication)`.
4. `NSApplicationMain` → AppKit run loop → `CleanMyMac.AppDelegate` + module graph wired by **ApplicationConfigurator** types (symbols/strings in main binary, not a separate framework).

Pseudocode: [binary_logic/00_entry_and_composition.md](./binary_logic/00_entry_and_composition.md) (disassembly-derived; **not** Ghidra/Hopper decompiler output).

---

## 4. Feature / module map (mechanism hypotheses)

| Feature | Primary frameworks | Mechanism hypothesis |
|---------|-------------------|----------------------|
| **System junk cleanup** | `JunkCleanupModule`, `CleaningCore`, `SystemJunkScanning`, `*Scanning` | Coordinator-per-flow (`SystemJunkFlowCoordinator`, `MailAttachmentsFlowCoordinator`, `AIClutterFlowCoordinator`); privileged deletes via `PrivilegedOperationsPerformerService` / `SystemJunkPrivilegedOperationsPerformerAdapter` |
| **Malware / protection** | `ProtectionModule`, `PantherEngineService`, `PANEngine`, `MalwareScanning*`, `MalwareMonitoringRoutines` | Background `_MalwareBackgroundScanner` with toggles (DMG/PKG/archive/PUA/external drives/Chrome extensions); UI flows `MalwareFinderFlowCoordinator`, `ReviewMalwareFlowCoordinator`; Moonlock cross-promo views |
| **Space Lens** | `SpaceLensModule`, `SpaceLensUI`, `SpaceLensScanning` | `SpaceLensModuleCoordinator`; scan results coordinators; optional cloud scanner feature string in main binary |
| **Smart Care** | `SmartCareModule`, `SmartCareModule` + scanning configs | `SmartCareModuleController` aggregates sub-modules (Organize, LAOF, Protection tiles); reminders via `SmartCareReminderController` |
| **Performance** | `PerformanceModule`, `PerformanceScanning`, `PerformanceService` | Flow coordinators for login/background items scanners |
| **Assistant** | `AssistantModule`, `AssistantUI`, `AssistantDomain` | Recommendation VMs per module (junk/protection/smart care); menu/login item surfaces |
| **Cloud / email** | `CloudStorageModule`, `EmailCleanupModule`, Gmail/Dropbox/Google APIs | OAuth URL scheme `db-hfn3qmlc7uq9kxd`; module-specific `.car` catalogs |
| **Activation (MAS)** | `ActivationService`, main-binary `ActivationManagerMAS*` | StoreKit + MacPaw account sign-in strings; `Features_PurchaseFlow.bundle` |
| **Routing / deep links** | `RoutingService`, `MainAppRoutingFactory`, `CommonRoutingFactory` | URL schemes `cleanmymac5`, `cleanmymac5-mas` |
| **Health monitor** | `HealthMonitoringAPI`, login item HealthMonitor app | Separate process; `HealthMonitorAppLifecycleHandler` in main app strings |
| **Updates** | `MASUpdaterServiceAPI`, embedded Sparkle | MAS updater API + Sparkle nibs in bundle |

Per-feature write-ups: `binary_logic/01_*.md` … `05_*.md`.

---

## 5. Extracted to disk

Base:  
`/Users/hema/Library/Application Support/Cursor/AgentStores/cursor_agent_stores/47ae0ec1-7dd0-49e7-8850-38658ea2ae6b/files/CleanMyMac_RE/`

| Path | Contents |
|------|----------|
| `workspace_assets/manifest.json` | Counts + Features_* list |
| `workspace_assets/manifest_hashes.txt` | 57 `.car`/`.nib` SHA-256 lines |
| `workspace_assets/Info_parsed.json` | REA plist evidence |
| `workspace_assets/Info.plist` | Raw copy |
| `workspace_assets/main_asset_catalog_p0.json` | All `.car` paths + record counts + digests |
| `workspace_assets/main_Assets_car_info.txt` | `assetutil --info` head |
| `binary_logic/main_inspect_macho.json` | Partial/truncated full macho JSON |
| `binary_logic/*.md` | Entry + feature logic notes |

REA does **not** extract `.car` rendition bytes via `inspect-asset-catalog` (metadata only).

---

## 6. Blockers & next steps

| Blocker | Impact | Next step |
|---------|--------|-----------|
| **Hopper REA bridge broken** | MCP `open_binary`, `procedure_pseudo_code`, `search_strings`, `get_objc_classes` unavailable; session `cleanup_incomplete` | Quit Hopper / clear `/tmp/rea-*`; reconfigure REA to skip Hopper or use **Ghidra** provider |
| **Ghidra not installed** | No automated pseudocode | User-approved Ghidra 12.x + `GHIDRA_INSTALL_DIR`; analyze `/tmp/CleanMyMac_5_MAS_arm64` and key frameworks |
| **UI mostly non-IB** | Limited `decode-interface-builder` value | `capture_native_ui_scenario` for module navigation; Swift type dump via `swift-metadata` / runtime |
| **Framework depth** | This pass: strings + symbols on selected arm64 slices | `rea inspect-macho` per framework; trace `ApplicationConfigurator` registrars in main binary with Ghidra |

---

## Evidence anchors

- Executable SHA-256: `e7861e5dcdbf0c03ae367cf91e4176f26ba187fdb16bd0035fe5e4ee6b301f01`
- REA plist evidence: `ev_f78b1b32da10fd3351d1d341f849fa045e1a962f56d593a27ec0c76c6db1994f`
- REA IB decode evidence: `ev_96fff598fad8100d627b9e7f1846cc08ddf108321b3cba92e69a207aa3505a23`
