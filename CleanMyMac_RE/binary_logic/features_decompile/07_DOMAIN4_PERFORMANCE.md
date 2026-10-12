# Domain 4 — Performance (agents, maintenance, memory)

Static evidence only. No shell recipe was copied out of the binary. If a maintenance action is only a task name plus a privileged-agent launch, that is all this note claims.

## Targets

| Binary | Role |
|--------|------|
| `/tmp/cmm_slices/PerformanceScanning_arm64` | Launch-agent and login-item scan tasks |
| `/tmp/cmm_slices/PerformanceModule_arm64` | UI flows and remover tasks |
| `/tmp/cmm_slices/PerformanceService_arm64` | DNS flush and Spotlight reindex runners |

## What the module actually contains

`PerformanceModuleBuilder.init` takes `reindexSpotlightObserver: PerformanceService.ReindexSpotlightObserverType`, `privilegedAgent: PrivilegedOperationsPerformerServiceType?`, and `fullDiskAccessProvider`. Linked from `PerformanceScanning`: `ApplicationsService`, `FileManagerService`, `KnowledgeBaseService`, `PerformanceService`, `ScanningCore`, `SharedUtils`, Foundation, `libswiftDispatch`.

Decompiles (2), snapshot `/tmp/cmm_perfsvc.rea-snapshot`:

| Function | Address |
|----------|---------|
| `FlushDNSCacheTaskRunner.launch(value:)` | `0x5454` |
| `ReindexSpotlightTaskRunner.launch(value:)` | `0x7b48` |

### Launch agents and login items

Classes in `PerformanceScanning`:

- `LaunchAgentsKnowledgeBase`, `LaunchAgentsScanTask`
- `LoginItemsKnowledgeBase`, `LaunchServicesLoginItemsScanTask`
- `LoginItemsScanTaskFactory`
- `BackgroundItemsScanTaskFactory.makeLaunchAgentsScanTask()` at `0x2200`
- Selector `removeLoginItemAtPath:error:`
- Strings `launchAgents`, `launchServicesLoginItems`, `validLoginItemsList`

`PerformanceModule` then maps those results:

- `LoginItemsFlowCoordinator`, `LoginItemsScannerConfig`
- `LoginItemsRemoverTask`, `LoginItemsOperationFactory`
- `LoginItemsConfigDependentsRemover`
- `PerformanceDetailsNode<LoginItemModelType, Node>`

No `~/Library/LaunchAgents` or `/Library/LaunchDaemons` cstring was recovered in this pass. The scan is a named task (`LaunchAgentsScanTask`) backed by a knowledge base, not an inlined directory list in the strings that were searched. Treat the standard launch-agent directories as not observed until a decompile of `LaunchAgentsScanTask` shows them.

### Maintenance tasks

`PerformanceScanning.MaintenanceScanTaskFactory`:

- `makeFlushDNSCacheTask()` at `0x46dc`
- `makeReindexSpotlightTask()` at `0x49b8`

`PerformanceService` runners:

- `FlushDNSCacheTaskRunner.init(privilegedAgent:)` and `launch(value:)` at `0x5454`
- Source path string `PerformanceService/FlushDNSCacheTaskRunner.swift`
- Failure strings: `Flush DNS Cache script failed with error:` and `Flush DNS Cache script completed with success but returned false as result`
- `ReindexSpotlightTaskRunner` with the same script-result strings for Spotlight
- `ReindexSpotlightObserver` wraps `CMSpotlightIndexingObserver` and publishes `AnyPublisher<Bool, Never>` via `spotlightIndexingObserverDidChangeState:`

The runner is constructed with the privileged-operations client. `FlushDNSCacheTaskRunner.launch` at `0x5454` (excerpt `excerpts/FlushDNSCacheTaskRunner_launch_0x5454.c`) only calls `FUN_00005634` and returns. `ReindexSpotlightTaskRunner.launch` at `0x7b48` (excerpt `excerpts/ReindexSpotlightTaskRunner_launch.c`) only calls `FUN_00007cc4`. The script body is in those callees, not in `launch`. It was not opened. Error strings say a script ran and returned a bool. `mdutil`, `dscacheutil`, and `atsutil` do not appear as strings in `PerformanceService` or `PerformanceScanning`. Font-cache clearing was not observed.

Do not reconstruct the script. A replacement should call documented system APIs or refuse the action.

### CPU and memory

Not observed: `host_statistics`, `task_info`, `processor_set`, or a sampling loop that reports live CPU percent.

Observed memory path: ObjC `+[CMRAMCleaner _cleanMemoryWithCancellationBlock:]`, string `physicalMemory`, and `Did fail to free up memory with exception %@`. That is a one-shot cleaner with a cancellation block, not a stats timer.

## Control flow

```text
PerformanceModuleBuilder
  privilegedAgent
  reindexSpotlightObserver
MaintenanceScanTaskFactory.makeFlushDNSCacheTask / makeReindexSpotlightTask
  FlushDNSCacheTaskRunner.launch(model)     // privileged agent, script result bool
  ReindexSpotlightTaskRunner.launch(model)
  ReindexSpotlightObserver.onValueChange    // Bool publisher
BackgroundItemsScanTaskFactory.makeLaunchAgentsScanTask()
  LaunchAgentsScanTask + LoginItemsKnowledgeBase
  LoginItemsRemoverTask                     // removeLoginItemAtPath:error:
CMRAMCleaner._cleanMemoryWithCancellationBlock:
```

## Rust sketch (clean room)

- `plist` to read agent plists only after the directory list is proven. Until then, model login items as a scan-task result type, not a hardcoded path walk.
- `sysinfo` on a `tokio` interval if the replacement wants CPU and memory. That loop is new. It is not a port of a loop found here.
- DNS flush and Spotlight reindex stay behind an explicit privileged helper with a typed error (`FlushDNSCacheError`, `ReindexSpotlightError`). Do not shell out to copied scripts.
- Publish `Bool` indexing state on a channel the way `ReindexSpotlightObserver.onValueChange` does.

## Unknown

- Directory list inside `LaunchAgentsScanTask`.
- Script body inside `FlushDNSCacheTaskRunner.launch` (not decompiled in the first four-function budget; symbol and error strings only).
- Font cache, `mdutil`, `dscacheutil`, `atsutil`.
- A CPU sampling loop.
