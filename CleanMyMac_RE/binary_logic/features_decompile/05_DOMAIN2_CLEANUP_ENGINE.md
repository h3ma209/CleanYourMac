# Domain 2 — Cleanup engine

Static evidence only. Ghidra via `rea decompile --provider ghidra`. No working deleter, no privileged-helper bypass, no malware signatures, no browser vaults.

## Targets

Arm64 slices under `/tmp/cmm_slices/` (thinned this pass unless already present):

| Slice | Source |
|-------|--------|
| `SystemJunkScanning_arm64` | `SystemJunkScanning.framework` |
| `MailJunkScanning_arm64` | `MailJunkScanning.framework` |
| `CleaningCore_arm64` | `CleaningCore.framework` |
| `LegacyScanningCore_arm64` | `LegacyScanningCore.framework` |
| `ScanningCore_arm64` | `ScanningCore.framework` (symbols only) |
| `JunkCleanupModule_arm64` | `JunkCleanupModule.framework` (symbols only) |

Snapshots created this pass: `/tmp/cmm_cleaningcore.rea-snapshot`, `/tmp/cmm_legacy.rea-snapshot`. Older `binary_logic/snapshots/*` images are different binaries and were not loaded here.

Decompiles (4):

| Function | Address | Binary |
|----------|---------|--------|
| `CleaningCore.FileRemover.remove(at:)` | `0x9c20` | CleaningCore |
| `CleaningCore.MoveToTrashFileRemover.remove(at:)` | `0x9d5c` | CleaningCore |
| `-[CMCompoundScanTask scan]` | `0x4c4c` | LegacyScanningCore |
| `-[CMURLIterator enumerate:withHandler:]` | `0x5484` | LegacyScanningCore |

Selector targets below were resolved from the `__objc_stubs` those functions `bl` into, plus `__objc_selrefs`. That is the same functions, not extra decompiles.

## Path and folder-name constants

All of the following are cstrings in **`SystemJunkScanning_arm64`** unless noted. Nothing here is inferred from the product name.

**Directory strings**

- `Data/Library/Caches`
- `Data/Library/Logs`
- `~/Library/Application Support/Adobe/Common/Media Cache Files`
- `~/Library/Application Support/Spotify/PersistentCache/Storage`
- `~/Library/Application Support/Spotify/prefs`
- `~/Library/Application Support/Telegram Desktop/tdata/user_data`
- `~/Library/Containers/org.telegram.desktop/Data/Library/Application Support/Telegram Desktop/tdata/user_data`
- `~/Library/Group Containers/6N38VWS5BX.ru.keepcoder.Telegram`
- `~/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/2.0b4.0.9`
- `~/Library/Containers/com.tencent.tenvideo/Data/Library/Application Support/Download/video`
- `~/Library/Containers/com.tencent.QQMusicMac/Data/Library/Application Support/QQMusicMac/iMusic`
- `~/Library/Containers/com.youku.mac/Data/Documents/YTCORE_PLAYER_CACHE`
- `~/Library/Containers/com.apple.Photos/Data/Library/Application Support/Photos Desktop/0`
- `~/Library/Preferences/com.apple.LaunchServices/com.apple.LaunchServices.plist`
- `~/Library/Preferences/com.apple.LaunchServices.QuarantineEvents`
- `~/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2`
- `/Library/SystemMigration/History`
- `/Library/Developer/PrivateFrameworks/CoreSimulator.framework/Versions/A/Resources/bin/simctl`
- `/System/Library/CoreServices/SystemVersion.plist`
- `/%@/QuarantineRoot/private/var/db/dyld`

**Folder-name / suffix signatures in the same binary**

- `DiagnosticReports`
- `DiagnosticMessages`
- `Application Support`
- `lproj`, `.lproj`
- `.lockfile` (filename token, not a lock API)
- `com.apple.quarantine`
- `com.apple.CoreSimulator.SimRuntime.`

**Scan-identifier cstrings** (category keys, not paths): `CMSystemJunkScanIdentifierUserCachesCompound`, `…SystemCachesCompound`, `…SandboxContainersCache`, `…UserLogs`, `…UserLogsCompound`, `…SystemLogsCompound`, `…LocalLogs`, `…AppleLogs`, `…DiagnosticLogs`, `…SandboxLogs`, `…QuarantineLogs`, `…BrokenPreferences`, `…XcodeDerivedData`, `…XcodeCoreSimulatorCaches`, `…XcodeDeviceLogs`, `…XcodeModuleCaches`.

