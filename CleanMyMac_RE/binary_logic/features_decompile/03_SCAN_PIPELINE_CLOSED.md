# Scan pipeline — control flow closed

Ghidra 12.1.4 via REA `--provider ghidra` on existing arm64 slices. Snapshots reused: `/tmp/cmm_msr_arm64.rea-snapshot`, `binary_logic/snapshots/PantherEngineService`, new `binary_logic/snapshots/PANEngine_arm64` (no prior PANEngine snapshot). Pseudocode is not original source.

| Slice | SHA-256 | Image base |
| --- | --- | --- |
| `MalwareScanningRoutines` arm64 | `066327bee0da9fe52b87db59043d7b9abafa6878e48c892c5379e4abe4906a4c` | `0x0` |
| `PANEngine` arm64 | `7c2e328ccfc09e5ffbcefdc5ca0115b8575eea9796dc40a16772768fa1280b15` | `0x0` |
| `PantherEngineService` arm64 | `10c6b6d029ba6c33bc6b1c78fe331023a35ef7863de083c7a8c16ce1cb0de311` | `0x0` |

## Known end to end

1. **Start.** `MalwareBackgroundScannerType.startScanning` thunk `0x68cc` jumps through the witness table. Body is `0x2ae0` (wrapper `0x2b90`) on `_MalwareBackgroundScanner`. It weak-loads a settings provider and a reports observer and calls `EngineFacade.startMonitoring`. Failure throws `MalwareBackgroundScanner.Errors`. `0x2bb0` calls `stopMonitoring`. This step does not enumerate files.
2. **Scan task.** `EngineFacade.makeScanTask` `0xbef8` → `0xf4f0`. If facade `engineState` is below 2, `PANMalwareScannerTaskBuilder` returns a `PANMalwareScannerTask` for the `PANScanConfiguration`. State 2 returns nil. The builder body was not kept.
3. **Walk.** `PANBaseFilesEnumerator.perform` `0x4d9c` builds a URL array and runs the file-tree enumerator. `iterateDirectoryWithURL:` `0x4e84` asks the delegate `checkDirectoryDescendantsWithURL:`. On `PANURLBaseMalwareDetectionTask` that is `0x23f74`, which descends unless `shouldDrop…` `0x23df8` returns true. The base drop helper always returns false. `iterateFileWithURL:` `0x4eec` symlink-resolves, appends, and flushes a chunk with finale false when the chunk exceeds `maxFileURLsChunkSize`. `perform` flushes the tail with finale true. `filterItemWithURL:` `0x500c` forwards to the delegate; the task method `0x23f8c` only reports the path.
4. **Check.** `PANBaseFilesChecker.perform` `0x1de00` pause/cancels, skips non-checkable URLs, and calls `detectionService checkURL:executionControl:dataSource:error:` with the URL, the execution control, a **nil data source**, and an error pointer. Nil detection plus an error notifies the error. Otherwise the delegate gets either `handleCheckedFileURL:` or `handleMalwareDetection:fileURL:`.
5. **Remove.** `EngineFacade.makeRemoveTask` `0xbf34` → `0xbfb4` is a different object: `PANMalwareRemoverTask` from `PANRemovalConfiguration` and the malware array, behind the same engine-state gate. The scan loop does not delete.

## Still unknown

- **`checkURL` body.** Not decompiled. No signature database, YARA, or rule bytes were recovered, on purpose.
- Subclass overrides of `shouldDropCheckingDescendants` (other tasks may skip directories).
- `enumerator:catchError:` `0x5068`.
- `MalwareMonitorService.startMonitoring` and `didScanPath` inside `MalwareMonitoringRoutines`.
- Remover task deletion / quarantine steps (`PANDiskURLMalwareItemRemovalService` and `PANMalwareRemoverTaskBuilder` bodies).
- Which concrete monitor (`PANObservingBasedMonitor` vs `SchedulerBasedMonitor`) a given preferences bag selects. Static analysis did not observe a live scan.
