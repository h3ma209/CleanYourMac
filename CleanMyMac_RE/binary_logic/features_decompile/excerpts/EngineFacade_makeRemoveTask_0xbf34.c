/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* static PANEngine.EngineFacade.makeRemoveTask(configuration: __C.PANRemovalConfiguration,
   malwareContainer: [__C.PANMalware]) -> __C.PANMalwareRemoverTask? */

undefined8 __thiscall
PANEngine::EngineFacade::_makeRemoveTask
          (undefined8 param_1,undefined8 configuration,undefined8 param_3)

{
  if (DAT_00027358 != -1) {
    _swift_once(&DAT_00027358,FUN_0000d950);
  }
  FUN_0000bfb4(configuration,param_3);
  return configuration;
}