**Mail.** `MailJunkScanning_arm64` has `CMMailTask`, `CMMailboxAttachmentsModel`, `MailJunkScanTasksFactory`, `enumerateAttachmentsWithHandler:canContinueHandler:`, `CMMailModuleScanIdentifierMail`. **No** `~/Library/Mail` (or any mailbox directory) string. Mailbox paths: **not observed**.

**CleaningCore / LegacyScanningCore:** no junk directory cstrings.

A bare `Library/Caches` or `~/Library/Logs` path is **not** a standalone cstring. The cache/log locations that are present are the `Data/Library/Caches` and `Data/Library/Logs` fragments plus the `~/Library/…` list above.

## Who starts a scan

**Module shell (symbols, JunkCleanupModule):** `JunkCleanupModuleCoordinator.makeFlowCoordinator(for:navigator:delegate:scanLocationURL:)` and `runTrashBinCustomFlow()`. Builder takes `PrivilegedOperationsPerformerService.PrivilegedOperationsPerformerServiceType?` and `homeFolderSandboxAccessProvider: ModuleCore.FileSystemAccessProviderType?`.

**Task factory (symbols, SystemJunkScanning):** `SystemJunkScanTasksFactory` returns `LegacyScanningWrapperCore.LegacyScanTaskWrapper<…>`. Methods that exist: `userCache`, `systemCache`, `systemLogs`, `userLogs`, `brokenPreferences`, `brokenStartupItems`, `documentsVersions`, `deletedUsers`, `mobileBackups`, `applicationLanguages`, `xcodeDerivedData`, `coreSimulatorCache`, `iosDeviceLogs`, `moduleCache`, and others. `userCache` / `systemCache` / `systemLogs` / `brokenStartupItems` take `CMSystemJunkPrivilegedOperationsPerformer`.

**Enumerator (decompiled):** `CMURLIterator` (`LegacyScanningCore`) method `enumerate:withHandler:`.

```
enumerate(urls, handler):
    progress.addPendingUnitsCount(urls.count)
    for url in urls:                      # NSFastEnumeration
        progress.sleepThreadWhilePaused()
        if progress.isCancelled: return
        progress.addUnitsCount(1)
        if filter == nil || filter(url):
            size = CMByteSizer.byteSizeAtURL(url)
            progress.addSize(size)
            progress.setPath(url.path)
            handler(url, size)
```

**Compound task (decompiled):** `-[CMCompoundScanTask scan]` calls `cmm_forEach_` on its tasks, then if progress is not cancelled walks tasks again:

```
scan:
    for task in tasks:
        progress.sleepThreadWhilePaused()
        if progress.isCancelled: break
        result = task.scan()
        append result.items
        if result is CMCompoundScanResult:
            append result.enclosedScanResultErrors
    return CMCompoundScanResult(identifier, items, errors)
```

**Queue (symbols, ScanningCore, not decompiled):** `GenericScanner.scan()` / `scan(completion:)` drive `ProcessingQueue`. That type’s fields include an `NSOperationQueue` (`So16NSOperationQueueC` is also a cstring in ScanningCore), `maxConcurrentOperationCount`, `NSRecursiveLock`, publishers `onQueueStart`, `onTaskStarted`, `onTaskFinished`, `onQueueComplete`. `ProcessingQueue.run` and `runSync` exist. `ControlObject.pause()` and `onceUnpaused` exist. `ExecutionStatus` field metadata is a 2-case enum: `failed`, `finished` (only those two cases in that record).

ObjC scan tasks present in SystemJunkScanning include `CMCacheScanTask`, `CMUserCachesScanTask`, `CMSystemCachesScanTask`, `CMLogsScanTask`, `CMUserLogsScanTask`, `CMSystemLogsScanTask`, `CMSandboxContainersCachesScanTask`. `CMURLIterator` is the shared walker; per-task directory lists live in knowledge-base objects (`CMCachesKnowledgeBase`, `CMLogsKnowledgeBase`) whose tables were **not** dumped.

## File lock / “file in use”

**Not observed:** the string `file in use`, `NSFileCoordinator`, `flock`, `EBUSY`.

**Observed:** `CleaningCore.FileRemover.remove(at:)` loads the object at `this+0x10` (initializer stores `FileManagerService.FileManagerProviderType`) and calls selector `removeFileAndHandleLockAtPath:error:` (objc stub `0x18020` → selref `0x252d8`). The body of that selector is in `FileManagerService`, not decompiled.

