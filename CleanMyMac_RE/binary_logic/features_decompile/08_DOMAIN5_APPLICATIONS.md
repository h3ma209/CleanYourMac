# Domain 5 — Applications (bundle id, leftovers, reset)

Static evidence only. No uninstaller was written. `~/Library/Cookies/%@.binarycookies` below is a leftover path template stored in `UninstallerScanning`. It was not opened as a cookie database.

## Targets

| Binary | Role |
|--------|------|
| `/tmp/cmm_slices/UninstallerScanning_arm64` | Leftover fetch, knowledge bases, reset-caches task |
| `/tmp/cmm_slices/ApplicationsModule_arm64` | Uninstall flow ids and module config |

`UninstallerScanning` links `ApplicationsService`, `FileManagerService`, `PermissionsUtils`, `ScanningCore`, `SharedUtils`, AppKit, Foundation.

Decompiles (2), snapshot `/tmp/cmm_uninstall.rea-snapshot`:

| Function | Address |
|----------|---------|
| `LeftoversFetcher.fetchLeftovers()` | `0x1f97c` |
| `UninstallerScanTaskFactory.resetCaches()` | `0x2f330` |

## Bundle identity

- Cstring `CFBundleIdentifier` and `CFBundleName` in `UninstallerScanning`.
- `ApplicationsModule.ModuleConfig.init(mainBundleIdentifier:appInfoServiceConfig:systemInfo:uninstaller:)`.
- `ModuleConfig.Uninstaller.init(ignoredAppBundleIDs: Set<String>)`.
- `ApplicationCategoryResourcesLoadingModel.prioritizeApplications(withBundleIDs:)`.
- Collection helper `indexOfAppBundle(bundleID:bundlePath:)`.

App records use ObjC `CMApplicationInfo`. Vendor-specific loaders are injected into `UninstallerScanTaskFactory`: Steam, Blizzard, JetBrains Toolbox, plus `AudioPluginLoaderType`.

## Leftover search roots (strings in UninstallerScanning)

These are the path templates actually present:

- `/Library/Application Support`
- `Library/Application Support`
- `Library/Application Support/CrashReporter`
- `Application Support/CrashReporter/`
- `~/Library/Application Support/Adobe/AIR/ELS`
- `/Library/Application Support/Avid/Audio/Plug-Ins`
- `Library/Preferences`
- `userPreferences`, `allPreferencesNames`, class `CMPreferencesFilesFetcher`
- `~/Library/Preferences/SDMHelpData/Other`
- `Library/Saved Application State`
- `~/Library/Saved Application State`
- `Library/Caches`
- `Library/Logs`, `/Library/Logs`
- `Library/Cookies`
- `~/Library/Cookies/%@.binarycookies`
- `~/Library/Containers/`
- `/Library/PDF Services`
- `/Library/Documentation/Applications`
- `Library/%@`

`LeftoversFetcher` keeps two format strings: `userApplicationTemplate` and `sharedApplicationTemplate` (addresses `0x1d61c` and `0x1d600` are the initialization thunks; the format text was not dumped). `fetchLeftovers() -> [CMApplicationInfo]` is at `0x1f97c`. `UninstallerScanTaskFactory.leftoversDiscoveryTask()` is at `0x305cc`.

`fetchLeftovers` at `0x1f97c` (excerpt `excerpts/LeftoversFetcher_fetchLeftovers_0x1f97c.c`) walks a table of 0x30-byte records. For each record it calls `isAppIgnoredWithBundleId_`. If the id is not ignored it allocates `CMApplicationInfo` with `initWithBundleID:appName:` and then `setPath:`. Ignored ids are skipped. The path templates above are the search roots; this function turns a row into an app record. It does not itself delete files.

## Reset versus uninstall

Two different entry points:

- Uninstall flow id: `ApplicationsModule` tagged string `uninstallApp`, plus `uninstallAppsFlow(_: [URL], kind:customDependenciesResolverFactory:)`. `ApplicationsModuleBuilder` takes `privilegedAgent` and `applicationSelfRemoverService`. That path removes the app the user selected.
- Reset caches: `UninstallerScanTaskFactory.resetCaches() -> some` at `0x2f330` (excerpt `excerpts/UninstallerScanTaskFactory_resetCaches_0x2f330.c`). The body allocates scan-task objects from factory fields at `+0x10` and `+0x28` and does not call `removeItem` or `trashItem`. A second factory init takes `directoryProvider: CMApplicationSupportDirectoryProviderType` and does not take the Steam or Blizzard loaders. That is the user-state sweep. Nothing in the decompile deletes the `.app` bundle.

`ignoredAppBundleIDs` is how the module skips bundle ids (the host app’s own id is the obvious use; the set contents were not dumped).

## Control flow

```text
ApplicationsModuleConfig.uninstallAppsFlow(urls)
  ApplicationsModuleBuilder(privilegedAgent, applicationSelfRemoverService)
  UninstallerScanTaskFactory.leftoversDiscoveryTask()
    LeftoversFetcher.fetchLeftovers() -> [CMApplicationInfo]
    knowledge base + path templates (Application Support, Preferences, Saved Application State, Caches, Logs, Containers)
  LeftoverUninstallerApplication(appInfo:isSystem:isAggressor:)
UninstallerScanTaskFactory.resetCaches()
  directoryProvider (application support)
  does not take the app-bundle loaders
```

## Rust sketch (clean room)

- Read `Contents/Info.plist` `CFBundleIdentifier` with `plist`.
- Walk only the roots listed above, under the user home and `/Library`, with a depth cap.
- Match leftovers with an explicit rule you choose (filename contains bundle id, or a local table). Do not copy an unseen knowledge base.
- Reset deletes files under Preferences, Saved Application State, and Caches for that bundle id and leaves the `.app` in `/Applications`.
- Uninstall is a separate command that trashes the bundle URL. Use the `trash` crate. Privileged deletes stay behind a helper you own, not `PrivilegedOperationsPerformerService`.

## Unknown

- The two leftover format strings (`userApplicationTemplate`, `sharedApplicationTemplate`).
- Whether match is substring, prefix, or a shipped knowledge-base row.
- Body of `resetCaches()` (opaque `some` return). It was not decompiled here.
- Which call sends the app bundle to trash versus the leftover files.
