# Space Lens & Smart Care

## Space Lens

**Frameworks:** `SpaceLensModule`, `SpaceLensUI`, `SpaceLensScanning`  
**Assets:** Module `.car` 21 · UI `.car` 36

**Observed types:**

- `SpaceLensModuleCoordinatorType` / coordinator pattern (same as other modules)
- `SpaceLensScanResultsScreenCoordinator`, `SpaceLensEmptyScanResultsScreenCoordinator`
- `SpaceLensModuleProgressTransformer` — progress UI binding
- Main binary string: `SpaceLensCloudScannerFeature` — optional cloud-backed scan facet

**Hypothesis:** Builds a hierarchical disk map from `SpaceLensScanning`; renders treemap/list in `SpaceLensUI`; module hosts navigation inside main window canvas.

```swift
// inferred
SpaceLensModuleCoordinator.startScan() 
  → SpaceLensScanning.engine.scan(volume)
  → SpaceLensScanResultsScreenCoordinator.present(results)
```

## Smart Care

**Frameworks:** `SmartCareModule`, links to organize/protection/junk sub-scans  
**Assets:** `.car` 26

**Observed types:**

- `SmartCareModuleController` / `SmartCareModuleControllerType`
- `ModulesInfoController` — aggregates submodule metadata
- Sub-module controller items: `OrganizeSubModuleControllerItem`, `LAOFSubModuleControllerItem`, `SmartCareProtectionSubModuleControllerItem`
- Main binary: `SmartCareCoordinatorBuilder`, `SmartCareReminderController`, `SmartCarePerformanceModuleScanningConfig`
- Notifications: `Reminder_RegularSmartCare_*` localized strings

**Hypothesis:** Smart Care is a **composite scan** that sequences submodule scanners (junk, performance, protection, organize), then presents unified results. Reminder notifications re-engage users on interval (`AnalyticsSmartCareReminderInterval` string).

```swift
// inferred orchestration
SmartCareModuleController.run() {
    for sub in [junk, laof, organize, protection, performance] {
        sub.scanIfEnabled()
    }
    presentUnifiedResults()
}
SmartCareReminderController.scheduleNext(interval: remotePrefs)
```

## Unknowns

- Exact submodule ordering and parallelism (static analysis of `SmartCareModule` arm64 needed).
- Space Lens cloud scanner data paths and privacy scope.