Same CleaningCore image also has selrefs `tryToUnlockFileIfNeededAtPath:recursively:`, `lock`, and `unlock`. `lock` / `unlock` are not tied to a file URL by these four decompiles (`NSRecursiveLock` is imported and used as a queue lock in ScanningCore symbols). `.lockfile` in SystemJunkScanning is a filename token.

SystemJunkScanning also contains the selector string `tryToUnlockFileIfNeededAtPath:recursively:`. No decompile shows it running.

So: a path-based unlock-then-remove selector is present and `FileRemover` calls the lock-handling remove. A distinct “file in use” check is **unknown**.

## Trash vs unlink

Two remover types, both `remove(at: URL) throws`:

| Type | What the body calls |
|------|---------------------|
| `MoveToTrashFileRemover` (`0x9d5c` → `0xa3ac`) | `NSFileManager` class ref, stub `defaultManager` (`0x17fa0`), then `trashItemAtURL:resultingItemURL:error:` (stub `0x18060`) |
| `FileRemover` (`0x9c20`) | `removeFileAndHandleLockAtPath:error:` on the injected file-manager object. Does **not** call `unlink` or `trashItem` in this function |

Selrefs `removeItemAt:error:` and `removeItemAtPath:error:` exist in CleaningCore. Which function sends them was **not** decompiled.

Other remover symbols, not decompiled: `DebugFileRemover.remove(at:)`, `ConfigDependentFileRemover.remove(at:)`, `FilesRemovalTask`, `PrivilegedFileRemover.remove(at:)`.

`unlink(2)`: **not observed**.

## Permission model (framework level only)

No escalation procedure.

- Sandbox folders: `JunkCleanupModuleBuilder` and `SmartCareModuleBuilder` take `ModuleCore.FileSystemAccessProviderType` (`homeFolderSandboxAccessProvider`; Smart Care also has `applicationsFolderSandboxAccessProvider`).
- Privileged file ops: CleaningCore links `PrivilegedOperationsPerformerService.framework`. `PrivilegedFileRemover.init` takes `FileRemoverType`, `FileSystemPrivilegedOperationsClientType`, and an `authorizeAgentBlock`. Nested error `NotAuthorizedPrivilegedAgentError` has a `path: String`. Source-path cstring: `CleaningCore/PrivilegedFileRemover.swift`.
- System junk tasks that need it take `CMSystemJunkPrivilegedOperationsPerformer` (`SystemJunkScanTasksFactory.userCache` / `systemCache` / `systemLogs` / `brokenStartupItems`).
- `LogsRemover.init` takes a `PrivilegedOperationsPerformerService` client plus `authorizeAgentBlock` (symbol only).

`PrivilegedFileRemover.remove` was not one of the four decompiles. Do not treat the initializer signature as a bypass.

## Clean-queue states (CleaningCore field metadata)

A 3-case enum at file `0x1b834`: `finished`, `idle`, `started`. Symbol `CleanQueueTaskState.State.isFinished` exists. The field record’s mangled type did not demangle; tying this enum to `CleanQueueTaskState.State` is an inference from that symbol plus the case list.

A 4-case enum at `0x1b744`: `success`, `partialSuccess`, `empty`, `failure`. Symbol `CleanQueueResult.CleanQueueOutcome` exists (`hasRemovedItems`, `hasFailedResults`). Same caveat: record-to-type binding is inferred.

`TasksProcessor` fields include `queue`, `cleanQueueStateUpdateLock`, `_taskStatePublisher`.

## Clean-room Rust sketch

Not a copy of MacPaw source. No unlock implementation and no helper authorization.

```rust
// UI thread: watch channel of ScanPhase.
// tokio::spawn one scan task.
// walkdir walks only the path list compiled from the constants above.
// Filter is a predicate trait (the binary’s CMURLIterator filter block).
// Pause: AtomicBool, same checkpoint as sleepThreadWhilePaused.
// Removal is a trait object selected by config:
//   TrashRemover  -> `trash` crate (trashItem equivalent)
//   DeleteRemover -> std::fs::remove_file behind an explicit user confirm
// Privileged paths stay "needs user-granted helper"; this sketch does not spawn one.
```

Crates: `tokio`, `walkdir`, `trash`, `ignore` if a skip list is added later. `NSOperationQueue` in the original maps to a bounded `tokio` worker set (`maxConcurrentOperationCount`), not to a second UI thread.
