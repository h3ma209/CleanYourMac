# Domain 1 — Smart Care orchestrator

Static evidence only. Ghidra via `rea decompile --provider ghidra`. No runtime observation. Malware detection, browser vaults, and Domains 3–6 are out of scope.

## Targets

| Binary | SHA-256 | Role |
|--------|---------|------|
| `/tmp/cmm_slices/SmartCareModule_arm64` (thinned from `SmartCareModule.framework/Versions/A/SmartCareModule`) | `a9822041af19467384773b3d72835166d88abfd4cdb74b40655e78ba576c4a90` | Orchestrator |
| `/tmp/CleanMyMac_5_MAS_arm64` | not re-hashed this pass | Host strings only |

Snapshot: `/tmp/cmm_smartcare.rea-snapshot` (created this pass). Snapshots under `binary_logic/snapshots/` are other binaries (main, Protection, Panther) and were not loaded for this image.

Decompiles (4):

| Function | Address | Evidence |
|----------|---------|----------|
| `SmartCareModule.SmartCareModuleController.startScan()` | `0xca18` | `ev_cf36e112bd3416ebb1017c25646078d1cf5c2caaba9f94287a6d12cc0d2f1e6d` |
| `SmartCareModule.SmartCareModuleController.pauseScan()` | `0xd5dc` | same snapshot, Ghidra pseudocode |
| `SmartCareModule.SmartCareModuleController.startClean()` | `0xd894` | same snapshot |
| `SmartCareModule.SmartCareModuleController.idle()` | `0xc974` | same snapshot |

## Orchestrator vs navigation shell

The type that **starts, pauses, and aggregates** Cleanup, Protection, and Performance is `SmartCareModule.SmartCareModuleController` (`SmartCareModuleControllerType`).

`SmartCareModule.SmartCareModuleCoordinator` is the module-navigation shell (`moduleCanStart(flow:)`, `failedRun(flow:error:)`, `scanDependencies`, intro/main-flow factories). It does not contain the fan-out loop. The main executable names a builder, not the loop: cstring `SmartCareCoordinatorBuilder` and class `_TtC10CleanMyMac27SmartCareCoordinatorBuilder` in `/tmp/CleanMyMac_5_MAS_arm64`, plus `SmartCareReminderController` (reminder scheduling only; interval options are settings strings, not a scan state machine).

## Frameworks actually linked from SmartCareModule

`otool -L` on the arm64 slice:

- `ApplicationsModule`, `ApplicationsModuleDomain`
- `JunkCleanupModule`
- `OrganizeModule`
- `PerformanceModule`, `PerformanceScanning`
- `ProtectionModule`
- `CleaningCore`, `ScanningCore`, `ModuleCore`, `ModuleUI`
- `Combine`, `CombineSchedulers`, `CombineUtils`, `CombineExt`
- `libswiftDispatch.dylib`

`SmartCareModules.init` takes six optional `ModuleCore.Module<…Config>` values: organizer, junk cleanup, performance, applications, protection, laof (second Organize config). Stored property names in Swift reflection (`__swift5_reflstr`): `organizerScanningModule`, `junkCleanupScanningModule`, `performanceScanningModule`, `applicationsScanningModule`, `protectionScanningModule`, `laofScanningModule`.

Concrete item classes (ObjC class symbols):

- `SmartCareSubModuleControllerItem` (generic card; ivars include `_state`, `moduleController`, `isScanCancelled`, `isCleanCancelled`)
- `SmartCareProtectionSubModuleControllerItem`
- `OrganizeSubModuleControllerItem`
- `LAOFSubModuleControllerItem`

A second reflection cluster names controllers `junkController`, `protectionController`, `performanceController`, `applicationsController`, `organizeController`, `laofController`.

## Proven states

Swift field metadata (`__swift5_fieldmd`), kind = enum.

**`SmartCareViewModel` UI state** (5-case enum at file `0x6e10c`, no associated values). Sits immediately before the class field list that stores `_state`, `_scanProgress`, `_cleanProgress`. Nominal type `SmartCareViewModelState` exists. The field-type pointer itself did not demangle; the binding is layout adjacency plus that nominal.

| Case |
|------|
| `idle` |
| `scanning` |
| `scanResults` |
| `cleaning` |
| `cleanResults` |

`stopping` is a **stored property** on `SmartCareViewModel` (class field list at `0x6e158`), not an enum case.

**Submodule item state** (6-case enum at `0x6cc30`). Nominal `SmartCareSubModuleControllerItemState`. `scanning` and `cleaning` carry associated value `isPaused: Bool` (mangled `Sb8isPaused_t`). `idle` and `cleanIdle` have no payload. `scanFinished` and `cleanFinished` have indirect payloads; the payload types were **not recovered**.

| Case | Payload |
|------|---------|
| `scanning` | `isPaused: Bool` |
| `scanFinished` | present, type unknown |
| `cleaning` | `isPaused: Bool` |
| `cleanFinished` | present, type unknown |
| `idle` | none |
| `cleanIdle` | none |

