

/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* SpaceLensScanning.SpaceLensScanTasksFactory.createScanTask(directory: Foundation.URL, strategy:
   SpaceLensScanning.SpaceLensCloudScannerStrategy) -> some */

long __thiscall
SpaceLensScanning::SpaceLensScanTasksFactory::_createScanTask
          (SpaceLensScanTasksFactory *this,URL directory,SpaceLensCloudScannerStrategy strategy)

{
  undefined8 *puVar1;
  long lVar2;
  ulong uVar3;
  undefined8 uVar4;
  undefined *puVar5;
  long lVar6;
  undefined8 uVar7;
  undefined8 *puVar8;
  long lVar9;
  undefined8 *puVar10;
  undefined8 uVar11;
  undefined *puVar12;
  long extraout_x8;
  undefined8 uVar13;
  long extraout_x12;
  ulong uVar14;
  long lVar15;
  long lVar16;
  undefined8 uVar17;
  code *local_90;
  undefined *local_88;
  long local_80;
  undefined8 local_78;
  undefined8 uStack_70;
  undefined8 local_68;
  
  lVar2 = 0;
  local_88 = directory.unknown;
  thunk_FUN_0000ec58();
  lVar16 = *(long *)(lVar2 + -8);
  (*(code *)PTR____chkstk_darwin_00014078)(*(undefined8 *)(lVar16 + 0x40));
  lVar15 = (long)&local_90 - (extraout_x8 + 0xfU & 0xfffffffffffffff0);
  (*(code *)PTR____chkstk_darwin_00014078)();
  uVar14 = lVar15 - extraout_x12;
  _swift_getObjectType(*(undefined8 *)(this + 0x10));
  thunk_FUN_0000ec64(uVar14);
  (**(code **)(lVar16 + 0x68))(lVar15,*(undefined4 *)PTR_00014018,lVar2);
  uVar3 = uVar14;
  thunk_FUN_0000ec4c(uVar14,lVar15);
  local_90 = *(code **)(lVar16 + 8);
  local_80 = lVar2;
  (*local_90)(lVar15,lVar2);
  uVar4 = 0;
  FUN_00002998();
  _swift_allocObject();
  _swift_retain(this);
  FUN_00002e7c(uVar4,this);
  _swift_release(this);
  lVar2 = 0;
  FUN_000027f0();
  uVar11 = 0x40;
  _swift_allocObject();
  puVar12 = PTR___swiftEmptySetSingleton_000141b0;
  *(byte *)(lVar2 + 0x10) = (byte)uVar3 & 1;
  *(undefined **)(lVar2 + 0x18) = puVar12;
  *(code **)(lVar2 + 0x20) = FUN_0000d5a8;
  *(undefined8 *)(lVar2 + 0x28) = uVar4;
  uVar7 = uVar4;
  _swift_retain();
  uVar17 = 0;
  uVar13 = 0;
  if ((uVar3 & 1) == 0) {
    _NSHomeDirectory();
    thunk_FUN_0000eca0();
    local_78 = uVar7;
    uStack_70 = uVar11;
    _swift_bridgeObjectRetain(uVar11);
    thunk_FUN_0000ecd0(0xd000000000000012,0x8000000000011bb0);
    _swift_bridgeObjectRelease(uVar11);
    uVar17 = local_78;
    uVar13 = uStack_70;
  }
  *(undefined8 *)(lVar2 + 0x30) = uVar17;
  *(undefined8 *)(lVar2 + 0x38) = uVar13;
  uVar17 = *(undefined8 *)(this + 0x20);
  lVar16 = 0;
  FUN_00001fcc();
  _swift_allocObject();
  *(undefined **)(lVar16 + 0x10) = PTR___swiftEmptyArrayStorage_000141a0;
  *(undefined8 *)(lVar16 + 0x20) = uVar17;
  *(__int8 *)(lVar16 + 0x18) = strategy.value;
  puVar5 = &_OBJC_CLASS___CMVolumeBackupSizeCacher;
  _swift_unknownObjectRetain_n(uVar17,2);
  FUN_0000f220();
  lVar6 = 0;
  FUN_0000a8f8();
  _swift_allocObject();
  uVar7 = 0;
  thunk_FUN_0000ec10();
  _swift_allocObject();
  thunk_FUN_0000ec04();
  *(undefined8 *)(lVar6 + 0x10) = uVar7;
  thunk_FUN_0000eba4(0);
  _swift_allocObject();
  uVar7 = 0;
  thunk_FUN_0000eb74();
  *(undefined8 *)(lVar6 + 0x18) = uVar7;
  local_78 = 0;
  uStack_70 = 0;
  local_68 = 0;
  FUN_00002620(&DAT_00019500,0xfc00);
  _swift_allocObject();
  _swift_retain(uVar7);
  puVar8 = &local_78;
  thunk_FUN_0000ec40();
  *(undefined8 **)(lVar6 + 0x20) = puVar8;
  lVar15 = _TtC17SpaceLensScanning17SpaceLensScanTask::directory;
  lVar9 = 0;
  thunk_FUN_0000eaf0();
  puVar12 = local_88;
  (**(code **)(*(long *)(lVar9 + -8) + 0x10))(lVar6 + lVar15,local_88,lVar9);
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::filter) = lVar2;
  *(undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::fileManager) = uVar17;
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::cloudStorageHelper) = lVar16;
  *(undefined **)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::volumeBackupSizeCacher) =
       puVar5;
  puVar10 = puVar8;
  _swift_retain();
  thunk_FUN_0000eae4();
  puVar5 = puVar12;
  FUN_000036a0();
  _swift_bridgeObjectRelease(puVar12);
  puVar1 = (undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::fileSystemType);
  *puVar1 = puVar10;
  puVar1[1] = puVar5;
  *(undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::maxConcurrentScans) = 4;
  lVar2 = 0;
  FUN_00002938();
  _swift_allocObject();
  _swift_defaultActor_initialize();
  _swift_release(uVar4);
  (*local_90)(uVar14,local_80);
  *(undefined8 *)(lVar2 + 0x70) = 0;
  *(undefined8 *)(lVar2 + 0x78) = 0;
  *(undefined8 *)(lVar2 + 0x80) = 500000000;
  *(undefined8 *)(lVar2 + 0x88) = uVar7;
  *(undefined8 **)(lVar2 + 0x90) = puVar8;
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::progressTracker) = lVar2;
  return lVar6;
}



