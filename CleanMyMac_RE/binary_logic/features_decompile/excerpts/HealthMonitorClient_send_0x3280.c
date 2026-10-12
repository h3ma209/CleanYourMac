

/* HealthMonitoringAPI.HealthMonitorClient.send<A where A: HealthMonitoringAPI.RequestType>(request:
   A) -> Combine.AnyPublisher<A.ReplyType, Swift.Error> */

AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
 __thiscall
HealthMonitoringAPI::HealthMonitorClient::_send
          (HealthMonitorClient *this,
          AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
          param_2,undefined8 param_3,long param_4)

{
  long lVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  undefined8 uVar4;
  code *pcVar5;
  
  uVar2 = *(undefined8 *)(this + 0x18);
  lVar1 = *(long *)(this + 0x20);
  _swift_getObjectType(uVar2);
  uVar3 = 0;
  _swift_getAssociatedTypeWitness
            (0,param_4,param_3,0x159d4,
             &__associated_type_descriptor_for_HealthMonitoringAPI_RequestType_ReplyType);
  pcVar5 = *(code **)(lVar1 + 8);
  uVar4 = *(undefined8 *)(param_4 + 0x10);
  _swift_getAssociatedConformanceWitness
            (param_4,param_3,uVar3,0x159d4,
             &
             __associated_conformance_descriptor_for_HealthMonitoringAPI_RequestType_HealthMonitoringAPI_RequestType_ReplyType__Swift_Decodable
            );
  (*pcVar5)(param_2.unknown,uVar3,param_3,uVar3,uVar4,param_4,uVar2,lVar1);
  return (AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
          )param_2.unknown;
}



## raw_result.limitations





/* HealthMonitoringAPI.HealthMonitorClient.send<A where A: HealthMonitoringAPI.RequestType>(request:
   A) -> Combine.AnyPublisher<A.ReplyType, Swift.Error> */

AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
 __thiscall
HealthMonitoringAPI::HealthMonitorClient::_send
          (HealthMonitorClient *this,
          AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
          param_2,undefined8 param_3,long param_4)

{
  long lVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  undefined8 uVar4;
  code *pcVar5;
  
  uVar2 = *(undefined8 *)(this + 0x18);
  lVar1 = *(long *)(this + 0x20);
  _swift_getObjectType(uVar2);
  uVar3 = 0;
  _swift_getAssociatedTypeWitness
            (0,param_4,param_3,0x159d4,
             &__associated_type_descriptor_for_HealthMonitoringAPI_RequestType_ReplyType);
  pcVar5 = *(code **)(lVar1 + 8);
  uVar4 = *(undefined8 *)(param_4 + 0x10);
  _swift_getAssociatedConformanceWitness
            (param_4,param_3,uVar3,0x159d4,
             &
             __associated_conformance_descriptor_for_HealthMonitoringAPI_RequestType_HealthMonitoringAPI_RequestType_ReplyType__Swift_Decodable
            );
  (*pcVar5)(param_2.unknown,uVar3,param_3,uVar3,uVar4,param_4,uVar2,lVar1);
  return (AnyPublisher<HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>(request:_A)__>_Combine_AnyPublisher<A_ReplyType,Swift_Error>_HealthMonitoringAPI_HealthMonitorClient_send<A_where_A:_HealthMonitoringAPI_RequestType>_request__A___>_Combine_AnyPublisher<A_ReplyType,Swift_Error>>
          )param_2.unknown;
}



