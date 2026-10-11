pub mod types;
pub use types::*;

use std::os::raw::{c_char, c_double, c_int, c_uint};

// LLVM integer predicate constants
#[allow(non_upper_case_globals)]
pub const LLVMIntEQ: c_uint = 32;
#[allow(non_upper_case_globals)]
pub const LLVMIntSLT: c_uint = 40;
#[allow(non_upper_case_globals)]
pub const LLVMIntSGT: c_uint = 38;
// #[allow(non_upper_case_globals)]
// pub const LLVMIntEQ:  c_uint = 32; // ==
#[allow(non_upper_case_globals)]
pub const LLVMIntNE:  c_uint = 33; // !=
// #[allow(non_upper_case_globals)]
// pub const LLVMIntUGT: c_uint = 34; // unsigned >
// #[allow(non_upper_case_globals)]
// pub const LLVMIntUGE: c_uint = 35; // unsigned >=
// #[allow(non_upper_case_globals)]
// pub const LLVMIntULT: c_uint = 36; // unsigned
// #[allow(non_upper_case_globals)]
// pub const LLVMIntULE: c_uint = 37; // unsigned <=
// #[allow(non_upper_case_globals)]
// pub const LLVMIntSGT: c_uint = 38; // signed >
#[allow(non_upper_case_globals)]
pub const LLVMIntSGE: c_uint = 39; // signed >=
// #[allow(non_upper_case_globals)]
// pub const LLVMIntSLT: c_uint = 40; // signed
#[allow(non_upper_case_globals)]
pub const LLVMIntSLE: c_uint = 41; // signed <=
// #[allow(non_upper_case_globals)]

// Linkage types
#[allow(non_upper_case_globals)]
pub const LLVMExternalLinkage: c_uint = 0;
#[allow(non_upper_case_globals, dead_code)]
pub const LLVMInternalLinkage: c_uint = 8;
// #[allow(non_upper_case_globals)]
// pub const LLVMPrivateLinkage: c_uint = 9;

// Emit file type
#[allow(non_upper_case_globals)]
pub const LLVMObjectFile: c_uint = 1;
// #[allow(non_upper_case_globals)]
// pub const LLVMAssemblyFile: c_uint = 0;
// #[allow(non_upper_case_globals)]
// pub const LLVMNullFile: c_uint = 2;

// Optimization level constants
// #[allow(non_upper_case_globals)]
// pub const LLVMCodeGenLevelNone: c_uint = 0;

// #[allow(non_upper_case_globals)]
// pub const LLVMCodeGenLevelLess: c_uint = 1;

#[allow(non_upper_case_globals)]
pub const LLVMCodeGenLevelDefault: c_uint = 2;

// #[allow(non_upper_case_globals)]
// pub const LLVMCodeGenLevelAggressive: c_uint = 3;

// Relocation model constants
#[allow(non_upper_case_globals)]
pub const LLVMRelocDefault: c_uint = 0;

// #[allow(non_upper_case_globals)]
// pub const LLVMRelocStatic: c_uint = 1;

// #[allow(non_upper_case_globals)]
// pub const LLVMRelocPIC: c_uint = 2;

// Code model constants
#[allow(non_upper_case_globals)]
pub const LLVMCodeModelDefault: c_uint = 0;

// #[allow(non_upper_case_globals)]
// pub const LLVMCodeModelSmall: c_uint = 2;

// #[allow(non_upper_case_globals)]
// pub const LLVMCodeModelLarge: c_uint = 4;
// #[allow(non_upper_case_globals)]
// pub const LLVMExternalLinkage: c_uint = 0;

// #[allow(non_upper_case_globals)]
// pub const LLVMAbortProcessAction: c_uint = 0;
// #[allow(non_upper_case_globals)]
// pub const LLVMPrintMessageAction: c_uint = 1;
// #[allow(non_upper_case_globals)]
// pub const LLVMReturnStatusAction: c_uint = 2;
#[allow(non_upper_case_globals)]
pub const LLVMVoidTypeKind: c_uint = 0;
// #[allow(non_upper_case_globals)]
// pub const LLVMIntegerTypeKind: c_uint = 8;
// #[allow(non_upper_case_globals)]
// pub const LLVMFloatTypeKind: c_uint = 1;
// #[allow(non_upper_case_globals)]
// pub const LLVMDoubleTypeKind: c_uint = 11;
// #[allow(non_upper_case_globals)]
// pub const LLVMPointerTypeKind: c_uint = 12;

