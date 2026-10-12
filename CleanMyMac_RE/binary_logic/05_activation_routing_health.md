# Activation, routing & health monitor

## MAS activation

**Frameworks / types:** `ActivationService`, main-binary MAS-specific types:

- `ActivationManagerMAS`, `ActivationServiceMAS`, `SharedActivationMAS`
- `ActivationLockedFunctionalityPerformerFactory` — gates features when inactive
- `EmptyActivationManager` — null object for dev/tests
- `Features_PurchaseFlow.bundle` (66 asset records)

**UI strings:** `MacPawAccountActivation_*` — sign-in / trial / license flows.

**Hypothesis:** StoreKit + MacPaw account network validates entitlement; locked performer wraps module entry points until activation succeeds.

```swift
let activation = ActivationManagerMAS.shared
guard activation.isActive else {
    ActivationLockedFunctionalityPerformer.showPaywall()
    return
}
moduleCoordinator.start()
```

## Routing & deep links

**Frameworks:** `RoutingService`, `RoutingServiceAPI`  
**Main binary:** `CommonRoutingFactory`, `MainAppRoutingFactory`

**Info.plist URL schemes (REA `inspect-plist`):**

| Scheme | Purpose |
|--------|---------|
| `cleanmymac5`, `cleanmymac5-mas` | Activation, module deep links |
| `db-hfn3qmlc7uq9kxd` | Dropbox OAuth callback |

**Hypothesis:** URL/`NSUserActivity` handlers in AppDelegate delegate to routing factory → module coordinators (e.g. open Protection after notification).

## Health monitor

**Components:**

- `HealthMonitoringAPI.framework` (linked from main app)
- Login item: `CleanMyMac_5_MAS_HealthMonitor.app`
- Main binary: `HealthMonitorAppLifecycleHandler`

**Hypothesis:** Separate lightweight process for periodic health checks / notification delivery; communicates via API framework (XPC or distributed notifications — not verified statically).

## Updates

- `MASUpdaterServiceAPI` — Mac App Store update path
- Embedded **Sparkle** (nibs only in UI map) — may serve non-MAS channels or internal builds; present in MAS bundle for shared code paths.

## Analytics nib

`CMAnalyticsWindowController.nib` in `AnalyticsService.framework` — debug/internal analytics window (153 IB objects). Duplicated in login item bundles.

## Unknowns

- Exact XPC service names for health monitor (inspect login item Info.plist + `launchd` plists next pass).
- Activation network endpoints (requires `inspect_web_network_capture` or strings in `NetworkService`).
