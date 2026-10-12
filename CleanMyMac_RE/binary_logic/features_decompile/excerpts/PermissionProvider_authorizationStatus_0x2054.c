

/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* PermissionsUtils.PermissionProvider.authorizationStatus(for: PermissionsUtils.Permission) ->
   PermissionsUtils.AuthStatus */

AuthStatus __thiscall
PermissionsUtils::PermissionProvider::authorizationStatus(PermissionProvider *this,Permission for)

{
  byte bVar1;
  undefined1 uVar2;
  AuthStatus AVar3;
  undefined *puVar4;
  undefined8 in_x1;
  undefined1 *in_x8;
  
  bVar1 = *(byte *)(ulong)(byte)for.value;
  if (bVar1 < 2) {
    uVar2 = bVar1 != 0;
  }
  else if (bVar1 == 2) {
    uVar2 = 2;
  }
  else {
    if (bVar1 != 3) {
      AVar3 = fullDiskAccessStatus(this);
      puVar4 = (undefined *)(ulong)(byte)AVar3.value;
      goto LAB_00002100;
    }
    uVar2 = 3;
  }
  puVar4 = &_OBJC_CLASS___MPPermissionsKit;
  FUN_00003e20(&_OBJC_CLASS___MPPermissionsKit,in_x1,uVar2);
  if ((long)puVar4 < 2) {
    if ((puVar4 == (undefined *)0x0) || (puVar4 != (undefined *)((long)&MACH_HEADER.magic + 1))) {
LAB_000020e4:
      *in_x8 = 0;
      goto LAB_00002100;
    }
    uVar2 = 1;
  }
  else if (puVar4 == (undefined *)((long)&MACH_HEADER.magic + 3)) {
    uVar2 = 3;
  }
  else {
    if (puVar4 != (undefined *)((long)&MACH_HEADER.magic + 2)) goto LAB_000020e4;
    uVar2 = 2;
  }
  *in_x8 = uVar2;
LAB_00002100:
  return (AuthStatus)(__int8)puVar4;
}



## raw_result.limitations





/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* PermissionsUtils.PermissionProvider.authorizationStatus(for: PermissionsUtils.Permission) ->
   PermissionsUtils.AuthStatus */

AuthStatus __thiscall
PermissionsUtils::PermissionProvider::authorizationStatus(PermissionProvider *this,Permission for)

{
  byte bVar1;
  undefined1 uVar2;
  AuthStatus AVar3;
  undefined *puVar4;
  undefined8 in_x1;
  undefined1 *in_x8;
  
  bVar1 = *(byte *)(ulong)(byte)for.value;
  if (bVar1 < 2) {
    uVar2 = bVar1 != 0;
  }
  else if (bVar1 == 2) {
    uVar2 = 2;
  }
  else {
    if (bVar1 != 3) {
      AVar3 = fullDiskAccessStatus(this);
      puVar4 = (undefined *)(ulong)(byte)AVar3.value;
      goto LAB_00002100;
    }
    uVar2 = 3;
  }
  puVar4 = &_OBJC_CLASS___MPPermissionsKit;
  FUN_00003e20(&_OBJC_CLASS___MPPermissionsKit,in_x1,uVar2);
  if ((long)puVar4 < 2) {
    if ((puVar4 == (undefined *)0x0) || (puVar4 != (undefined *)((long)&MACH_HEADER.magic + 1))) {
LAB_000020e4:
      *in_x8 = 0;
      goto LAB_00002100;
    }
    uVar2 = 1;
  }
  else if (puVar4 == (undefined *)((long)&MACH_HEADER.magic + 3)) {
    uVar2 = 3;
  }
  else {
    if (puVar4 != (undefined *)((long)&MACH_HEADER.magic + 2)) goto LAB_000020e4;
    uVar2 = 2;
  }
  *in_x8 = uVar2;
LAB_00002100:
  return (AuthStatus)(__int8)puVar4;
}



