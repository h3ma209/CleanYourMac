## evidence_id

ev_44e769ee9d4cce7067f3dc8606bef629a6eb7355d5e77659932168619e2a45d3

## subject.name

CleanMyMac_5_MAS_arm64

## subject.digest

| Key    | Value                                                            |
|--------|------------------------------------------------------------------|
| sha256 | aae6adccdd0777bb48932758920030c84dfecbebfa4c9d794b6e620fa4ba3cea |

## subject.format

mach-o

## subject.architecture

arm64

## subject.local_path

/private/tmp/CleanMyMac_5_MAS_arm64

## provider

| Key     | Value  |
|---------|--------|
| id      | ghidra |
| name    | Ghidra |
| version | 12.1.4 |

## analysis_profile.provider

| Key     | Value  |
|---------|--------|
| id      | ghidra |
| name    | Ghidra |
| version | 12.1.4 |

## analysis_profile.parameters.target_kind

executable

## analysis_profile.parameters.target_format

mach-o

## analysis_profile.parameters.architecture

arm64

## analysis_profile.parameters.available_architectures

arm64

## analysis_profile.parameters.import_mode

ephemeral-source-immutable

## analysis_profile.parameters.annotation_policy

atomic-function-entry-metadata-v1

## analysis_profile.parameters.load_image_observations

source-mappings-entry-context-v2

## analysis_profile.parameters.function_body_evidence

complete-inclusive-ranges-v1

## analysis_profile.parameters.function_references

complete-body-and-entry-reference-manager-v2

## analysis_profile.parameters.string_inventory_evidence

defined-data-coverage-v1

## analysis_profile.parameters.location_resolution

explicit-address-exact-entry-symbol-first-v3

## analysis_profile.parameters.instruction_flow_evidence

decoded-return-pcode-v1

## analysis_profile.parameters.function_boundary_observations

ghidra-terminal-call-limitations-v1

## analysis_profile.parameters.no_return_repair

returning-imports-decoded-return-v2

## analysis_profile.parameters.process_launch

inspected-jvm-launch-support-v1

## analysis_profile.parameters.jump_table_evidence

typed-case-default-blocks-v1

## analysis_profile.parameters.decompiler_jump_loads

true

## analysis_profile.parameters.loader

auto-from-header

## analysis_profile.parameters.language_id

auto-from-header

## analysis_profile.parameters.compiler_spec_id

auto-default

## analysis_profile.parameters.analyzer_preset

ghidra-default

## analysis_profile.digest

20565088524a2d0359b9b96f46ac48e679d774c7b0092c56abc06de8fdbec11c

## predicate_type

rea.analysis

## operation

procedure_pseudo_code

## parameters

| Key       | Value       |
|-----------|-------------|
| procedure | 0x100004678 |

## raw_result.value


/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Removing unreachable block (ram,0x0001000048f4) */

undefined8 entry(void)

