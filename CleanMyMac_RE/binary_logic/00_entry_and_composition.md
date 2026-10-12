# Entry point & application composition

**Provenance:** `rea inspect-macho`, `otool -tv`, `swift demangle`. **Not** Hopper/Ghidra pseudocode.

## Mach-O facts (arm64 slice)

| Field | Value |
|-------|--------|
| Entry file offset | 18040 (`LC_MAIN entryoff`) |
| Virtual entry | `0x100004678` (`__TEXT,__text`) |
| UUID | `9E01313F-D270-368D-87FD-E30D0B9D0932` |
| Min OS | 11.0 · SDK macOS 26.5 |
| Dependencies | 147 dylibs/frameworks (94+ `@rpath` MacPaw modules) |

## External symbols at entry (disassembly)

| VA | Demangled / symbol |
|----|---------------------|
| `0x100004694` | `SharedUtils.ComposedApplication` metadata |
| `0x1000046fc` | `NSProcessInfo.isCommandLineRun` |
| `0x100004738` | `NSProcessInfo.isSandbox` |
| `0x10000478c` | `NSProcessInfo.isRunningTests` |
| `0x1000047a0` | `NSProcessInfo.isRunningSwiftUIPreviews` |
| `0x100004804` | ObjC `CMIssuesReportingApplication` |
| `0x100004898` | `NSApplication.setup(with: ComposedApplication)` |
| `0x1000048b0` | `AppKit.NSApplicationMain` |

String literal pool references `CleanMyMac/main.swift`.

## Reconstructed pseudocode

```swift
// LC_MAIN @ 0x100004678 — CleanMyMac/main.swift
@main
struct Main {
    static func main() {
        var composed = ComposedApplication.self // stack metadata / config builder

        let pi = NSProcessInfo.processInfo
        // stored @ 0x100557148 (global slot in __DATA)

        if pi.isCommandLineRun {
            // WorkspaceManagement + CLI ComposedApplication factory @ 0x100004ce0
            return
        }
        if pi.isRunningTests { return /* test host */ }
        if pi.isRunningSwiftUIPreviews {
            if NSApp != nil { return }
        }

        let appClass = CMIssuesReportingApplication.self
        let config = /* heap ComposedApplication @ factories 0x100064a9c, 0x10004e094 */
        NSApplication.shared.setup(with: config)
        NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
    }
}
```

## Composition layer (main binary strings)

Types referenced in `CleanMyMac/` and `ApplicationConfigurator/` sources (compile-time paths in `__cstring`):

- `KnowledgeBaseConfigurator`, `LicensePlanManagerConfigurator`, `LoggerConfigurator`, `PreferencesConfigurator`
- `AppGroupContainerMigrator`, `AppGroupFileContainer`
- `CommonRoutingFactory`, `MainAppRoutingFactory`
- `CleanMyMac.AppDelegate` (`_TtC10CleanMyMac11AppDelegate`)
- Lifecycle hooks: `HealthMonitorAppLifecycleHandler`, `PantherEngineAppLifecycleHandler`

**Inference:** `ComposedApplication` registers configurators that bind **ModuleCore** feature modules, services (`RoutingService`, `ActivationService`, `PantherEngineService`), and UI (`MainAppUI`) before AppKit delegate callbacks run.

## Custom NSApplication

`CMIssuesReportingApplication` subclasses `NSApplication` — likely wraps crash/issue reporting before event dispatch (class referenced at GUI branch in entry disassembly).

## CLI / test / preview modes

Early exits avoid `NSApplicationMain` when:

- Command-line tools path (`isCommandLineRun` + `WorkspaceManagement`)
- Unit test host (`isRunningTests`)
- SwiftUI preview host (`isRunningSwiftUIPreviews`)

Normal MAS GUI launch takes the `CMIssuesReportingApplication` branch only.
