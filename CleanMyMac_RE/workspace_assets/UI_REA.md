# CleanMyMac 5 MAS — UI reverse (REA)

**Target:** `/Applications/CleanMyMac_5_MAS.app`  
**REA CLI:** 6.4.0. MCP `user-rea` was unavailable (`failed during live tool discovery`); these calls used the CLI.

## Evidence

| Call | Evidence | What it covers |
|------|----------|----------------|
| `rea decode-interface-builder` | `ev_1bc1a11bb3045695e362316bb789930f265c29d9fabd9452f9bb507a96954afd` | 11 compiled nibs, 1488 nodes, 623 edges, `truncated: false` |
| `rea demangle-swift` on `MainAppUI` | `ev_c70f1fe7d16aed56433c07bb7a2d7fe65f7eaf734f4586fa0c3209f3a80893d8` | Window shell, sidebar model, module slot |
| Prior `inspect-asset-catalog` | `workspace_assets/main_asset_catalog_p0.json` | Catalog paths and record counts |

Full nib graph: `workspace_assets/ui_rea/interface_builder_full.json`.

`nm -gU` plus `swift-demangle` supplied the wider ViewController list below. That pass is shell evidence, not an REA Evidence record. Exported symbols only. Static analysis did not run the app.

## Interface Builder

No `.storyboardc` in the bundle. Feature screens are not Interface Builder documents.

Decoded nibs are Sparkle and MacPaw analytics, copied into the main app and both login-item apps:

| Archive | Objects | Connections | Custom class |
|---------|---------|-------------|--------------|
| `AnalyticsService` `CMAnalyticsWindowController.nib` | 153 | 22 | `CMAnalyticsWindowController` |
| Sparkle `SUUpdateAlert.nib` | 95 | 18 | `SUUpdateAlert` |
| Sparkle `SUUpdatePermissionPrompt` (two keyed archives) | 150 each | 24 each | `SUUpdatePermissionPrompt` |
| Sparkle `SUStatus.nib` | 44 | 11 | `SUStatusController` |

Observed actions: `installUpdate:`, `remindMeLater:`, `skipThisVersion:`, `toggleMoreInfo:`, `finishPrompt:`. Those selectors are archive observations. This pass did not prove a matching implementation in the product modules.

Graph kinds: 506 constraints, 180 views, 196 controls, 18 actions, 11 scenes. Hierarchy marked complete for every document. Constraints and pixel layout are not reconstructed.

## Window shell

REA demangle of the allocating initializer:

`MainAppUI.MainWindowContainerViewController.init(navigationBarViewController:sidebarViewController:moduleViewController:canvasViewController:)`

- Sidebar argument is `NSViewController & MainAppUI.IsHoverableViewControllerType`.
- `SidebarViewModel.init(listItems:footerItems:)` takes two `[SidebarItemType]` arrays.
- `SidebarItem.init(title:activeModuleImage:inactiveModuleImage:backgroundColors:)`.
- `ModuleViewController.init(viewModel:controllers:overlayControllers:)` holds the active module plus overlay controllers.
- `NavigationBarViewController.init(viewModel:imageLoader:)` takes `NotificationsUI.ImageLoader`.

Compile paths left in `MainAppUI` strings place this tree under `cleanmymac5/UI/MainAppUI/Common/Sources/Container/`, including `Sidebar/SidebarView.swift`.

Other `MainAppUI` controllers from exported symbols: `MainWindowController`, `CanvasViewController`, `AboutViewController`, `AcknowledgmentsViewController`.

## Shared scan screens

`ModuleCore` exports the scan-module screens used by more than one feature. Names are observations. Call order is not proven here.

`IntroViewController`, `IntroScreenCoordinator`, `ScanningScreenCoordinator`, `ScanProgressViewController`, `ScanResultsScreenCoordinator`, `ReviewBeforeCleaningScreenCoordinator`, `CleaningScreenCoordinator`, `CleanProgressViewController`, `CleanResultsScreenCoordinator`, `CleanLogViewController`, `ViewLogScreenCoordinator`, `TilesViewController`, `TileScanResultsViewController`, `VolumeSelectionViewController`, `CategoriesViewController`, `GroupsViewController`, `OutlineViewController`, `DetailsViewController`, `NoDataViewController`.

`ModuleUI` adds `CleanupCompleteViewController`, `IgnoreMessageViewController`, and modal present/presented controllers.

`NavigationUI.NavigationController` pushes and pops `NSViewController`s (`pushViewController`, `popViewController`, `popToRootViewController`). `TransitionViewController.selectIndex` switches among a controller array.

## Sidebar labels

English `Localizable.strings` keys, not a proven binding to `SidebarItem`:

| Key | Label |
|-----|-------|
| `menu_item_view_smartCare` | Smart Care |
| `menu_item_view_junkCleanup` | Cleanup |
| `menu_item_view_protection` | Protection |
| `menu_item_view_performance` | Performance |
| `menu_item_view_applications` | Applications |
| `menu_item_view_myClutter` | My Clutter |
| `menu_item_view_spaceLens` | Space Lens |
| `menu_item_view_cloudCleanup` | Cloud Cleanup |
| `menu_item_view_assistant` | Assistant |

