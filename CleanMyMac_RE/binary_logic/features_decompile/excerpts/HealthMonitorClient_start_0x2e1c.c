

/* HealthMonitoringAPI.HealthMonitorClient.start() -> () */

void __thiscall HealthMonitoringAPI::HealthMonitorClient::start(HealthMonitorClient *this)

{
  code *pcVar1;
  undefined8 uVar2;
  long lVar3;
  long extraout_x8;
  undefined1 *puVar4;
  long lVar5;
  
  lVar3 = 0;
  thunk_FUN_000119d0();
  lVar5 = *(long *)(lVar3 + -8);
  (*(code *)PTR____chkstk_darwin_00018128)(*(undefined8 *)(lVar5 + 0x40));
  puVar4 = &stack0xffffffffffffffd0 + -(extraout_x8 + 0xfU & 0xfffffffffffffff0);
  thunk_FUN_000119c4(puVar4);
  thunk_FUN_000119dc(puVar4,FUN_00002ee0,0,0,1,0xd00000000000002d,0x80000000000168d0);
  (**(code **)(lVar5 + 8))(puVar4,lVar3);
  pcVar1 = *(code **)(*(long *)(this + 0x30) + 0x10);
  uVar2 = *(undefined8 *)(*(long *)(this + 0x30) + 0x18);
  _swift_retain(uVar2);
  (*pcVar1)();
  _swift_release(uVar2);
  return;
}



## raw_result.limitations





/* HealthMonitoringAPI.HealthMonitorClient.start() -> () */

void __thiscall HealthMonitoringAPI::HealthMonitorClient::start(HealthMonitorClient *this)

{
  code *pcVar1;
  undefined8 uVar2;
  long lVar3;
  long extraout_x8;
  undefined1 *puVar4;
  long lVar5;
  
  lVar3 = 0;
  thunk_FUN_000119d0();
  lVar5 = *(long *)(lVar3 + -8);
  (*(code *)PTR____chkstk_darwin_00018128)(*(undefined8 *)(lVar5 + 0x40));
  puVar4 = &stack0xffffffffffffffd0 + -(extraout_x8 + 0xfU & 0xfffffffffffffff0);
  thunk_FUN_000119c4(puVar4);
  thunk_FUN_000119dc(puVar4,FUN_00002ee0,0,0,1,0xd00000000000002d,0x80000000000168d0);
  (**(code **)(lVar5 + 8))(puVar4,lVar3);
  pcVar1 = *(code **)(*(long *)(this + 0x30) + 0x10);
  uVar2 = *(undefined8 *)(*(long *)(this + 0x30) + 0x18);
  _swift_retain(uVar2);
  (*pcVar1)();
  _swift_release(uVar2);
  return;
}