extern "C" {
    // Context
    // pub fn LLVMContextCreate() -> LLVMContextRef;
    // pub fn LLVMContextDispose(C: LLVMContextRef);

    // Module
    pub fn LLVMModuleCreateWithName(ModuleID: *const c_char) -> LLVMModuleRef;
    // pub fn LLVMDisposeModule(M: LLVMModuleRef);
    #[allow(dead_code)]
    pub fn LLVMDumpModule(M: LLVMModuleRef);

    // Types
    pub fn LLVMInt32Type() -> LLVMTypeRef;
    pub fn LLVMInt64Type() -> LLVMTypeRef;
    pub fn LLVMVoidType() -> LLVMTypeRef;
    pub fn LLVMInt1Type() -> LLVMTypeRef;
    pub fn LLVMInt8Type() -> LLVMTypeRef;
    pub fn LLVMInt16Type() -> LLVMTypeRef;
    pub fn LLVMFloatType() -> LLVMTypeRef;
    pub fn LLVMDoubleType() -> LLVMTypeRef;
    pub fn LLVMPointerType(ElementType: LLVMTypeRef, AddressSpace: c_uint) -> LLVMTypeRef;
    pub fn LLVMFunctionType(
        ReturnType: LLVMTypeRef,
        ParamTypes: *mut LLVMTypeRef,
        ParamCount: c_uint,
        IsVarArg: c_int,
    ) -> LLVMTypeRef;

    // Functions
    pub fn LLVMAddFunction(
        M: LLVMModuleRef,
        Name: *const c_char,
        FunctionTy: LLVMTypeRef,
    ) -> LLVMValueRef;
    // pub fn LLVMGetIntTypeWidth(IntegerTy: LLVMTypeRef) -> c_uint;
    pub fn LLVMGetReturnType(FunctionTy: LLVMTypeRef) -> LLVMTypeRef;
    pub fn LLVMGetTypeKind(Ty: LLVMTypeRef) -> c_uint;
    pub fn LLVMBuildCall2(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Fn: LLVMValueRef,
        Args: *mut LLVMValueRef,
        NumArgs: c_uint,
        Name: *const c_char,
    ) -> LLVMValueRef;

    // Basic Blocks
    pub fn LLVMAppendBasicBlock(Fn: LLVMValueRef, Name: *const c_char) -> LLVMBasicBlockRef;

    // Builder
    pub fn LLVMCreateBuilder() -> LLVMBuilderRef;
    // pub fn LLVMDisposeBuilder(Builder: LLVMBuilderRef);
    pub fn LLVMPositionBuilderAtEnd(Builder: LLVMBuilderRef, Block: LLVMBasicBlockRef);

    // Instructions
    pub fn LLVMBuildRet(Builder: LLVMBuilderRef, V: LLVMValueRef) -> LLVMValueRef;

    // Arithmetics
    pub fn LLVMBuildAdd(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    pub fn LLVMBuildSub(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    pub fn LLVMBuildMul(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    pub fn LLVMBuildSDiv(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;

    // Comparison
    pub fn LLVMBuildICmp(
        Builder: LLVMBuilderRef,
        Op: c_uint,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;

    // Memory
    pub fn LLVMBuildAlloca(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    pub fn LLVMBuildStore(
        Builder: LLVMBuilderRef,
        Val: LLVMValueRef,
        Ptr: LLVMValueRef,
    ) -> LLVMValueRef;
    pub fn LLVMBuildLoad2(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        PointerVal: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;

    // String pointer
    pub fn LLVMBuildGlobalStringPtr(
        Builder: LLVMBuilderRef,
        Str: *const c_char,
        Name: *const c_char,
    ) -> LLVMValueRef;

    // Constants
    pub fn LLVMConstInt(IntTy: LLVMTypeRef, N: u64, SignExtend: c_int) -> LLVMValueRef;
    pub fn LLVMConstReal(RealTy: LLVMTypeRef, N: c_double) -> LLVMValueRef;

    pub fn LLVMGetParam(Fn: LLVMValueRef, Index: c_uint) -> LLVMValueRef;
    pub fn LLVMGetLastBasicBlock(Fn: LLVMValueRef) -> LLVMBasicBlockRef;
    pub fn LLVMGetBasicBlockTerminator(BB: LLVMBasicBlockRef) -> LLVMValueRef;
    #[allow(dead_code)]
    pub fn LLVMGetBasicBlockParent(BB: LLVMBasicBlockRef) -> LLVMValueRef;
    #[allow(dead_code)]
    pub fn LLVMGetInsertBlock(Builder: LLVMBuilderRef) -> LLVMBasicBlockRef;
    pub fn LLVMBuildRetVoid(Builder: LLVMBuilderRef) -> LLVMValueRef;
    pub fn LLVMBuildBr(Builder: LLVMBuilderRef, Dest: LLVMBasicBlockRef) -> LLVMValueRef;
    pub fn LLVMBuildCondBr(
        Builder: LLVMBuilderRef,
        If: LLVMValueRef,
        Then: LLVMBasicBlockRef,
        Else: LLVMBasicBlockRef,
    ) -> LLVMValueRef;
    #[allow(dead_code)]
    pub fn LLVMBuildPhi(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    #[allow(dead_code)]
    pub fn LLVMAddIncoming(
        PhiNode: LLVMValueRef,
        IncomingValues: *mut LLVMValueRef,
        IncomingBlocks: *mut LLVMBasicBlockRef,
        Count: c_uint,
    );
    #[allow(dead_code)]
    pub fn LLVMGetUndef(Ty: LLVMTypeRef) -> LLVMValueRef;
    pub fn LLVMTypeOf(Val: LLVMValueRef) -> LLVMTypeRef;
    #[allow(dead_code)]
    pub fn LLVMAddGlobal(M: LLVMModuleRef, Ty: LLVMTypeRef, Name: *const c_char) -> LLVMValueRef;
    #[allow(dead_code)]
    pub fn LLVMSetInitializer(GlobalVar: LLVMValueRef, ConstantVal: LLVMValueRef);
    pub fn LLVMSetLinkage(Global: LLVMValueRef, Linkage: c_uint);

    pub fn LLVMBuildUnreachable(
        Builder: LLVMBuilderRef,
    ) -> LLVMValueRef;

    pub fn LLVMGetAllocatedType(
        AllocaInst: LLVMValueRef,
    ) -> LLVMTypeRef;

    // Link two modules together — other is consumed and becomes invalid
    pub fn LLVMLinkModules2(Dest: LLVMModuleRef, Src: LLVMModuleRef) -> c_int; // returns 0 on success, 1 on error

    // pub fn LLVMVerifyModule(
    //     M: LLVMModuleRef,
    //     Action: c_uint,
    //     OutMessage: *mut *mut c_char,
    // ) -> c_int;

    // Emit object file or assembly from a module
    pub fn LLVMTargetMachineEmitToFile(
        T: LLVMTargetMachineRef,
        M: LLVMModuleRef,
        Filename: *const c_char,
        codegen: c_uint, // LLVMObjectFile or LLVMAssemblyFile
        ErrorMessage: *mut *mut c_char,
    ) -> c_int; // returns 0 on success

    // Target machine creation
    pub fn LLVMCreateTargetMachine(
        T: LLVMTargetRef,
        Triple: *const c_char,
        CPU: *const c_char,
        Features: *const c_char,
        Level: c_uint,     // optimization level
        Reloc: c_uint,     // relocation model
        CodeModel: c_uint, // code model
    ) -> LLVMTargetMachineRef;
    // pub fn LLVMDisposeTargetMachine(T: LLVMTargetMachineRef);

    // Target lookup
    pub fn LLVMGetTargetFromTriple(
        Triple: *const c_char,
        T: *mut LLVMTargetRef,
        ErrorMessage: *mut *mut c_char,
    ) -> c_int;
    // pub fn LLVMGetDefaultTargetTriple() -> *mut c_char;
    pub fn LLVMGetNamedFunction(M: LLVMModuleRef, Name: *const c_char) -> LLVMValueRef;

    // Target initialization — call these before using any target
    pub fn LLVMInitializeX86TargetInfo();
    pub fn LLVMInitializeX86Target();
    pub fn LLVMInitializeX86TargetMC();
    pub fn LLVMInitializeX86AsmPrinter();

    pub fn LLVMInitializeAArch64TargetInfo();
    pub fn LLVMInitializeAArch64Target();
    pub fn LLVMInitializeAArch64TargetMC();
    pub fn LLVMInitializeAArch64AsmPrinter();

    pub fn LLVMInitializeRISCVTargetInfo();
    pub fn LLVMInitializeRISCVTarget();
    pub fn LLVMInitializeRISCVTargetMC();
    pub fn LLVMInitializeRISCVAsmPrinter();

    // Memory management
    pub fn LLVMDisposeMessage(Message: *mut c_char);

    pub fn LLVMDisposeTargetMachine(T: LLVMTargetMachineRef);

    pub fn LLVMGetGlobalContext() -> LLVMContextRef;

    pub fn LLVMStructCreateNamed(
        C: LLVMContextRef,
        Name: *const ::std::os::raw::c_char,
    ) -> LLVMTypeRef;

    pub fn LLVMStructSetBody(
        StructTy: LLVMTypeRef,
        ElementTypes: *mut LLVMTypeRef,
        ElementCount: ::std::os::raw::c_uint,
        Packed: LLVMBool,
    );
}
