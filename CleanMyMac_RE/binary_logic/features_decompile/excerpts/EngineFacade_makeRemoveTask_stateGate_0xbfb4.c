/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */

ID FUN_0000bfb4(undefined8 param_1,undefined8 param_2)

{
  code *pcVar1;
  undefined8 uVar2;
  long unaff_x20;
  ID IVar3;
  ulong uVar4;
  undefined1 auStack_b0 [16];
  code *local_a0;
  undefined1 *puStack_98;
  undefined1 auStack_80 [16];
  ID local_58;
  
  IVar3 = *(ID *)(unaff_x20 + PANEngineFacade::syncingLock);
  _objc_msgSend(IVar3,"lock");
  uVar4 = *(ulong *)(unaff_x20 + PANEngineFacade::_engineState);
  _objc_msgSend(IVar3,"unlock");
  if (uVar4 < 2) {
    puStack_98 = auStack_80;
    local_a0 = FUN_000108f4;
    uVar2 = 0;
    FUN_00010904(0,&DAT_000276c8,&PTR_PANMalwareRemoverTaskBuilder_00026e20);
    thunk_FUN_0001391c(&local_58,FUN_00010c70,auStack_b0,uVar2);
    uVar2 = 0;
    FUN_00010904(0,&DAT_000276a8,&PTR__OBJC_CLASS___PANMalware_00026e18);
    thunk_FUN_000138ec(param_2,uVar2);
    IVar3 = _objc_msgSend(local_58,"makeRemoveTaskWithConfiguration:malwareContainer:",param_1,
                          param_2);
  }
  else {
    IVar3 = 0;
    if (uVar4 != 2) {
      local_58 = uVar4;
      thunk_FUN_00013994(&__type_metadata_for_PANEngine_EngineFacade_State,&local_58,
                         &__type_metadata_for_PANEngine_EngineFacade_State,PTR_000200d8);
                    /* WARNING: Does not return */
      pcVar1 = (code *)SoftwareBreakpoint(1,0xc160);
      (*pcVar1)();
    }
  }
  return IVar3;
}
