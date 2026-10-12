# Login items & satellite executables

Follow-up to Track 1/3 after [CMM RE Track 1+2](586a3607-79cb-426d-a729-48a543766aee).

## Bundles

| App | Bundle ID | Executable | Role (inferred) |
|-----|-----------|------------|-----------------|
| `CleanMyMac_5_MAS_HealthMonitor.app` | `S8EX82NJP6.com.macpaw.CleanMyMac-mas.HealthMonitor` | `CleanMyMac_5_MAS_HealthMonitor` | Background health/malware monitoring, reports, IPC to main app |
| `CleanMyMac_5_MAS_Menu.app` | `com.macpaw.CleanMyMac-mas.Menu` | `CleanMyMac_5_MAS_Menu` | Menu-bar agent: assistant UI, health issues, quick actions |

Plists (exported): `HealthMonitor_Info.json`, `Menu_Info.json`.

## Shared mechanism (dylib overlap)

Both satellites link the same **platform spine**:

- `HealthMonitoringAPI`, `HealthMonitoringCore`, `HealthMonitoringReports`, `HealthIssuesController`
- `PantherEngineService`, `PANEngine` (malware engine in background)
- `RoutingService`, `RoutingServiceAPI` (deep links / open-main-app routes)
- `IPCProtocol` (HealthMonitor only — explicit IPC channel)
- `CleanMyMacLoginItems`, `PreferencesStorage`, `ActivationService` (Menu)

**Interpretation:** Login items are not thin launchers; they embed **scan/monitor + routing** so menu/health UX works when the main window is closed.

## Menu-only extras

- `AssistantDomain`, `AssistantUI`, `AssistantCore` — recommendations in menu bar
- `CleanMyDriveScanner` — external volume / drive cleanup entry point from menu

## Main app `Features_*.bundle` (onboarding / gated UI)

Paths listed in `Features_bundles_paths.txt` (8 bundles under `Contents/Resources/`): purchase flow, settings, FDA prompts, RTP, carousel, email subscription, keep-menu, intro.

## XPC / services

No `.xpc` bundles under either login item in this MAS build; background work uses **embedded frameworks + IPCProtocol** rather than separate XPC targets in the login-item bundles themselves.

## Next probes

- `strings` / REA on `IPCProtocol` message names (requires Ghidra or working REA provider)
- `capture_native_ui_scenario` on Menu.app while main app quit vs running
- Compare HealthMonitor vs main binary `HealthMonitorClient.start()` call sites after decompilation
