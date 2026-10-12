# Feature 01: CarouselFlow (`Features_CarouselFlow.bundle`)

## Bundle identity

| Field | Value |
|-------|-------|
| Path | `/Applications/CleanMyMac_5_MAS.app/Contents/Resources/Features_CarouselFlow.bundle` |
| CFBundleName | `Features_CarouselFlow` |
| CFBundleIdentifier | `com.macpaw.CleanMyMac.CarouselFlow.generated.resources` |
| CFBundlePackageType | `BNDL` (resource bundle) |
| Version | 1.0 (1) |

## Inventory (no executable code in bundle)

- **MacOS/** — absent (no `Contents/MacOS` dylib/exe)
- **Resources/Assets.car** — ~6.9 MB asset catalog
- **Resources/*.lproj/Localizable.strings** — 13 locales (en, de, es, fr, it, ja, ko, nl, pl, pt-BR, uk, zh-Hans)
- **Info.plist** — Tuist-generated resource bundle metadata only

**Conclusion:** Resource-only feature module. UI copy + images live in bundle; Swift UI/logic is linked into main `CleanMyMac_5_MAS` binary.

## Onboarding carousel content (en strings sample)

Localized keys follow `Carousel_Page_<Feature>_{Title,Subtitle,Description}_<Audience>`:

- **Intro** — “Take a Tour”, new-user vs upgrade copy
- **Cleanup** — deeper scan / more junk messaging
- **Applications** — Uninstaller + Updater
- **Assistant** — health score recommendations
- **CloudCleanup** — cloud storage declutter
- **Duplicates** — variants for New_Users, Subscriber, Upgrade, OneTime, MAS

Audience suffixes: `New_Users`, `Upgrade`, `Subscriber`, `Onetime`, `Upgrade_OneTime_MAS`.

## Main binary integration (arm64 slice)

Runtime bundle load string:

- `Features_CarouselFlow.bundle`
- Error: `unable to find bundle named Features_CarouselFlow`

Tuist-generated accessors (embedded in main binary):

- `CarouselFlow/TuistAssets+CarouselFlow.swift`
- `CarouselFlow/TuistBundle+CarouselFlow.swift`

### Swift types (from embedded symbols / debug paths)

| Symbol / path | Role (inferred) |
|---------------|-----------------|
| `CarouselFlow.CarouselBuilder` | Builds carousel flow |
| `CarouselFlow.CarouselFlowPresenter` | Presenter / orchestration |
| `CarouselFlow.CarouselPagesProvider` | Page sequence |
| `CarouselFlow.CarouselViewController` | Container VC |
| `CarouselFlow.CarouselPageViewController` | Single page |
| `CarouselFlow.CarouselView` / `CarouselPageView` | Views |
| `CarouselFlow.CarouselFlowResources` | Bundle resource accessor |
| `CleanMyMac.CarouselFlowPresenterWrapper` | App-shell adapter |
| `CarouselFlowPresenterType` (CleanMyMac protocol) | DI boundary in app target |

Protocols in module `CarouselFlow`:

- `CarouselPageDelegate`
- `DismissNotifiableType`
- `CarouselAnalyticsTrackerType`
- `OnboardingEligibilityCheckerType`

Source paths (CI build tree, not shipped):

- `Features/CarouselFlow/Common/Sources/Carousel/CarouselUI/*.swift`
- `CarouselFlow/CarouselBuilder.swift`, `CarouselFlowPresenter.swift`

App wiring:

- Property `carouselFlowPresenter` on main app type (strings show duplicate symbol refs)

## Initialization hooks

**Ghidra decompile:** see [features_decompile/01_Features_CarouselFlow.md](./features_decompile/01_Features_CarouselFlow.md) (`CarouselFlowResources::bundle`, page VC `viewDidLoad` / `handleNextAction`).

## Initialization hooks (static summary)

Expected flow (from type graph, not yet decompiled):

1. App constructs `CarouselFlowPresenterWrapper` conforming to `CarouselFlowPresenterType`.
2. `CarouselBuilder` assembles VC graph using `CarouselPagesProvider` + localized strings from `Features_CarouselFlow.bundle`.
3. `OnboardingEligibilityCheckerType` gates whether carousel runs (new vs upgrade vs subscriber).
4. `CarouselAnalyticsTrackerType` emits tour events on page changes / dismiss.

**Status:** Ghidra decompile captured in `features_decompile/01_Features_CarouselFlow.md`. Early attempts logged in `feature_01_CarouselFlow_decompile.log`.

## Next probes

1. Decompile `_TtC12CarouselFlow21CarouselFlowPresenter` and `CleanMyMac.CarouselFlowPresenterWrapper`
2. `trace` / xrefs from string `Features_CarouselFlow.bundle` @ `0x1004050e0` (Ghidra xrefs may be sparse on Swift metadata)
3. Second Features bundle: `Features_IntroFlow.bundle` or `Features_PurchaseFlow.bundle`

## REA artifacts used

- `plutil` on bundle Info.plist + en Localizable.strings
- `rea inspect-macho` on arm64 thin (uuid `9E01313F-D270-368D-87FD-E30D0B9D0932`, min OS 11.0)
- `strings` on `/tmp/CleanMyMac_5_MAS_arm64`
