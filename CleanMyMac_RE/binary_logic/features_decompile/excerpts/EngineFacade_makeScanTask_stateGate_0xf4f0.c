/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */

ID FUN_0000f4f0(undefined8 param_1,undefined8 param_2,long param_3)

{
  code *pcVar1;
  undefined8 uVar2;
  ulong uVar3;
  ID IVar4;
  undefined1 auStack_a0 [16];
  code *local_90;
  undefined1 *puStack_88;
  long local_80;
  undefined1 auStack_70 [16];
  long local_60;
  undefined8 uStack_58;
  ID local_48;
  
  IVar4 = *(ID *)(param_3 + PANEngineFacade::syncingLock);
  _objc_msgSend(IVar4,"lock");
  uVar3 = *(ulong *)(param_3 + PANEngineFacade::_engineState);
  _objc_msgSend(IVar4,"unlock");
  if (uVar3 < 2) {
    puStack_88 = auStack_70;
    local_90 = FUN_000108fc;
    uVar2 = 0;
    local_80 = param_3;
    local_60 = param_3;
    uStack_58 = param_2;
    FUN_00010904(0,&DAT_000276d0,&PTR_PANMalwareScannerTaskBuilder_00026e28);
    thunk_FUN_0001391c(&local_48,FUN_00010c70,auStack_a0,uVar2);
    IVar4 = _objc_msgSend(local_48,"makeScanTaskWithConfiguration:",param_1);
  }
  else {
    IVar4 = 0;
    if (uVar3 != 2) {
      local_48 = uVar3;
      thunk_FUN_00013994(&__type_metadata_for_PANEngine_EngineFacade_State,&local_48,
                         &__type_metadata_for_PANEngine_EngineFacade_State,PTR_000200d8);
                    /* WARNING: Does not return */
      pcVar1 = (code *)SoftwareBreakpoint(1,0xf648);
      (*pcVar1)();
    }
  }
  return IVar4;
}
