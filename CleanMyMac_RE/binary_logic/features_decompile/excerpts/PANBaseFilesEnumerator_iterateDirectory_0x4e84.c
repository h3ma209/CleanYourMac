
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* Function Stack Size: 0x20 bytes */

bool PANBaseFilesEnumerator::enumerator_iterateDirectoryWithURL_
               (ID param_1,SEL param_2,ID param_3,ID param_4)

{
  ID IVar1;
  
  IVar1 = objc_stub::delegate(param_1);
  IVar1 = objc_stub::enumerator_checkDirectoryDescendantsWithURL_(IVar1,param_1,param_4);
  return (bool)IVar1;
}

