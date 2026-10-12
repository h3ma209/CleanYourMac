# Real-Time Protection — binary logic (Track 2)

**Target bundle:** `Features_RealTimeProtection.bundle` (resources only).  
**Executable logic:** main app `CleanMyMac_5_MAS` (modal orchestration) + `ProtectionModule.framework` (Smart Care tiles, background scan glue) + `PantherEngineService.framework` (monitor delegate) + lower stack in `02_RealTimeProtection_ARCHITECTURE.md` (`MalwareScanningRoutines` walk/check).

Ghidra 12.1.4 via REA CLI (`--snapshot` under `binary_logic/snapshots/`).

## Bundle selection (Step 0)

From `Features_bundles_paths.txt`, excluded onboarding/paywall bundles. **First core utility:** `Features_RealTimeProtection.bundle`.

## 1. Modal orchestration (main app)

| Address | Procedure | Notes |
| --- | --- | --- |
| `0x10028cd94` | `_TtC18RealTimeProtection25MoonlockRTPStateModalView::initWithFrame:` | AppKit view construction |
| `0x10028cbe8` | `MoonlockRTPStateModalView::updateConstraints` | Layout |
| `0x10002f598` | `DebugMenuHandler::showMoonlockRTPModal` | Debug entry |
| `0x1003c3c20` | `objc_stub::showMoonlockRTPModal` | Routed show hook |

Presenter `MoonlockRTPStateModalPresenter` gates display (intro incomplete, first launch, module unavailable, duplicate enqueue, unchanged `RealTimeProtectionState`). App shell: `moonlockStateService`.

Moonlock bundle IDs: `S8EX82NJP6.com.macpaw.moonlock-mas`, `S8EX82NJP6.com.macpaw.moonlock.interoperability`.

Excerpt: `excerpts/MoonlockRTPStateModalView_initWithFrame_0x10028cd94.md`

## 2. Protection module — preferences and timing

| Address | Procedure | Notes |
| --- | --- | --- |
| `0x2971c` | `MalwareBackgroundScanningPreferences::get_isMonitoringEnabled` | Reads global flag at `PTR_000bcf10` |
| `0x8fbd0` | `ProtectionWatcherTiming::$get_timeoutBeforeScanningInBackground` | Background pass delay |
| `0x81c44` | `PantherEnginePrivilegedAgentDependency::get_isFulfilled` | Privileged helper gate |

Symbols: `ProtectionBackgroundMalwareScanHandler`, `backgroundMalwareScanCompleted`, `backgroundMalwareCancellable` — from `ProtectionModuleBuilder` with Moonlock RTP flags.

Binary: `/tmp/ProtectionModule_arm64` — snapshot `snapshots/ProtectionModule`.

## 3. PantherEngine — monitor callbacks

| Address | Method | Role |
| --- | --- | --- |
| `0x2434` | `MalwareBackgroundScanner.init(...)` | Queue, model factory, preferences, analytics |
| `0x3690` | `malwareMonitorWillStartFastSystemScanning` | Module tracker flag |
| `0x3748` | `malwareMonitorDidScanPath:` | Path forward |
| `0x38ac` | `malwareMonitorReportNewMalware:detectionSource:` | `PANMalware` → Swift handler `FUN_00003808` |
| `0x3924` | `malwareMonitorReportAllDetectedMalware:` | Batch report |

Binary: `/tmp/PantherEngineService_arm64` — snapshot `snapshots/PantherEngineService`.

## 4. Scan engine (prior pass)

`MalwareScanningRoutines`: `PANBaseFilesEnumerator perform` @ `0x4d9c`, `iterateDirectoryWithURL` @ `0x4e84`, `PANBaseFilesChecker perform` @ `0x1de00`. See `excerpts/*.c`.

## 5. End-to-end loop

1. Moonlock vs CMM owner → RTP modal copy + monitor enable.
2. `EngineFacade.startMonitoring` → monitors → `MalwareBackgroundScanner` delegates.
3. Detections → Protection tiles / removal tasks (separate from walk).

## Snapshots (this session)

- `snapshots/CleanMyMac_main` — `/tmp/CleanMyMac_5_MAS_arm64`
- `snapshots/ProtectionModule` — `/tmp/ProtectionModule_arm64`
- `snapshots/PantherEngineService` — `/tmp/PantherEngineService_arm64`

## Limits

- `ProtectionBackgroundMalwareScanHandler` — strings/builder only in Ghidra.
- `checkURL` detection rules closed-box.
- Analytics excluded per user filter.
