# Real-Time Protection UI map

Started from `Features_RealTimeProtection.bundle`. English strings are a single status modal (who owns the monitor: CleanMyMac vs Moonlock). No scan, walk, or quarantine copy. Scan loop lives in frameworks named below.

## Bundle

| Field | Value |
| --- | --- |
| Path | `Contents/Resources/Features_RealTimeProtection.bundle` |
| CFBundleName | Features_RealTimeProtection |
| CFBundleIdentifier | `com.macpaw.CleanMyMac.RealTimeProtection.generated.resources` |
| Package | BNDL (resources only, no executable) |
| Version | 1.0 (1) |
| Min OS | 11.0 |
| SDK | macosx26.5 |

Contents: `Info.plist`, `Assets.car`, `Localizable.strings` in 13 lproj folders. No nibs, storyboards, or code.

## Strings (en)

| Key | Role |
| --- | --- |
| `RTP_ByCMM_Title` / `RTP_ByCMM_Description` | Monitor resumed because Moonlock is inactive |
| `RTP_ByMoonlock_Title` / `RTP_ByMoonlock_Description` | Moonlock owns protection; CMM monitor off |
| `RTP_GotIt_Button_title` | Dismiss |

Sidecar: `RTP_strings.json`.

## Assets

`Assets.car` via `assetutil --info` (REA catalog tool cannot open a resource bundle). 11 records. Four images, each 1x and 2x:

- `ModalMoonlockIcon`
- `ModalMoonlockEnabled`
- `ModalMoonlockDisabled`
- `ModalMoonlockCMMIcon`

Sidecar: `RTP_assets.json`.

## Types (main executable strings / Swift metadata)

Resource bundle has no types. Main binary `CleanMyMac_5_MAS` (arm64 slice `/tmp/CleanMyMac_5_MAS_arm64`) names the modal:

| Kind | Name |
| --- | --- |
| View | `MoonlockRTPStateModalView` |
| Controller | `MoonlockRTPStateModalController` (`RealTimeProtection.MoonlockRTPStateModalController`) |
| Presenter | `MoonlockRTPStateModalPresenter` / `MoonlockRTPStateModalPresenterType` |
| State | `RealTimeProtectionState` |
| Analytics | `MoonlockRTPStateModalAnalyticsTracker` |

Source paths baked into the binary:

- `Features/RealTimeProtection/Common/Sources/MoonlockRTPStateModalView/MoonlockRTPStateModalView.swift`
- `RealTimeProtection/MoonlockRTPStateModalPresenter.swift`

Log lines (behavior of the presenter, not a scanner): skip modal if already enqueued, presenter missing, protection module missing, intro not completed, status unchanged, or first launch. Show path logs `Showing Moonlock RTP modal with state:`.

Accessibility id: `MoonlockRTPStateModal-GotItButton`.

## Linked Protection UI (not this bundle)

On-demand scan chrome is `ProtectionModule.framework`, separate from the RTP status modal:

- Coordinators: `ProtectionModuleCoordinator`, `ProtectionTilesScreenCoordinator`, `MalwareFinderFlowCoordinator`, `ReviewMalwareFlowCoordinator`
- Controllers: `ProtectionIntroViewController`, `ProtectionScanResultsViewController`, `MalwareCleanupCompleteViewController`, `MoonlockDescriptionViewController`, `MoonlockCrossPromoScanResultsViewController`
- Presenters / view models: `ProtectionScanProgressViewModel`, `ProtectionScanResultsViewModel`, `MalwareScanningInProgressTileViewModel`, `MalwareNotFoundTileViewModel`, `MalwareMoonlockTileViewModel`, `MalwareTileViewModel`

Intro and cross-promo types exist in that module. They are not the file-walk engine.