{
  code *pcVar1;
  bool bVar2;
  __int32 _Var3;
  ComposedApplication CVar4;
  ID IVar5;
  WorkspaceManagement *self;
  ID IVar6;
  undefined8 uVar7;
  ComposedApplication CVar8;
  UnsafeMutablePointer<static_Swift_CommandLine_unsafeArgv_getter_:_Swift_UnsafeMutablePointer<Swift_UnsafeMutablePointer<Swift_Int8>_>>
  UVar9;
  long extraout_x8;
  ComposedApplication with;
  long lVar10;
  undefined1 auStack_50 [16];
  
  CVar4 = SharedUtils::ComposedApplication::typeMetadataAccessor();
  lVar10 = *(long *)(CVar4.unknown + -8);
  (*(code *)PTR____chkstk_darwin_1004d2420)(*(undefined8 *)(lVar10 + 0x40));
  with.unknown = auStack_50 + -(extraout_x8 + 0xfU & 0xfffffffffffffff0);
  FUN_100112538(0);
  FUN_10011041c();
  DAT_100557148 = (NSProcessInfo *)NSProcessInfo::processInfo((ID)&_OBJC_CLASS___NSProcessInfo);
  bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isCommandLineRun(DAT_100557148);
  if (bVar2) {
    IVar5 = objc_stub::arguments((ID)DAT_100557148);
    (extension_Foundation)::Swift::Array<undefined>::__unconditionallyBridgeFromObjectiveC();
    bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isSandbox((NSProcessInfo *)0x0);
    self = SharedUtils::WorkspaceManagement::typeMetadataAccessor();
    _objc_allocWithZone();
    IVar6 = objc_stub::init((ID)self);
    uVar7 = 0;
    FUN_1000a5a9c(0);
    _objc_allocWithZone();
    FUN_100004ce0(IVar5,bVar2,IVar6,uVar7);
    FUN_1000a3c9c();
  }
  else {
    bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isRunningTests(DAT_100557148);
    if ((bVar2) ||
       (bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isRunningSwiftUIPreviews
                          (DAT_100557148), bVar2)) {
      IVar5 = NSApplication::sharedApplication((ID)&_OBJC_CLASS___NSApplication);
      NSApplication::setDelegate_(IVar5,0);
      if (*(ID *)PTR__NSApp_1004d2470 == 0) {
                    /* WARNING: Does not return */
        pcVar1 = (code *)SoftwareBreakpoint(1,0x1000048f4);
        (*pcVar1)();
      }
      objc_stub::run(*(ID *)PTR__NSApp_1004d2470);
    }
    else {
      IVar5 = 0;
      FUN_100004f6c(0,&DAT_100557150,&PTR_CMIssuesReportingApplication_100547af0);
      _swift_getObjCClassFromMetadata();
      IVar5 = objc_stub::sharedApplication(IVar5);
      _swift_dynamicCastObjCClass(IVar5,&objc::class_t::CMIssuesReportingApplication);
      objc_stub::handleComplexCrashes(IVar5);
      uVar7 = 0;
      FUN_100064a9c();
      _swift_allocObject();
      FUN_10004e094();
      FUN_10004d3c0(with.unknown);
      CVar8.unknown = with.unknown;
      (extension_SharedUtils)::__C::NSApplication::setup(with);
      _Var3 = Swift::CommandLine::get_argc((CommandLine)(__int8)CVar8.unknown);
      UVar9 = Swift::CommandLine::_get_unsafeArgv((CommandLine)(__int8)_Var3);
      AppKit::_NSApplicationMain
                (_Var3,(UnsafeMutablePointer<AppKit_NSApplicationMain(Swift_Int32,Swift_UnsafeMutablePointer<Swift_UnsafeMutablePointer<Swift_Int8>_>)__>_Swift_Int32>
                        )UVar9.unknown);
      _swift_release(uVar7);
      (**(code **)(lVar10 + 8))(with.unknown,CVar4.unknown);
    }
  }
  return 0;
}



## raw_result.limitations



## normalized_result


/* WARNING: Function: _objc_opt_self replaced with injection: _objc_retain_fixup */
/* WARNING: Function: _objc_retainAutoreleasedReturnValue replaced with injection:
   _objc_retain_fixup */
/* WARNING: Function: _objc_release replaced with injection: _objc_release_fixup */
/* WARNING: Function: _objc_retain replaced with injection: _objc_retain_fixup */
/* WARNING: Removing unreachable block (ram,0x0001000048f4) */

undefined8 entry(void)

