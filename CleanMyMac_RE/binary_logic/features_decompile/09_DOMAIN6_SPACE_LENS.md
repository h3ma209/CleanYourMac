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

`CloudStatus` and `SpaceLensCloudScannerStrategy` are enums (equality and hash only; case names were not dumped).

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
    PathFilter + VolumeChecker + ProgressTracker
    emits ScanProgressValue(scannedSize, currentPath)
    builds SpaceLensFileSystemEntity
      path, size, isDirectory, isPurgeable, cloudStatus, children
SpaceLensDeletionPlanner.units(for:shouldSpare:)
  -> [Unit(path, size, removableLeavesCount)]
UI reads the entity tree or the unit list
  bubble layout: not observed
```

## Rust sketch (clean room)

- `tokio::spawn` one walk per directory the user picked. Send `ScanProgress { scanned_size: u64, current_path: Option<String> }` on a channel.
- Owned tree:

```rust
struct Node {
    path: String,
    size: u64,
    is_directory: bool,
    is_purgeable: bool,
    children: Vec<Node>,
}
```

- A second pass computes leaf counts. Do not store them as the source of truth.
- The UI only reads `size` values. Proportional bubbles are a view over that array, not something this binary was shown to serialize.
- Skip the walk when full-disk access is denied, matching the factory’s permission parameter.

## Unknown

- How `size` is summed (catalog attributes versus recursive add). `createScanTask` only builds the task object.
- `CloudStatus` case names.
- Whether `0x3558` is the unit init or the progress init. The two demangled symbols share that address in `nm` output.
- Any frontend bubble structure.
