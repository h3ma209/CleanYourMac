

/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* UninstallerScanning.LeftoversFetcher.fetchLeftovers() -> [__C.CMApplicationInfo] */

undefined * __thiscall UninstallerScanning::LeftoversFetcher::fetchLeftovers(LeftoversFetcher *this)

{
  ulong uVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  undefined8 uVar4;
  code *pcVar5;
  long in_x0;
  undefined8 uVar6;
  ID IVar7;
  undefined *puVar8;
  undefined8 uVar9;
  undefined8 uVar10;
  undefined8 uVar11;
  undefined *puVar12;
  undefined *puVar13;
  ulong uVar14;
  ID self;
  ulong uVar15;
  undefined8 *puVar16;
  ulong uVar17;
  ulong uVar18;
  
  FUN_0001fc04();
  uVar18 = *(ulong *)(in_x0 + 0x10);
  puVar13 = PTR___swiftEmptyArrayStorage_000542f8;
  if (uVar18 == 0) {
LAB_0001fbd8:
    _swift_bridgeObjectRelease();
    return puVar13;
  }
  uVar14 = 0;
  self = *(ID *)(this + 0x40);
LAB_0001f9d0:
  puVar16 = (undefined8 *)(in_x0 + 0x28 + uVar14 * 0x30);
  uVar17 = uVar14;
  do {
    if (*(ulong *)(in_x0 + 0x10) <= uVar17) {
                    /* WARNING: Does not return */
      pcVar5 = (code *)SoftwareBreakpoint(1,0x1fc04);
      (*pcVar5)();
    }
    uVar9 = puVar16[-1];
    uVar2 = *puVar16;
    uVar10 = puVar16[1];
    uVar3 = puVar16[2];
    uVar11 = puVar16[3];
    uVar4 = puVar16[4];
    _swift_bridgeObjectRetain(uVar2);
    _swift_bridgeObjectRetain(uVar3);
    _swift_bridgeObjectRetain(uVar4);
    uVar6 = uVar9;
    thunk_FUN_0003cbb0(uVar9,uVar2);
    IVar7 = objc_stub::isAppIgnoredWithBundleId_(self,uVar6);
    if ((IVar7 & 1) == 0) {
      puVar8 = &_OBJC_CLASS___CMApplicationInfo;
      _objc_allocWithZone();
      thunk_FUN_0003cbb0(uVar9,uVar2);
      thunk_FUN_0003cbb0(uVar10,uVar3);
      IVar7 = objc_stub::initWithBundleID_appName_((ID)puVar8,uVar9,uVar10);
      if (IVar7 != 0) break;
    }
    uVar17 = uVar17 + 1;
    _swift_bridgeObjectRelease(uVar3);
    _swift_bridgeObjectRelease(uVar4);
    _swift_bridgeObjectRelease(uVar2);
    puVar16 = puVar16 + 6;
    if (uVar18 == uVar17) goto LAB_0001fbd8;
  } while( true );
  thunk_FUN_0003cbb0(uVar11,uVar4);
  objc_stub::setPath_(IVar7,uVar11);
  _swift_bridgeObjectRelease(uVar2);
  _swift_bridgeObjectRelease(uVar3);
  _swift_bridgeObjectRelease(uVar4);
  puVar8 = puVar13;
  _swift_isUniquelyReferencedNonObjC_nonNull_bridgeObject();
  if (((((ulong)puVar8 & 1) == 0) || ((long)puVar13 < 0)) ||
     (puVar8 = puVar13, ((ulong)puVar13 >> 0x3e & 1) != 0)) {
    if ((ulong)puVar13 >> 0x3e == 0) {
      puVar12 = *(undefined **)(((ulong)puVar13 & 0xffffffffffffff8) + 0x10);
    }
    else {
      puVar12 = (undefined *)((ulong)puVar13 & 0xffffffffffffff8);
      if ((undefined *)0x7fffffffffffffff < puVar13) {
        puVar12 = puVar13;
      }
      thunk_FUN_0003ceec(puVar12);
    }
    puVar8 = (undefined *)0x0;
    FUN_00022504(0,puVar12 + 1,1,puVar13);
  }
  uVar15 = (ulong)puVar8 & 0xffffffffffffff8;
  uVar1 = *(ulong *)(uVar15 + 0x10);
  puVar13 = puVar8;
  if (*(ulong *)(uVar15 + 0x18) >> 1 <= uVar1) {
    puVar13 = (undefined *)(ulong)(1 < *(ulong *)(uVar15 + 0x18));
    FUN_00022504(puVar13,uVar1 + 1,1,puVar8);
    uVar15 = (ulong)puVar13 & 0xffffffffffffff8;
  }
  uVar14 = uVar17 + 1;
  *(ulong *)(uVar15 + 0x10) = uVar1 + 1;
  *(ID *)(uVar15 + uVar1 * 8 + 0x20) = IVar7;
  if (uVar18 - 1 == uVar17) goto LAB_0001fbd8;
  goto LAB_0001f9d0;
}



