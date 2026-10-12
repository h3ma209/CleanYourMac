# Performance & Assistant

## Performance module

**Frameworks:** `PerformanceModule`, `PerformanceScanning`, `PerformanceService`  
**Assets:** `.car` 53

**Observed types (strings):**

- `BackgroundItemsFlowCoordinator`, `BackgroundItemsScannerConfig`
- `LoginItemsFlowCoordinator`
- `PerformanceModuleAnalyticsTracker`

**Hypothesis:** Scans macOS **login items** and **background item** APIs (macOS 13+ SMAppService / legacy login items), presents remediation in module UI. Distinct from `CleanMyMacLoginItems` framework (agent install) but related operationally.

```swift
LoginItemsFlowCoordinator.run() {
    let items = PerformanceScanning.loginItems()
    // user selects disable/remove
}
```

## Assistant

**Frameworks:** `AssistantModule`, `AssistantUI`, `AssistantDomain`  
**Assets:** `AssistantModule` `.car` **210** (largest module catalog — rich illustration set)

**Observed types:**

- `SmartCareModuleRecommendationVM`
- `JunkCleanupModuleRecommendationVM`
- `ProtectionModuleRecommendationVM`
- Menu string: `menu_item_view_assistant`
- Main binary: `AssistantActivationHandler`, `AssistantActivationHandlerType`

**Hypothesis:** Assistant is a **recommendation layer** over existing modules (suggests junk/protection/smart care actions), not a separate scan engine. UI also surfaces in menu bar app (`AssistantUI.framework` embedded in `CleanMyMac_5_MAS_Menu.app`).

## Toolbox / misc UI modules

- `ToolboxModule` — utilities (screen cleaning mode framework linked)
- `ApplicationsModule` — app uninstall/update adjacent features
- `ScreenCleaningMode.framework` — screen clean overlay (5 catalog assets)

## Unknowns

- LLM/local model boundary for Assistant (check `AssistantDomain` + network service calls — not traced this pass).
- Whether performance scans invoke privileged helper (likely via shared `PrivilegedOperationsPerformerService`).
