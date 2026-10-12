# Junk cleanup module

**Frameworks:** `JunkCleanupModule`, `CleaningCore`, `LegacyScanningCore`, `SystemJunkScanning`, `MailJunkScanning`, `TrashJunkScanning`, `DownloadsScanning`, `ScanningCore`, `UninstallerScanning`, …

**Assets:** `JunkCleanupModule.framework/.../Assets.car` — **104** catalog records (REA).

## Observed types (strings / nm)

| Symbol | Role (inferred) |
|--------|-----------------|
| `JunkCleanupModuleCoordinator` | Top-level module orchestration |
| `JunkCleanupCoordinatorType` | Coordinator protocol |
| `SystemJunkFlowCoordinator` | System junk scan/clean UX flow |
| `MailAttachmentsFlowCoordinator` | Mail attachments cleanup |
| `AIClutterFlowCoordinator` | AI-assisted clutter (local model scanning libs linked at app level) |
| `SystemJunkPrivilegedOperationsPerformerAdapter` | Privileged file ops bridge |
| `JunkCleanupScanningTrackerType` | Analytics/telemetry for scans |
| `EmailCleanupPromoRowView` | AppKit view — cross-promo to email module |

## Mechanism hypothesis

1. **Scan phase:** Domain-specific scanning frameworks (`SystemJunkScanning`, etc.) enumerate candidates; results aggregated through `CleaningCore` / `ScanningCore`.
2. **UI phase:** Flow coordinators drive module screens inside `MainAppUI.ModuleView` canvas.
3. **Clean phase:** Deletes/quarantine via `PrivilegedOperationsPerformerService` when sandbox/TCC insufficient — adapter pattern in module.
4. **Sandbox gates:** Strings `module_sandbox_*_folder_access_*` — FDAC prompts before scan (pairs with `Features_FDAResolutionUIKit`).

## Pseudocode (flow-level, inferred)

```swift
final class JunkCleanupModuleCoordinator {
    func start(condition: JunkCleanupMainFlowStartCondition) {
        switch activeFlow {
        case .systemJunk:  SystemJunkFlowCoordinator().run()
        case .mail:        MailAttachmentsFlowCoordinator().run()
        case .aiClutter:   AIClutterFlowCoordinator().run()
        }
        // scanning trackers emit events; privileged adapter executes removals
    }
}
```

## Unknowns

- Exact scan rule sources (likely `PantherDB` / knowledge base services).
- Per-category file predicates inside `SystemJunkScanning` (needs Ghidra on scanning dylibs).
