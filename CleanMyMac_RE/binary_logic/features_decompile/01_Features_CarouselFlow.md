# Features_CarouselFlow — Ghidra decompile (Track 2)

## Bundle inventory

**Path:** `/Applications/CleanMyMac_5_MAS.app/Contents/Resources/Features_CarouselFlow.bundle`

| Component | Notes |
|-----------|--------|
| `Contents/Info.plist` | `CFBundleIdentifier`: `com.macpaw.CleanMyMac.CarouselFlow.generated.resources`; `CFBundleName`: `Features_CarouselFlow`; version 1.0 |
| `Contents/Resources/Assets.car` | Asset catalog |
| `Contents/Resources/*.lproj/Localizable.strings` | UI copy keys (`Carousel_Page_*`, etc.) |
| Mach-O executable | **None** — logic lives in main app binary (`CarouselFlow` Swift module) |

## Provider log (CLI)

```
provider: ghidra 12.1.4
target: /tmp/CleanMyMac_5_MAS_arm64 (arm64 thin)
GHIDRA_INSTALL_DIR=/opt/homebrew/opt/ghidra/libexec
JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home
operations: rea analyze, rea search (strings/procedures), rea decompile
```

**MCP note:** After `~/.cursor/mcp.json` env + Cursor reload, `open_binary` with `provider_id: ghidra` succeeds. Heavy decompiles may need CLI `rea decompile` (MCP can timeout).

## Entry points (procedure search `Carousel`)

| Address | Symbol |
|---------|--------|
| `0x1000fc30c` | `_TtC12CarouselFlow21CarouselFlowResources::bundle` |
| `0x1000fc34c` | `_TtC12CarouselFlow21CarouselFlowResources::init` |
| `0x1000fe374` | `_TtC12CarouselFlow26CarouselPageViewController::viewDidLoad` |
| `0x1000fe8b0` | `…CarouselPageViewController_CarouselFlow_::handleNextAction` |

String xref: `Features_CarouselFlow.bundle`, `unable to find bundle named Features_CarouselFlow`, `CarouselFlow/TuistBundle+CarouselFlow.swift`.

---

## Initial decompile log (Ghidra pseudocode excerpts)

### `_TtC12CarouselFlow21CarouselFlowResources::bundle` @ `0x1000fc30c`

```
/* WARNING: Function: _objc_retainAutoreleaseReturnValue replaced with injection: _objc_retain_fixup
    */
/* Function Stack Size: 0x10 bytes */

ID _TtC12CarouselFlow21CarouselFlowResources::bundle(ID param_1,SEL param_2)

{
  if (DAT_1005655f0 != -1) {
    _swift_once(&DAT_1005655f0,FUN_1000fba0c);
  }
  return DAT_1005adf40;
}
```

### `_TtC12CarouselFlow21CarouselFlowResources::init` @ `0x1000fc34c`

```
/* Function Stack Size: 0x10 bytes */

ID _TtC12CarouselFlow21CarouselFlowResources::init(ID param_1,SEL param_2)

{
  class_t *pcVar1;
  ID IVar2;
  objc_super local_30;
  
  pcVar1 = (class_t *)param_1;
  _swift_getObjectType();
  local_30.receiver = param_1;
  local_30.super_class = pcVar1;
  IVar2 = _objc_msgSendSuper2(&local_30,"init");
  return IVar2;
}
```

### `_TtC12CarouselFlow26CarouselPageViewController::viewDidLoad` @ `0x1000fe374`

```
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* Function Stack Size: 0x10 bytes */

void _TtC12CarouselFlow26CarouselPageViewController::viewDidLoad(ID param_1,SEL param_2)

{
  FUN_1000fdde8(param_1);
  return;
}
```

### `handleNextAction` @ `0x1000fe8b0`

```
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* Function Stack Size: 0x10 bytes */

void objc::Categories::_TtC12CarouselFlow26CarouselPageViewController_CarouselFlow_::
     handleNextAction(ID param_1,SEL param_2)

{
  undefined8 *puVar1;
  long lVar2;
  long lVar3;
  undefined8 uVar4;
  code *pcVar5;
  
  lVar3 = param_1 + _TtC12CarouselFlow26CarouselPageViewController::viewModel;
  lVar2 = 0;
  FUN_1000ff434();
  puVar1 = (undefined8 *)(lVar3 + *(int *)(lVar2 + 0x30));
  pcVar5 = (code *)*puVar1;
  if (pcVar5 != (code *)0x0) {
    uVar4 = puVar1[1];
    FUN_1000122ec(pcVar5,uVar4);
    (*pcVar5)();
    FUN_1000122fc(pcVar5,uVar4);
    return;
  }
  lVar3 = param_1 + _TtC12CarouselFlow26CarouselPageViewController::delegate;
  _swift_unknownObjectWeakLoadStrong();
  if (lVar3 != 0) {
    FUN_100100060(param_1);
    _swift_unknownObjectRelease(lVar3);
    return;
  }
  return;
}
```

## Interpretation (brief)

- **Bundle load:** `CarouselFlowResources::bundle` uses `swift_once` + cached `NSBundle` (`DAT_1005adf40`) — ties UI assets/strings to `Features_CarouselFlow.bundle`.
- **UI flow:** Page VC `viewDidLoad` delegates setup to `FUN_1000fdde8`; Next button invokes view-model closure or weak `delegate` path.
