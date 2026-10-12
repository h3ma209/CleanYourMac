
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* Function Stack Size: 0x10 bytes */

void PANBaseFilesChecker::perform(ID param_1,SEL param_2)

{
  bool bVar1;
  ID IVar2;
  ID IVar3;
  ID IVar4;
  ID IVar5;
  ID IVar6;
  ID IVar7;
  undefined *puVar8;
  ID self;
  long lVar9;
  ID local_178;
  ID local_170;
  undefined *local_168;
  undefined8 local_160;
  code *local_158;
  undefined *puStack_150;
  ID local_148;
  undefined8 local_140;
  long lStack_138;
  long *local_130;
  undefined8 uStack_128;
  undefined8 local_120;
  undefined8 uStack_118;
  undefined8 uStack_110;
  undefined8 uStack_108;
  undefined1 auStack_100 [128];
  long local_80;
  
  local_80 = *(long *)PTR____stack_chk_guard_0006c130;
  IVar2 = numericProgressReporter(param_1);
  IVar3 = fileURLs(param_1);
  IVar3 = objc_stub::count(IVar3);
  objc_stub::addExpectedUnitsCount_(IVar2,IVar3);
  uStack_118 = 0;
  local_120 = 0;
  uStack_108 = 0;
  uStack_110 = 0;
  lStack_138 = 0;
  local_140 = 0;
  uStack_128 = 0;
  local_130 = (long *)0x0;
  IVar2 = fileURLs(param_1);
  local_178 = objc_stub::countByEnumeratingWithState_objects_count_
                        (IVar2,&local_140,auStack_100,0x10);
  if (local_178 != 0) {
    lVar9 = *local_130;
    puVar8 = PTR___NSConcreteStackBlock_0006c128;
    do {
      IVar3 = 0;
      do {
        if (*local_130 != lVar9) {
          _objc_enumerationMutation(IVar2);
        }
        self = *(ID *)(lStack_138 + IVar3 * 8);
        IVar4 = executionControl(param_1);
        IVar4 = objc_stub::sleepIfPausedOrExitIfCanceled(IVar4);
        if ((IVar4 & 1) != 0) goto LAB_0001e0f4;
        local_160 = 0xc2000000;
        local_158 = FUN_0001e13c;
        puStack_150 = &DAT_0006c5d8;
        local_168 = puVar8;
        local_148 = self;
        IVar4 = PANFreezeDetector::detectFreezeWithUserInfo_
                          ((ID)&_OBJC_CLASS___PANFreezeDetector,&local_168);
        bVar1 = isCheckable_(param_1,self);
        if (bVar1 == 0) {
          IVar5 = 0;
        }
        else {
          IVar5 = detectionService(param_1);
          IVar6 = executionControl(param_1);
          local_170 = 0;
          IVar5 = objc_stub::checkURL_executionControl_dataSource_error_
                            (IVar5,self,IVar6,0,&local_170);
          puVar8 = PTR___NSConcreteStackBlock_0006c128;
          if ((IVar5 == 0) && (local_170 != 0)) {
            pan_notifyError_(param_1,local_170);
            puVar8 = PTR___NSConcreteStackBlock_0006c128;
          }
        }
        IVar6 = textualProgressReporter(param_1);
        IVar7 = objc_stub::path(self);
        objc_stub::report_(IVar6,IVar7);
        IVar6 = numericProgressReporter(param_1);
        objc_stub::incrementCompletedUnitsCount(IVar6);
        PANFreezeDetector::cancel_((ID)&_OBJC_CLASS___PANFreezeDetector,IVar4);
        IVar4 = objc_stub::delegate(param_1);
        if (IVar5 == 0) {
          objc_stub::checker_handleCheckedFileURL_(IVar4,param_1,self);
        }
        else {
          objc_stub::checker_handleMalwareDetection_fileURL_(IVar4,param_1,IVar5,self);
        }
        IVar3 = IVar3 + 1;
      } while (local_178 != IVar3);
      local_178 = objc_stub::countByEnumeratingWithState_objects_count_
                            (IVar2,&local_140,auStack_100,0x10);
    } while (local_178 != 0);
  }
LAB_0001e0f4:
  if (*(long *)PTR____stack_chk_guard_0006c130 != local_80) {
                    /* WARNING: Subroutine does not return */
    ___stack_chk_fail(0);
  }
  return;
}