## raw_result.limitations





/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* UninstallerScanning.LeftoversFetcher.fetchLeftovers() -> [__C.CMApplicationInfo] */

undefined * __thiscall UninstallerScanning::LeftoversFetcher::fetchLeftovers(LeftoversFetcher *this)

{
  ulong uVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  undefined8 uVar4;
  code *pcVar5;
  long in_x0;
  undefined8 uVar6;
  ID IVar7;
  undefined *puVar8;
  undefined8 uVar9;
  undefined8 uVar10;
  undefined8 uVar11;
  undefined *puVar12;
  undefined *puVar13;
  ulong uVar14;
  ID self;
  ulong uVar15;
  undefined8 *puVar16;
  ulong uVar17;
  ulong uVar18;
  
  FUN_0001fc04();
  uVar18 = *(ulong *)(in_x0 + 0x10);
  puVar13 = PTR___swiftEmptyArrayStorage_000542f8;
  if (uVar18 == 0) {
LAB_0001fbd8:
    _swift_bridgeObjectRelease();
    return puVar13;
  }
  uVar14 = 0;
  self = *(ID *)(this + 0x40);
LAB_0001f9d0:
  puVar16 = (undefined8 *)(in_x0 + 0x28 + uVar14 * 0x30);
  uVar17 = uVar14;
  do {
    if (*(ulong *)(in_x0 + 0x10) <= uVar17) {
                    /* WARNING: Does not return */
      pcVar5 = (code *)SoftwareBreakpoint(1,0x1fc04);
      (*pcVar5)();
    }
    uVar9 = puVar16[-1];
    uVar2 = *puVar16;
    uVar10 = puVar16[1];
    uVar3 = puVar16[2];
    uVar11 = puVar16[3];
    uVar4 = puVar16[4];
    _swift_bridgeObjectRetain(uVar2);
    _swift_bridgeObjectRetain(uVar3);
    _swift_bridgeObjectRetain(uVar4);
    uVar6 = uVar9;
    thunk_FUN_0003cbb0(uVar9,uVar2);
    IVar7 = objc_stub::isAppIgnoredWithBundleId_(self,uVar6);
    if ((IVar7 & 1) == 0) {
      puVar8 = &_OBJC_CLASS___CMApplicationInfo;
      _objc_allocWithZone();
      thunk_FUN_0003cbb0(uVar9,uVar2);
      thunk_FUN_0003cbb0(uVar10,uVar3);
      IVar7 = objc_stub::initWithBundleID_appName_((ID)puVar8,uVar9,uVar10);
      if (IVar7 != 0) break;
    }
    uVar17 = uVar17 + 1;
    _swift_bridgeObjectRelease(uVar3);
    _swift_bridgeObjectRelease(uVar4);
    _swift_bridgeObjectRelease(uVar2);
    puVar16 = puVar16 + 6;
    if (uVar18 == uVar17) goto LAB_0001fbd8;
  } while( true );
  thunk_FUN_0003cbb0(uVar11,uVar4);
  objc_stub::setPath_(IVar7,uVar11);
  _swift_bridgeObjectRelease(uVar2);
  _swift_bridgeObjectRelease(uVar3);
  _swift_bridgeObjectRelease(uVar4);
  puVar8 = puVar13;
  _swift_isUniquelyReferencedNonObjC_nonNull_bridgeObject();
  if (((((ulong)puVar8 & 1) == 0) || ((long)puVar13 < 0)) ||
     (puVar8 = puVar13, ((ulong)puVar13 >> 0x3e & 1) != 0)) {
    if ((ulong)puVar13 >> 0x3e == 0) {
      puVar12 = *(undefined **)(((ulong)puVar13 & 0xffffffffffffff8) + 0x10);
    }
    else {
      puVar12 = (undefined *)((ulong)puVar13 & 0xffffffffffffff8);
      if ((undefined *)0x7fffffffffffffff < puVar13) {
        puVar12 = puVar13;
      }
      thunk_FUN_0003ceec(puVar12);
    }
    puVar8 = (undefined *)0x0;
    FUN_00022504(0,puVar12 + 1,1,puVar13);
  }
  uVar15 = (ulong)puVar8 & 0xffffffffffffff8;
  uVar1 = *(ulong *)(uVar15 + 0x10);
  puVar13 = puVar8;
  if (*(ulong *)(uVar15 + 0x18) >> 1 <= uVar1) {
    puVar13 = (undefined *)(ulong)(1 < *(ulong *)(uVar15 + 0x18));
    FUN_00022504(puVar13,uVar1 + 1,1,puVar8);
    uVar15 = (ulong)puVar13 & 0xffffffffffffff8;
  }
  uVar14 = uVar17 + 1;
  *(ulong *)(uVar15 + 0x10) = uVar1 + 1;
  *(ID *)(uVar15 + uVar1 * 8 + 0x20) = IVar7;
  if (uVar18 - 1 == uVar17) goto LAB_0001fbd8;
  goto LAB_0001f9d0;
}