## raw_result.limitations





/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* SpaceLensScanning.SpaceLensScanTasksFactory.createScanTask(directory: Foundation.URL, strategy:
   SpaceLensScanning.SpaceLensCloudScannerStrategy) -> some */

long __thiscall
SpaceLensScanning::SpaceLensScanTasksFactory::_createScanTask
          (SpaceLensScanTasksFactory *this,URL directory,SpaceLensCloudScannerStrategy strategy)

{
  undefined8 *puVar1;
  long lVar2;
  ulong uVar3;
  undefined8 uVar4;
  undefined *puVar5;
  long lVar6;
  undefined8 uVar7;
  undefined8 *puVar8;
  long lVar9;
  undefined8 *puVar10;
  undefined8 uVar11;
  undefined *puVar12;
  long extraout_x8;
  undefined8 uVar13;
  long extraout_x12;
  ulong uVar14;
  long lVar15;
  long lVar16;
  undefined8 uVar17;
  code *local_90;
  undefined *local_88;
  long local_80;
  undefined8 local_78;
  undefined8 uStack_70;
  undefined8 local_68;
  
  lVar2 = 0;
  local_88 = directory.unknown;
  thunk_FUN_0000ec58();
  lVar16 = *(long *)(lVar2 + -8);
  (*(code *)PTR____chkstk_darwin_00014078)(*(undefined8 *)(lVar16 + 0x40));
  lVar15 = (long)&local_90 - (extraout_x8 + 0xfU & 0xfffffffffffffff0);
  (*(code *)PTR____chkstk_darwin_00014078)();
  uVar14 = lVar15 - extraout_x12;
  _swift_getObjectType(*(undefined8 *)(this + 0x10));
  thunk_FUN_0000ec64(uVar14);
  (**(code **)(lVar16 + 0x68))(lVar15,*(undefined4 *)PTR_00014018,lVar2);
  uVar3 = uVar14;
  thunk_FUN_0000ec4c(uVar14,lVar15);
  local_90 = *(code **)(lVar16 + 8);
  local_80 = lVar2;
  (*local_90)(lVar15,lVar2);
  uVar4 = 0;
  FUN_00002998();
  _swift_allocObject();
  _swift_retain(this);
  FUN_00002e7c(uVar4,this);
  _swift_release(this);
  lVar2 = 0;
  FUN_000027f0();
  uVar11 = 0x40;
  _swift_allocObject();
  puVar12 = PTR___swiftEmptySetSingleton_000141b0;
  *(byte *)(lVar2 + 0x10) = (byte)uVar3 & 1;
  *(undefined **)(lVar2 + 0x18) = puVar12;
  *(code **)(lVar2 + 0x20) = FUN_0000d5a8;
  *(undefined8 *)(lVar2 + 0x28) = uVar4;
  uVar7 = uVar4;
  _swift_retain();
  uVar17 = 0;
  uVar13 = 0;
  if ((uVar3 & 1) == 0) {
    _NSHomeDirectory();
    thunk_FUN_0000eca0();
    local_78 = uVar7;
    uStack_70 = uVar11;
    _swift_bridgeObjectRetain(uVar11);
    thunk_FUN_0000ecd0(0xd000000000000012,0x8000000000011bb0);
    _swift_bridgeObjectRelease(uVar11);
    uVar17 = local_78;
    uVar13 = uStack_70;
  }
  *(undefined8 *)(lVar2 + 0x30) = uVar17;
  *(undefined8 *)(lVar2 + 0x38) = uVar13;
  uVar17 = *(undefined8 *)(this + 0x20);
  lVar16 = 0;
  FUN_00001fcc();
  _swift_allocObject();
  *(undefined **)(lVar16 + 0x10) = PTR___swiftEmptyArrayStorage_000141a0;
  *(undefined8 *)(lVar16 + 0x20) = uVar17;
  *(__int8 *)(lVar16 + 0x18) = strategy.value;
  puVar5 = &_OBJC_CLASS___CMVolumeBackupSizeCacher;
  _swift_unknownObjectRetain_n(uVar17,2);
  FUN_0000f220();
  lVar6 = 0;
  FUN_0000a8f8();
  _swift_allocObject();
  uVar7 = 0;
  thunk_FUN_0000ec10();
  _swift_allocObject();
  thunk_FUN_0000ec04();
  *(undefined8 *)(lVar6 + 0x10) = uVar7;
  thunk_FUN_0000eba4(0);
  _swift_allocObject();
  uVar7 = 0;
  thunk_FUN_0000eb74();
  *(undefined8 *)(lVar6 + 0x18) = uVar7;
  local_78 = 0;
  uStack_70 = 0;
  local_68 = 0;
  FUN_00002620(&DAT_00019500,0xfc00);
  _swift_allocObject();
  _swift_retain(uVar7);
  puVar8 = &local_78;
  thunk_FUN_0000ec40();
  *(undefined8 **)(lVar6 + 0x20) = puVar8;
  lVar15 = _TtC17SpaceLensScanning17SpaceLensScanTask::directory;
  lVar9 = 0;
  thunk_FUN_0000eaf0();
  puVar12 = local_88;
  (**(code **)(*(long *)(lVar9 + -8) + 0x10))(lVar6 + lVar15,local_88,lVar9);
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::filter) = lVar2;
  *(undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::fileManager) = uVar17;
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::cloudStorageHelper) = lVar16;
  *(undefined **)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::volumeBackupSizeCacher) =
       puVar5;
  puVar10 = puVar8;
  _swift_retain();
  thunk_FUN_0000eae4();
  puVar5 = puVar12;
  FUN_000036a0();
  _swift_bridgeObjectRelease(puVar12);
  puVar1 = (undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::fileSystemType);
  *puVar1 = puVar10;
  puVar1[1] = puVar5;
  *(undefined8 *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::maxConcurrentScans) = 4;
  lVar2 = 0;
  FUN_00002938();
  _swift_allocObject();
  _swift_defaultActor_initialize();
  _swift_release(uVar4);
  (*local_90)(uVar14,local_80);
  *(undefined8 *)(lVar2 + 0x70) = 0;
  *(undefined8 *)(lVar2 + 0x78) = 0;
  *(undefined8 *)(lVar2 + 0x80) = 500000000;
  *(undefined8 *)(lVar2 + 0x88) = uVar7;
  *(undefined8 **)(lVar2 + 0x90) = puVar8;
  *(long *)(lVar6 + _TtC17SpaceLensScanning17SpaceLensScanTask::progressTracker) = lVar2;
  return lVar6;
}



