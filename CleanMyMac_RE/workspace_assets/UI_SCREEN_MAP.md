# UI & Asset Screen Map (Track 1)

## Architecture summary

CleanMyMac 5 MAS ships **Swift/AppKit module UI in dylibs**, not storyboards. Compiled **Assets.car** per module carries icons/illustrations. Onboarding/monetization/permissions use **`Features_*.bundle`** resource bundles.

```
MainWindowContainerViewController (MainAppUI)
├── navigationBarViewController
├── sidebarViewController (NavigationUI — hoverable)
├── moduleViewController  ← active feature module
└── canvasViewController  ← live background / module canvas
```

Demangled constructor (arm64):  
`MainAppUI.MainWindowContainerViewController.init(navigationBarViewController:sidebarViewController:moduleViewController:canvasViewController:)`

---

## Menu → module mapping

| Menu key (`Localizable.strings`) | Inferred module | Asset catalog (records) |
|----------------------------------|-----------------|-------------------------|
| `menu_item_view_junkCleanup` | Junk / System Cleanup | `JunkCleanupModule` (104) |
| `menu_item_view_protection` | Malware & privacy | `ProtectionModule` (116) |
| `menu_item_view_performance` | Login/background items | `PerformanceModule` (53) |
| `menu_item_view_smartCare` | Aggregated care scan | `SmartCareModule` (26) |
| `menu_item_view_spaceLens` | Disk map | `SpaceLensModule` (21), `SpaceLensUI` (36) |
| `menu_item_view_cloudCleanup` | iCloud/Dropbox/etc. | `CloudStorageModule` (95) |
| `menu_item_view_assistant` | AI assistant surface | `AssistantModule` (210), `AssistantUI` (9) |

Additional modules with significant UI assets: `ApplicationsModule` (49), `OrganizeModule` (47), `EmailCleanupModule` (60), `ToolboxModule` (20), `StatisticsDashboardUI` (28).

---

## Features_* bundles → flows

| Bundle | Likely screen / flow |
|--------|----------------------|
| `Features_IntroFlow` | First launch intro |
| `Features_CarouselFlow` | Feature carousel / education |
| `Features_PurchaseFlow` | MAS purchase / trial |
| `Features_Settings` | Settings sub-flows |
| `Features_RealTimeProtection` | RTP enablement UX |
| `Features_FDAResolutionUIKit` | Full Disk Access prompts |
| `Features_EmailSubscription` | Marketing email opt-in |
| `Features_KeepMenuOpen` | Menu bar agent preference |

---

## Login items (satellite UI)

| App | UI notes |
|-----|----------|
| `CleanMyMac_5_MAS_Menu.app` | Menu bar; `Assets.car` 293 records; embeds AssistantUI/SharedUI |
| `CleanMyMac_5_MAS_HealthMonitor.app` | Health notifications; SharedResources catalog (83) |

---

## Interface Builder (decoded)

| NIB | Class / purpose |
|-----|-----------------|
| `CMAnalyticsWindowController.nib` | MacPaw analytics debug/window (`AnalyticsService`) |
| Sparkle `SUUpdate*.nib`, `SUStatus.nib` | Third-party update UI |

No `.storyboardc` in bundle. Feature screens are **not** IB-document based.

---

## Localization

735 `.strings` + 168 `.stringsdict` under `Contents/` (module frameworks + bundles). English menu/module keys copied to `en_Localizable.strings` in this workspace.

---

## Asset extraction status

| Action | Status |
|--------|--------|
| SHA-256 manifest for `.car`/`.nib` | Done → `manifest_hashes.txt` |
| REA catalog metadata (paths, counts, digests) | Done → `main_asset_catalog_p0.json` |
| Rendition PNG/PDF extraction | **Not supported** by REA inspect-asset-catalog |
| Copy raw `.car` files | Omitted (large); paths listed in manifest JSON |
