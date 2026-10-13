# Domain 6 — Space Lens (size tree)

Static evidence from Swift type metadata and method signatures in `SpaceLensScanning`. No bubble or treemap type was found, so the UI weight model is not claimed.

## Targets

| Binary | Role |
|--------|------|
| `/tmp/cmm_slices/SpaceLensScanning_arm64` | Scan task, filesystem node, deletion planner |
| `/tmp/cmm_slices/SpaceLensModule_arm64` | Module UI. No `bubble`, `treemap`, `weight`, or `proportional` strings |

`SpaceLensScanning` links `FileManagerService`, `PermissionsUtils`, `ScanningCore`, `SystemUtils`, `CombineUtils`, AppKit, Foundation.

Decompiles (2), snapshot `/tmp/cmm_spacelens.rea-snapshot`:

| Function | Address |
|----------|---------|
| `SpaceLensDeletionPlanner.units(for:shouldSpare:)` | `0x5c38` |
| `SpaceLensScanTasksFactory.createScanTask(directory:strategy:)` | `0xd060` |

Earlier module notes in [03_space_lens_smart_care.md](../03_space_lens_smart_care.md) name the UI coordinator. This file is the node the scan produces.

## Node

`SpaceLensFileSystemEntity` fields proven by getters and `init`:

| Field | Type | Address of getter or init |
|-------|------|---------------------------|
| `path` | `String` | getter `0x3c20` |
| `size` | `UInt64` | getter `0x3c4c` |
| `isDirectory` | `Bool` | getter `0x3c54` |
| `isPurgeable` | `Bool` | getter `0x3c5c` |
| `cloudStatus` | `CloudStatus` | getter `0x3c64` |
| `children` | `[SpaceLensFileSystemEntity]` | getter `0x3c6c`, setter `0x3c98` |
| `init(path:size:isDirectory:isPurgeable:cloudStatus:)` | | `0x3c74` |

Derived counts, not stored as the primary payload: `leavesCount` (`0x3fe0`), `removableLeavesCount` (`0x4460`), `cloudFilesCount` (`0x4970`), `cloudLocalFilesCount` (`0x4e80`).

`ScanProgressValue` is the live update: `init(scannedSize: UInt64, currentPath: String?)` at `0x3558`, `activePath` getter at `0x3914`.

`CloudStatus` reflection names, in order: `notCloud`, `cloudEvicted`, `cloudLocal`. Classifier `FUN_0000236c` returns `0` / `1` / `2`. Mapping those names onto those returns is inferred. `SpaceLensCloudScannerStrategy` case names were not recovered. Raw byte `1` selects the `lstat` / `SF_DATALESS` path.

## Walk and handoff

`SpaceLensScanTasksFactory`:

- `init(fullDiskAccessPermissionProvider:fileManager:)` at `0xd050`
- `createScanTask(directory: URL, strategy: SpaceLensCloudScannerStrategy)` at `0xd060` (excerpt `excerpts/SpaceLensScanTasksFactory_createScanTask_0xd060.c`)
- Implementation class `SpaceLensScanTask`
- Helpers: `PathFilter`, `ProgressTracker`, `VolumeChecker`, `CloudStorageHelper`

`createScanTask` writes these `SpaceLensScanTask` fields from the factory and the directory argument: `directory`, `filter`, `fileManager`, `cloudStorageHelper`, `volumeBackupSizeCacher`, `fileSystemType`, `progressTracker`. It sets `maxConcurrentScans` to **4**. The walk itself is not in this factory method. Four concurrent scans is the parallelism bound that was stored.

Full-disk access is a constructor dependency. The task is per directory URL, not one implicit whole-disk walk.

`SpaceLensDeletionPlanner.Unit` is a flat row, not the tree:

- `init(path:size:removableLeavesCount:)` at `0x3558` (same address slot as the progress init in the symbol table; confirm in a disassembler before calling it — both symbols demangle to `0x3558`, which means one of them is an alias or the dump collided; the entity init at `0x3c74` is unambiguous)
- `units(for:shouldSpare:) -> [Unit]` at `0x5c38` walks a `SpaceLensFileSystemEntity` and returns removable leaves

The tree is the entity. The planner flattens it to path, byte size, and a removable-leaf count. No serialization format (JSON, plist, or protobuf) was observed. No view type with bubble weights was observed in `SpaceLensModule` strings.

## Control flow

```text
SpaceLensScanTasksFactory(fullDiskAccess, fileManager)
  createScanTask(directory, cloudStrategy) -> SpaceLensScanTask
    PathFilter + CloudStorageHelper + ProgressTracker
    maxConcurrentScans = 4 (stored; no second read in this slice)
async FUN_00007b54
  FUN_00008134 lists one directory
    skip customIgnorePaths, "/.nofollow", "/.nofollow/"
    file size keys: NSURLFileSizeKey and NSURLTotalFileAllocatedSizeKey
    cloud: FUN_0000236c, or lstat SF_DATALESS when strategy byte is 1
  sum child sizes (stride 0x28), then progress task-switch
leavesCount / removableLeavesCount are later tree reductions
SpaceLensDeletionPlanner.units(for:shouldSpare:) -> [Unit]
UI: SpaceLensViewController + BubbleUIConfig + BubblesPackingLayout
```

Node bytes, proven by the walk store and the count getters:

| Offset | Field |
|--------|--------|
| `+0x00` | `path` |
| `+0x10` | `size` |
| `+0x18` | `isDirectory` |
| `+0x19` | `isPurgeable` |
| `+0x1a` | `cloudStatus` |
| `+0x20` | `children` |

`leavesCount`: purgeable node returns 0 and is not opened. Non-purgeable file returns 1. Non-purgeable directory sums children. Checked add.

`removableLeavesCount`: same, except a file counts only when `cloudStatus != 1` (evicted / not-downloaded / dataless).

Directory `size` in the async caller is the sum of child `size` fields. Which of the two URL size keys is stored on a leaf was not recovered.

## Rust

Recovered flow, not original source: [excerpts/space_lens_flow.rs](excerpts/space_lens_flow.rs).

## Unknown

- Which of `fileSize` and `totalFileAllocatedSize` is written into `Entity.size`.
- `SpaceLensCloudScannerStrategy` case names. Only raw byte `1` vs other is visible.
- Where `isPurgeable` becomes true. The leaf store in `FUN_00008134` writes it false.
- Who reads `maxConcurrentScans` after the store at `0xd3bc`.
- `0x3558` is still three symbols on one address: unit `init`, progress `init`, `scannedSize` getter.
- `FUN_00005cb8` (deletion flatten, 2164 bytes) was not expanded. `units()` in the Rust file follows the public signature only.
- `BubblesPackingLayout` radius formula. Bubble classes are in `SpaceLensUI`; packing math was not decompiled.
