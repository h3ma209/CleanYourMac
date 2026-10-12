/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* Function Stack Size: 0x20 bytes */

void PANBaseFilesEnumerator::enumerator_iterateFileWithURL_
               (ID param_1,SEL param_2,ID param_3,ID param_4)

{
  ID IVar1;
  ID IVar2;
  unsigned_long_long uVar3;
  
  IVar1 = objc_stub::URLByResolvingSymlinksInPath(param_4);
  if (IVar1 != 0) {
    IVar2 = fileURLsContainer(param_1);
    objc_stub::addObject_(IVar2,IVar1);
  }
  IVar1 = fileURLsContainer(param_1);
  IVar1 = objc_stub::count(IVar1);
  uVar3 = maxFileURLsChunkSize(param_1);
  if (uVar3 < IVar1) {
    IVar1 = objc_stub::delegate(param_1);
    IVar2 = fileURLsContainer(param_1);
    IVar2 = objc_stub::copy(IVar2);
    objc_stub::enumerator_checkChunkOfFileURLs_isFinale_(IVar1,param_1,IVar2,0);
    IVar1 = fileURLsContainer(param_1);
    objc_stub::removeAllObjects(IVar1);
  }
  return;
}