Not observed as state names: `Analyzing`, `Resolving`, `Completed`.

## Control flow

Ivar offsets on `SmartCareModuleController` (values stored at `_OBJC_IVAR_$_…`): `submodules` = `0x10`, `delegate` = `0x18`, `analyticsTracker` = `0x28`, `nfrTracker` = `0x50`, `successfulCleanupCounter` = `0x78`, `scanProgressObserver` = `0xa8`, `cleanProgressObserver` = `0xb0`.

`startScan`, `pauseScan`, `startClean`, and `idle` all load `self.submodules` (`this+0x10`, then the Swift array at `+0x70` of that buffer) and walk it. Each element is an existential. The witness called depends on the entry point (`+0x68` start scan, `+0x78` pause scan, `+0x90` start clean, `+0xb8` idle). Those slots are protocol methods on `SmartCareSubModuleControllerItemType`. The requirement names at those slots were not recovered; the controller method names are.

```
startScan() -> AnyPublisher<SmartCareModuleScanResult, ModuleScanError>:
    for (index, item) in submodules:
        publisher = item.<witness +0x68>()          // per-module scan publisher
        wrap with Combine closures that capture (self, index, item)
        append to array
    merge the array into one publisher
    map output to SmartCareModuleScanResult
    return publisher

pauseScan():
    for item in submodules:
        item.<witness +0x78>()

startClean() -> AnyPublisher<SmartCareModuleCleanResult, ModuleCleanError>:
    same fan-out as startScan, witness +0x90
    result type SmartCareModuleCleanResult

idle():
    for item in submodules:
        item.<witness +0xb8>()
```

Also exported, not decompiled: `stopScan`, `resumeScan`, `setScanCancelledState`, `stopClean`, `pauseClean`, `resumeClean`, `setCleanCancelledState`. Progress publishers: `progress -> AnyPublisher<ScanningCore.ScanProgress, Never>`, `cleanProgress -> AnyPublisher<ModuleCore.CleanProgress, Never>`.

Aggregation is Combine, not a hand-rolled result struct built inside the loop. The loop allocates one publisher per submodule and then combines the array (`__swiftEmptyArrayStorage` grown each iteration, then a combine helper over that array, then a map whose type metadata is `SmartCareModuleScanResult` / the clean equivalent).

## Queues

Inside these four functions: **no** `DispatchQueue`, `OS_dispatch_queue.async`, or `NSOperationQueue` call.

The framework does link `libswiftDispatch` and `CombineSchedulers`. Whether a subscriber hops off the main queue is **not observed** here.

Off-main work shows up one layer down, in symbols only (not decompiled this pass): `ScanningCore.ProcessingQueue` stores an `NSOperationQueue`, `maxConcurrentOperationCount`, and `NSRecursiveLock`, and exposes `run` / `runSync`. `ScanningCore.GenericScanner.scan()` returns that queue’s `QueueResult`. `ScanningCore.ControlObject.pause()` and `onceUnpaused` exist. Smart Care’s own pause is the submodule witness loop above.

## UI status names

Observed publishers and delegates (symbols and reflection), not guessed:

- `SmartCareViewModel._state` → `SmartCareViewModelState`
- `SmartCareViewModel._scanProgress`, `_cleanProgress`
- `SmartCareViewModel._onReview`, `_onUnlock`, `_onShowSettings`, `_onShowLog`
- `CardViewModel.state` → `AnyPublisher<ComposedCardState, Never>`
- `SmartCareModuleControllerDelegate`
- `SmartCareExternalDelegate`
- `SmartCareScreenCoordinatorDelegate` (`performMainAction`, `performBackAction`, `mainComponentsUIStateStartingFrom`)
- `SmartCareNotificationSender` / `SmartCareNotificationSenderType`
- `ScanProgressObserver` (ivar `scanProgressObserver`)
- Localization keys in the same binary: `local_notification_scan_complete_title`, `ubiquitous_button_scan`, `ubiquitous_button_scan_stop`, `scan_results_title`, `junkCleanup_runStoppedSubtitle`, `alert_stop_scan_message_text`, `alert_start_scan_message_text`
- Analytics cstrings: `Analytics: trackScanStart`, `Analytics: trackScanComplete` (`SmartCareAnalyticsTracker.swift` path string)

`SmartCareViewModelEvent` is a nominal type. Its cases were **not** recovered.

## Clean-room Rust sketch

Not a copy of MacPaw source. Thread model only.

```rust
// UI thread owns ViewModelState { Idle, Scanning, ScanResults, Cleaning, CleanResults }.
// One tokio task per enabled module (junk, protection, performance, …).
// Orchestrator merges JoinSet / FuturesUnordered results into one ScanReport.
// Pause = AtomicBool checkpoint inside each task, matching isPaused on the
// scanning/cleaning item states. No privileged helper in this domain.
```

Crates: `tokio` (module tasks), `futures` (merge), a UI channel (`tokio::sync::watch` for state). Scan work itself belongs in Domain 2.
