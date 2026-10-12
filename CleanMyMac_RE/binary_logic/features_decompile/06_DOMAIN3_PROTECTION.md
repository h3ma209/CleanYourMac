# Domain 3 — Protection (IPC and permission checks)

Static evidence only. The file walk and per-file check call are already in [03_SCAN_PIPELINE_CLOSED.md](./03_SCAN_PIPELINE_CLOSED.md) and [02_RealTimeProtection_ARCHITECTURE.md](./02_RealTimeProtection_ARCHITECTURE.md). This note does not reopen `checkURL`, signature bytes, or quarantine. Browser cookies, history databases, and autofill or password vaults were not inspected.

## Targets

| Binary | Role |
|--------|------|
| `/tmp/cmm_slices/IPCProtocol_arm64` | XPC client and server engines |
| `/tmp/cmm_slices/HealthMonitoringAPI_arm64` | Client the UI uses to talk to the health-monitor login item |
| `/tmp/cmm_slices/PermissionsUtils_arm64` | Permission enum and status lookup |
| `/tmp/cmm_slices/SystemPermissionsKit_arm64` | `PANPermissionsService` full-disk status |
| `/tmp/cmm_slices/HealthMonitor_arm64` | Login-item process `CleanMyMac_5_MAS_HealthMonitor` |

Snapshots: `/tmp/cmm_healthapi.rea-snapshot`, `/tmp/cmm_ipc.rea-snapshot`, `/tmp/cmm_perms.rea-snapshot`.

Decompiles (4):

| Function | Address | Binary |
|----------|---------|--------|
| `HealthMonitorClient.start()` | `0x2e1c` | HealthMonitoringAPI |
| `HealthMonitorClient.send(request:)` | `0x3280` | HealthMonitoringAPI |
| `XPCClientEngine.start()` | `0x1eec` | IPCProtocol |
| `PermissionProvider.authorizationStatus(for:)` | `0x2054` | PermissionsUtils |

## Scan pipeline (reused, not re-decompiled)

`MalwareBackgroundScanner.startScanning` at `0x2ae0` in `PantherEngineService` calls `EngineFacade.startMonitoring` or throws `failedToStart`. `makeScanTask` (`PANEngine` `0xbef8`) is a separate object from `makeRemoveTask` (`0xbf34`). The walk is `PANBaseFilesEnumerator.perform` then `PANBaseFilesChecker.perform`, which calls `checkURL` and stops. Detection stays a trait with no rule body.

Moonlock is a gate, not a second scanner. The main app stores `SharedPreferencesKeys.Moonlock.lastSavedMoonlockRTPValue` and presents `MoonlockRTPStateModal*`. The health-monitor binary contains the bundle ids `S8EX82NJP6.com.macpaw.moonlock-mas` and `S8EX82NJP6.com.macpaw.moonlock.interoperability`. When that preference says Moonlock owns monitoring, this app does not start its own monitor. The walker itself is unchanged.

## IPC shape

`IPCProtocol` is an NSXPC stack, not a custom socket. Types present in the binary:

- `XPCClientEngine`, `XPCServerEngine`, `XPCConnection`, `XPCListener`
- `XPCMessage`, `XPCMessageMapper`, `XPCMessageValidator`, `XPCMessageJSONEncoder`
- `XPCRequest` with a `message` and an optional `reply`
- Errors: `XPCConnectionError`, `XPCProtocolError`, `ProtocolVersionMismatch`, `MessageTypeMismatch`, `MailformedTransportMessage`

Server descriptors (symbol signatures, not a guessed mach name):

- `XPCServerEngine.Descriptor.machService(name:requirement:)`
- `XPCServerEngine.Descriptor.xpcService(requirement:)`
- `XPCServerEngine.Descriptor.privilegedHelper(name:authorizedClients:)`
- `XPCServerEngine.makeAnonymous(requirement:)` returns an `NSXPCListenerEndpoint` plus an engine

Client send API:

- `XPCClientEngine.start() -> AnyPublisher<Void, Error>`
- `send(request:) -> AnyPublisher<Void, Error>`
- `send(request:replyWith:) -> AnyPublisher<Reply, Error>`
- `sendSequential(request:replyWith:)`

`HealthMonitoringAPI` sits on that client:

- `HealthMonitorClient.makeDefaultClient(for: String)`
- `start()`, `notifyForeground()`, `notifyBackground()`
- `send<Request: RequestType>(request:) -> AnyPublisher<Request.ReplyType, Error>`
- `subscribe<Event: EventType>(to:) -> AnyPublisher<Event, Never>`
- `onConnect`, `connectionFailureDetected`
- `HealthMonitorServer.start()` and `hasActiveClients`