## Module UI types

Exported controllers and coordinators. Asset record counts are from the prior catalog inventory.

| Module | UI types | Assets.car records |
|--------|----------|--------------------|
| Smart Care | `SmartCareModuleCoordinator`, `SmartCareScreenCoordinator` | 26 |
| Cleanup | `JunkCleanupModuleCoordinator` | 104 |
| Protection | `MoonlockDescriptionViewController` | 116 |
| Performance | no exported `*ViewController` in `PerformanceModule` | 53 |
| Applications | no exported `*ViewController` in `ApplicationsModule` | 49 |
| Space Lens | coordinator in `SpaceLensModule`; screens in `SpaceLensUI`: `SpaceLensViewController`, `SpaceLensListViewController`, `SpaceLensReviewItemsViewController`, `SpaceLensCleanupCompleteViewController` | module 21, UI 36 |
| Cloud | `CloudStorageUI`: account list, scan option, summary, footer, bottom info, `CloudModalViewController` | module 95, UI 9 |
| Assistant | `AssistantUI.AssistantRootViewController`, recommendation list, expanded recommendation, health-score infographic; module announcements | module 210, UI 9 |
| Email | `EmailCleanerCardViewController` | 60 |
| Organize | `MediaGalleryViewController` plus preview, metadata, select-control | 47 |
| Toolbox | uses `ModuleCore.ScanningModuleCoordinator` | 20 |

Also present: `DataManagerUI` container/footer, `StatisticsDashboard` (local storage, clouds cleaned, malware scanned, time saved), `SettingsUI.SettingsWindowController`, `NotificationsUI` side panel and header, `TipsServiceUI.AITipsViewController`, `TerminatorUI.TerminatorWindowController`.

`SpaceLensModule` exports screen-id tags `reviewSelection` and `scanResultsReview`. String values of those tags were not recovered.

## Screen ids

`ModuleCore` arm64 getters build these Swift strings with `mov`/`movk` immediates in `/tmp/ModuleCore_arm64`. Shell disassembly, not an REA Evidence record.

| Getter | Address | String |
|--------|---------|--------|
| `intro` | `0x1f5e0` | `intro` |
| `scanning` | `0x1f61c` | `scanning` |
| `tiles` | `0x1f6c8` | `tiles` |
| `cleaning` | `0x1f8c8` | `cleaning` |
| `cleanResults` | `0x1f90c` | `cleanResults` |
| `log` | `0x1fa00` | `log` |
| `mainDataManager` | `0x1f7c0` | `mainDataManager` |

`spaceLensManager` and `tilesDataManager` exist as getters and as cstrings at file offsets `0x137240` and `0x137260`. Their getter bodies were not opened.

`ScanningModuleMainFlowCoordinator` exports these transitions. Names are the API. This pass did not prove which method runs first.

- `coordinatorDidRequestScanning`
- `coordinatorDidFinishedScanning`
- `coordinatorDidRequestViewResults`
- `coordinatorDidRequestReview` and `coordinatorDidRequestReviewOfTile`
- `coordinatorDidRequestCleaning`
- `coordinatorDidRequestCleaningResults`
- `coordinatorDidRequestViewLog`
- `coordinatorDidRequestBack`, `coordinatorDidRequestRescan`, `coordinatorDidRequestReset`, `coordinatorDidRequestRestart`

`ModuleScreensNavigator.push` takes a screen coordinator and a `NavigationStep` (`main` or `module`, each carrying a screen id).

## Sidebar keys

Main executable arm64 slice `/tmp/CleanMyMac_5_MAS_arm64`. One function at `0x10002aaa8` builds eight `Localizable` keys (`"Localizable"` is an immediate at `0x10002ab58`) and switches on a byte tag:

| Tag | Key |
|-----|-----|
| 0 | `menu_item_view_smartCare` |
| 1 | `menu_item_view_junkCleanup` |
| 2 | `menu_item_view_applications` |
| 3 | `menu_item_view_performance` |
| 4 | `menu_item_view_protection` |
| 5 | `menu_item_view_myClutter` |
| 6 | `menu_item_view_spaceLens` |
| 7 | `menu_item_view_cloudCleanup` |

`menu_item_view_assistant` is referenced near `0x1000ac2b0`, outside this switch. The byte array that feeds the switch was not recovered, so on-screen row order is still unknown.

## Performance and Applications

Both link `PerformanceModuleBuilder.build()` and `ApplicationsModuleBuilder.build()`, each returning `ModuleCore.Module<…Config>`. Neither framework exports its own `ViewController`. Their screens are the shared `ModuleCore` flow above.

## Unknown

- On-screen order of the eight sidebar rows. Tags are known. The tag byte array is not.
- Whether Assistant is a footer item. Its key is outside the 0–7 switch.
- Which `coordinatorDidRequest*` runs first at runtime.
- Pixel layout, colors, and sidebar width.
- Purchase, intro, carousel, and settings view code. Their asset catalogs were already inventoried.
