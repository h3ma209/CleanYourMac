
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* Function Stack Size: 0x10 bytes */

void PANBaseFilesEnumerator::perform(ID param_1,SEL param_2)

{
  unsigned_long_long uVar1;
  ID IVar2;
  ID IVar3;
  
  uVar1 = maxFileURLsChunkSize(param_1);
  IVar2 = NSMutableArray::arrayWithCapacity_((ID)&_OBJC_CLASS___NSMutableArray,uVar1);
  setFileURLsContainer_(param_1,IVar2);
  IVar2 = makeFileTreeEnumerator(param_1);
  objc_stub::setDelegate_(IVar2,param_1);
  objc_stub::perform(IVar2);
  IVar2 = objc_stub::delegate(param_1);
  IVar3 = fileURLsContainer(param_1);
  IVar3 = objc_stub::copy(IVar3);
  objc_stub::enumerator_checkChunkOfFileURLs_isFinale_(IVar2,param_1,IVar3,1);
  return;
}