{
  code *pcVar1;
  bool bVar2;
  __int32 _Var3;
  ComposedApplication CVar4;
  ID IVar5;
  WorkspaceManagement *self;
  ID IVar6;
  undefined8 uVar7;
  ComposedApplication CVar8;
  UnsafeMutablePointer<static_Swift_CommandLine_unsafeArgv_getter_:_Swift_UnsafeMutablePointer<Swift_UnsafeMutablePointer<Swift_Int8>_>>
  UVar9;
  long extraout_x8;
  ComposedApplication with;
  long lVar10;
  undefined1 auStack_50 [16];
  
  CVar4 = SharedUtils::ComposedApplication::typeMetadataAccessor();
  lVar10 = *(long *)(CVar4.unknown + -8);
  (*(code *)PTR____chkstk_darwin_1004d2420)(*(undefined8 *)(lVar10 + 0x40));
  with.unknown = auStack_50 + -(extraout_x8 + 0xfU & 0xfffffffffffffff0);
  FUN_100112538(0);
  FUN_10011041c();
  DAT_100557148 = (NSProcessInfo *)NSProcessInfo::processInfo((ID)&_OBJC_CLASS___NSProcessInfo);
  bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isCommandLineRun(DAT_100557148);
  if (bVar2) {
    IVar5 = objc_stub::arguments((ID)DAT_100557148);
    (extension_Foundation)::Swift::Array<undefined>::__unconditionallyBridgeFromObjectiveC();
    bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isSandbox((NSProcessInfo *)0x0);
    self = SharedUtils::WorkspaceManagement::typeMetadataAccessor();
    _objc_allocWithZone();
    IVar6 = objc_stub::init((ID)self);
    uVar7 = 0;
    FUN_1000a5a9c(0);
    _objc_allocWithZone();
    FUN_100004ce0(IVar5,bVar2,IVar6,uVar7);
    FUN_1000a3c9c();
  }
  else {
    bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isRunningTests(DAT_100557148);
    if ((bVar2) ||
       (bVar2 = (extension_SharedUtils)::__C::NSProcessInfo::get_isRunningSwiftUIPreviews
                          (DAT_100557148), bVar2)) {
      IVar5 = NSApplication::sharedApplication((ID)&_OBJC_CLASS___NSApplication);
      NSApplication::setDelegate_(IVar5,0);
      if (*(ID *)PTR__NSApp_1004d2470 == 0) {
                    /* WARNING: Does not return */
        pcVar1 = (code *)SoftwareBreakpoint(1,0x1000048f4);
        (*pcVar1)();
      }
      objc_stub::run(*(ID *)PTR__NSApp_1004d2470);
    }
    else {
      IVar5 = 0;
      FUN_100004f6c(0,&DAT_100557150,&PTR_CMIssuesReportingApplication_100547af0);
      _swift_getObjCClassFromMetadata();
      IVar5 = objc_stub::sharedApplication(IVar5);
      _swift_dynamicCastObjCClass(IVar5,&objc::class_t::CMIssuesReportingApplication);
      objc_stub::handleComplexCrashes(IVar5);
      uVar7 = 0;
      FUN_100064a9c();
      _swift_allocObject();
      FUN_10004e094();
      FUN_10004d3c0(with.unknown);
      CVar8.unknown = with.unknown;
      (extension_SharedUtils)::__C::NSApplication::setup(with);
      _Var3 = Swift::CommandLine::get_argc((CommandLine)(__int8)CVar8.unknown);
      UVar9 = Swift::CommandLine::_get_unsafeArgv((CommandLine)(__int8)_Var3);
      AppKit::_NSApplicationMain
                (_Var3,(UnsafeMutablePointer<AppKit_NSApplicationMain(Swift_Int32,Swift_UnsafeMutablePointer<Swift_UnsafeMutablePointer<Swift_Int8>_>)__>_Swift_Int32>
                        )UVar9.unknown);
      _swift_release(uVar7);
      (**(code **)(lVar10 + 8))(with.unknown,CVar4.unknown);
    }
  }
  return 0;
}



## confidence

observed

## authority

shipped-artifact

## environment

null

## limitations

The session serves operations only after Ghidra reports default auto-analysis complete; incomplete analysis does not expose partial results.,The imported Program and temporary project are ephemeral and deleted on close. Annotation operations edit session database metadata; original executable bytes are never written.,Cancelling an active Ghidra request terminates its ephemeral database and discards session annotations. The selected target remains open; the next provider query imports the original artifact again.,Default-space addresses use lowercase 0x-prefixed hexadecimal; other address spaces use <percent-encoded-space>:0x<hex>.,Pseudocode is Ghidra decompiler output, not original source and not text-equivalent to Hopper output.,External functions and functions without an analyzable body return null; other decompiler failures remain explicit.,macOS sessions require a matching executable Ghidra native decompiler; REA checks for it but does not build native components or change Gatekeeper quarantine state.

## locations



## evidence_links