Process identities (strings in the health-monitor slice):

- `S8EX82NJP6.com.macpaw.CleanMyMac-mas.HealthMonitor`
- `com.macpaw.CleanMyMac-mas.Menu`
- `com.macpaw.CleanMyMac-mas`

`HealthMonitorClient.start` (`0x2e1c`, excerpt `excerpts/HealthMonitorClient_start_0x2e1c.c`) builds a stack value and calls the witness stored at `this+0x30` (function pointer at `+0x10`, context at `+0x18`). That is the client engine start. It does not enumerate files.

`HealthMonitorClient.send` (`0x3280`, excerpt `excerpts/HealthMonitorClient_send_0x3280.c`) loads the reply type witness for `RequestType.ReplyType` and calls the function at `this+0x20`. The request object is forwarded. No JSON keys appear in that body.

`XPCClientEngine.start` (`0x1eec`, excerpt `excerpts/XPCClientEngine_start_0x1eec.c`) is a Combine pipeline setup. The saved pseudocode does not contain a mach-service string. The concrete name passed to `makeDefaultClient(for:)` stays unknown.

No message schema beyond `Encodable` request and `Decodable` reply was recovered. Do not invent field names.

## TCC and permissions

`PermissionsUtils.Permission` cases observed:

- `fullDiskAccess`
- `photos`
- `calendar`
- `contacts`
- `reminders`

Also present: `BluetoothPermissionProviderType`, `FileAccessPermissionProviderType`, `FullDiskAccessObserver`, `FullDiskAccessPermissionProviderType`.

Not observed in `PermissionsUtils` or `SystemPermissionsKit`: `kTCCServiceCamera`, `kTCCServiceMicrophone`, or a camera or microphone enum case. Camera and microphone auditing is not in these binaries.

`PermissionProvider.authorizationStatus(for:)` at `0x2054` (excerpt `excerpts/PermissionProvider_authorizationStatus_0x2054.c`) switches on the permission tag. Tags 0 through 3 call into `MPPermissionsKit`. Any other tag calls `fullDiskAccessStatus`. The kit result is mapped onto an `AuthStatus` byte. Which tag is photos versus contacts was not labeled by the decompiler, so the mapping is not asserted. `SystemPermissionsKit.PermissionsService.fullDiskAccessStatus` is a static that wraps `+[PANPermissionsService fullDiskAccessStatus]`. `requestAuthorizationForPermissionType:withCompletion:` and `requestFullDiskAccess` are selectors. `providePermissionAlertTriggeringURLsWithHomeDirectoryURL:` exists on `SystemPermissionsKit`. This note does not describe how to bypass TCC.

## Frameworks

`IPCProtocol` links Foundation, Security, Combine, `libswiftXPC`. `SystemPermissionsKit` links `PermissionsKit`. `HealthMonitoringAPI` is the facade the main app and the menu login item both link (menu and health-monitor `otool -L` from the earlier login-item pass).

## Control flow

```text
UI or menu
  HealthMonitorClient.makeDefaultClient(for:)
  HealthMonitorClient.start()          // witness at +0x30
    XPCClientEngine.start()
    XPCClientEngine.send(request:replyWith:)
      XPCMessageJSONEncoder
      NSXPCConnection
  HealthMonitorServer (inside HealthMonitor login item)
    hasActiveClients
    owns MalwareBackgroundScanner.startScanning  // see closed pipeline
PermissionProvider.authorizationStatus(for: .fullDiskAccess)
  gates scans that need disk access (Space Lens factory takes the same provider)
```

## Rust sketch (clean room)

- One `tokio` task owns the monitor connection. The UI only receives `onConnect`, `connectionFailureDetected`, and typed events on a channel.
- Encode requests as an explicit enum, not a copy of `XPCMessageJSONEncoder`.
- Full-disk access is a status enum (`unknown`, `granted`, `denied`) checked before a scan task starts. Do not prompt by touching arbitrary home-directory URLs.
- Detection is a trait object. This spec does not implement it.

## Unknown

- Mach service string passed to `makeDefaultClient(for:)`.
- JSON keys inside `XPCMessage`.
- `HealthMonitorClient.send` body if the excerpt is only a witness forward.
- Camera and microphone checks.
- `checkURL` body (intentionally not opened).
