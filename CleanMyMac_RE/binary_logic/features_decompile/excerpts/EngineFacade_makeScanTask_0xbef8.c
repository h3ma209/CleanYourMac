/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* static PANEngine.EngineFacade.makeScanTask(configuration: __C.PANScanConfiguration) ->
   __C.PANMalwareScannerTask? */

PANScanConfiguration * __thiscall
PANEngine::EngineFacade::_makeScanTask(EngineFacade *this,PANScanConfiguration *configuration)

{
  undefined8 uVar1;
  undefined1 auStack_c0 [16];
  code *pcStack_b0;
  undefined1 *puStack_a8;
  undefined8 uStack_a0;
  undefined1 auStack_90 [16];
  undefined8 uStack_80;
  undefined8 uStack_68;
  
  if (DAT_00027358 != -1) {
    _swift_once(&DAT_00027358,FUN_0000d950);
  }
  puStack_a8 = auStack_90;
  uStack_80 = DAT_00027478;
  pcStack_b0 = thunk_FUN_000107e0;
  uStack_a0 = DAT_00027478;
  uVar1 = 0;
  thunk_FUN_00013760(0);
  thunk_FUN_0001391c(&uStack_68,FUN_00010c70,auStack_c0,uVar1);
  FUN_0000f4f0(configuration,uStack_68,DAT_00027478);
  return configuration;
}
