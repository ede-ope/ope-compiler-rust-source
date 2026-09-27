use crate::{
    borrow_checker::{CustTypeInfo, FuncDeclInfo, FuncExpr},
    cfg_resourses::{BlockId, Inst, InstKind, Terminator, CFG},
    llvm::*,
    parser::{EnumVariants, Expr, ExprKind, Ident, InterfaceParam, TypeHint, TypeSpec},
    scanner::{FullTok, Token, TruthVal},
    ComdInfo,
};
use std::{collections::HashMap, io, os::raw::c_int, path::PathBuf, thread, time::Duration};
use std::{io::Write, process::Output};

// ── Elem — symbol table entry ────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Elem {
    Var(LLVMValueRef),
    Global(LLVMValueRef, LLVMTypeRef),
    Function(LLVMValueRef, LLVMTypeRef),
    ExternFunc(LLVMValueRef, LLVMTypeRef),
    Struct {
        ty: LLVMTypeRef,
        fields: HashMap<String, u32>,
    },
    Enum {
        ty: LLVMTypeRef,
        variants: HashMap<String, u64>,
    },
    Trait {
        methods: HashMap<String, (LLVMValueRef, LLVMTypeRef)>,
    },
    Type(LLVMTypeRef),
}

// ── Codegen ──────────────────────────────────────────────────────────────────
pub struct Codegen {
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
    pub symbol_table: Vec<HashMap<String, Elem>>, // scope stack
    pub strings: HashMap<String, LLVMValueRef>,   // string dedup

    // CFG block id → LLVM basic block — built while walking the CFG
    llvm_blocks: HashMap<BlockId, LLVMBasicBlockRef>,
}

impl Codegen {
    pub fn new(module_name: &str) -> Self {
        unsafe {
            let module =
                LLVMModuleCreateWithName(format!("{}\0", module_name).as_ptr() as *const _);
            let builder = LLVMCreateBuilder();
            Self {
                module,
                builder,
                symbol_table: vec![HashMap::new()],
                strings: HashMap::new(),
                llvm_blocks: HashMap::new(),
            }
        }
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn cstring(&self, s: &str) -> std::ffi::CString {
        std::ffi::CString::new(s).unwrap()
    }

    fn ident_cstr(&self, s: &str) -> std::ffi::CString {
        self.cstring(s)
    }

    // ── scope management ─────────────────────────────────────────────────────

    fn push_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop();
    }

    fn enter_function(&mut self) {
        self.symbol_table.truncate(1); // keep global scope only
        self.push_scope(); // push fresh function scope
        self.llvm_blocks.clear(); // clear block map for new function
    }

    // ── symbol declaration ───────────────────────────────────────────────────

    fn declare(&mut self, name: String, elem: Elem) {
        self.symbol_table.last_mut().unwrap().insert(name, elem);
    }

    fn declare_global(&mut self, name: String, elem: Elem) {
        self.symbol_table.first_mut().unwrap().insert(name, elem);
    }

    // ── symbol lookup ────────────────────────────────────────────────────────

    fn lookup(&self, name: &str) -> Option<&Elem> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(elem) = scope.get(name) {
                return Some(elem);
            }
        }
        None
    }

    fn lookup_var(&self, name: &str) -> Option<LLVMValueRef> {
        match self.lookup(name)? {
            Elem::Var(val) => Some(*val),
            Elem::Global(val, _) => Some(*val),
            _ => None,
        }
    }

    fn lookup_function(&self, name: &str) -> Option<(LLVMValueRef, LLVMTypeRef)> {
        match self.lookup(name)? {
            Elem::Function(val, ty) => Some((*val, *ty)),
            Elem::ExternFunc(val, ty) => Some((*val, *ty)),
            _ => None,
        }
    }

    fn lookup_struct(&self, name: &str) -> Option<(LLVMTypeRef, &HashMap<String, u32>)> {
        match self.lookup(name)? {
            Elem::Struct { ty, fields } => Some((*ty, fields)),
            _ => None,
        }
    }

    // ── lvalue resolution ────────────────────────────────────────────────────

    fn gen_lvalue(&self, expr: &Expr) -> LLVMValueRef {
        match &expr.kind {
            ExprKind::MutableIdent { var, fields } => {
                if fields.is_empty() {
                    match self.lookup(&var.ident) {
                        Some(Elem::Var(val)) => *val,
                        _ => panic!("undefined or non-mutable variable: {}", var.ident),
                    }
                } else {
                    todo!("mutable ident with fields (GEP)")
                }
            }
            _ => todo!("complex lvalue not yet implemented"),
        }
    }

    // ── compound assignment ──────────────────────────────────────────────────

    fn _gen_compound_op(
        &mut self,
        lhs: LLVMValueRef,
        op: &Token,
        rhs: LLVMValueRef,
    ) -> LLVMValueRef {
        unsafe {
            match op {
                Token::Plus => LLVMBuildAdd(self.builder, lhs, rhs, c"addtmp".as_ptr()),
                Token::Minus => LLVMBuildSub(self.builder, lhs, rhs, c"subtmp".as_ptr()),
                Token::Star => LLVMBuildMul(self.builder, lhs, rhs, c"multmp".as_ptr()),
                Token::Slash => LLVMBuildSDiv(self.builder, lhs, rhs, c"divtmp".as_ptr()),
                _ => todo!("compound op not yet implemented"),
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // CFG → LLVM codegen
    // ═════════════════════════════════════════════════════════════════════════

    /// Generate an LLVM function from a FuncDeclInfo whose body is a CFG.
    pub fn gen_func(&mut self, name: &str, info: &FuncDeclInfo) -> Option<LLVMValueRef> {
        if let Some(from) = &info.is_extern {
            let abi = &from.0;
            let is_variadic = from.1.is_some();
            if abi.ident == "C" {
                let (func, _) =
                    self.gen_extern_function(name, &info.expect, &info.par, is_variadic);
                Some(func)
            } else {
                None
            }
        } else {
            Some(self.gen_function(name, info))
        }
    }

    /// Generate the entry point (main).
    pub fn gen_entry(&mut self, info: &FuncDeclInfo) -> LLVMValueRef {
        self.enter_function();
        unsafe {
            let ret_type = self.gen_type(&info.expect);
            let fn_type = LLVMFunctionType(ret_type, std::ptr::null_mut(), 0, 0);
            let func = LLVMAddFunction(self.module, c"main".as_ptr(), fn_type);

            match &info.block {
                FuncExpr::Cfg(cfg) => {
                    self.gen_cfg_body(func, cfg, &info.par, &info.expect);
                }
                _ => (),
            }

            func
        }
    }

    /// Core: generate a non-extern function from its FuncDeclInfo.
    // fn gen_function(&mut self, name: &str, info: &FuncDeclInfo) -> LLVMValueRef {
    //     unsafe {
    //         let mut param_types: Vec<LLVMTypeRef> =
    //             info.par.iter().map(|p| self.gen_type(&p.ty)).collect();
    //         let ret_type = self.gen_type(&info.expect);
    //         let fn_type = LLVMFunctionType(
    //             ret_type,
    //             param_types.as_mut_ptr(),
    //             param_types.len() as u32,
    //             0,
    //         );

    //         let cname = self.cstring(name);

    //         // ← check if already declared in this module
    //         let func = {
    //             let existing = LLVMGetNamedFunction(self.module, cname.as_ptr());
    //             if !existing.is_null() {
    //                 // reuse the existing declaration — just add a body to it
    //                 existing
    //             } else {
    //                 // create new function
    //                 LLVMAddFunction(self.module, cname.as_ptr(), fn_type)
    //             }
    //         };

    //         // set external linkage for public functions
    //         if info.is_public {
    //             LLVMSetLinkage(func, LLVMExternalLinkage);
    //         }

    //         // register in symbol table
    //         self.declare_global(name.to_string(), Elem::Function(func, fn_type));

    //         // create entry block and generate body
    //         self.gen_cfg_body(func, &info.block, &info.par, &info.expect, None);

    //         func
    //     }
    // }
    fn gen_function(&mut self, name: &str, info: &FuncDeclInfo) -> LLVMValueRef {
        self.enter_function();
        unsafe {
            // 1. param types
            let mut param_types: Vec<LLVMTypeRef> =
                info.par.iter().map(|p| self.gen_type(&p.ty)).collect();

            // 2. function type + LLVM function
            let ret_type = self.gen_type(&info.expect);
            let fn_type = LLVMFunctionType(
                ret_type,
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                0,
            );
            let fname = self.ident_cstr(name);
            let func = {
                let existing = LLVMGetNamedFunction(self.module, fname.as_ptr());
                if !existing.is_null() {
                    // reuse the existing declaration — just add a body to it
                    existing
                } else {
                    // create new function
                    LLVMAddFunction(self.module, fname.as_ptr(), fn_type)
                }
            };
            // let func = LLVMAddFunction(self.module, fname.as_ptr(), fn_type);

            if info.is_public {
                LLVMSetLinkage(func, LLVMExternalLinkage);
            } else {
                LLVMSetLinkage(func, LLVMInternalLinkage);
            }

            // 3. register before body so recursive calls resolve
            self.declare_global(name.to_string(), Elem::Function(func, fn_type));

            match &info.block {
                FuncExpr::Cfg(cfg) => {
                    self.gen_cfg_body(func, cfg, &info.par, &info.expect);
                }
                _ => (),
            }

            func
        }
    }

    /// Walk the CFG and emit LLVM IR for every basic block.
    ///
    /// Strategy:
    ///   Pass 1 — pre-create one LLVM basic block per CFG block so forward
    ///            branches (`Jump`, `Branch`) can reference them immediately.
    ///   Pass 2 — fill each block: bind params (entry only), emit instructions,
    ///            emit terminator.
    fn gen_cfg_body(
        &mut self,
        func: LLVMValueRef,
        cfg: &CFG,
        params: &[InterfaceParam],
        expect: &TypeSpec,
    ) {
        unsafe {
            // ── Pass 1: pre-create LLVM basic blocks ──────────────────────
            for block in &cfg.blocks {
                let label = self.cstring(&block.label);
                let llvm_bb = LLVMAppendBasicBlock(func, label.as_ptr());
                self.llvm_blocks.insert(block.id, llvm_bb);
            }

            // ── Bind parameters in the entry block ────────────────────────
            let entry_llvm_bb = self.llvm_blocks[&cfg.entry];
            LLVMPositionBuilderAtEnd(self.builder, entry_llvm_bb);

            for (i, param) in params.iter().enumerate() {
                let llvm_param = LLVMGetParam(func, i as u32);
                let pname = self.ident_cstr(&param.ident.ident);
                let alloca =
                    LLVMBuildAlloca(self.builder, self.gen_type(&param.ty), pname.as_ptr());
                LLVMBuildStore(self.builder, llvm_param, alloca);
                self.declare(param.ident.ident.clone(), Elem::Var(alloca));
            }

            // ── Pass 2: emit each block ───────────────────────────────────
            for block in &cfg.blocks {
                let llvm_bb = self.llvm_blocks[&block.id];

                // If this is not the entry, position builder; for the entry
                // the builder is already there (after param binding above).
                if block.id != cfg.entry {
                    LLVMPositionBuilderAtEnd(self.builder, llvm_bb);
                    self.push_scope(); // new scope per CFG block
                }

                // emit instructions
                for inst in &block.insts {
                    self.gen_inst(inst);
                }

                // emit terminator
                match &block.terminator {
                    Some(Terminator::Jump(target)) => {
                        let target_bb = self.llvm_blocks[target];
                        LLVMBuildBr(self.builder, target_bb);
                    }

                    Some(Terminator::Branch {
                        cond,
                        true_bb,
                        false_bb,
                    }) => {
                        let cond_val = self.gen_expr(cond);
                        let true_llvm = self.llvm_blocks[true_bb];
                        let false_llvm = self.llvm_blocks[false_bb];
                        LLVMBuildCondBr(self.builder, cond_val, true_llvm, false_llvm);
                    }

                    Some(Terminator::Return(Some(expr))) => {
                        let val = self.gen_expr(expr);
                        LLVMBuildRet(self.builder, val);
                    }

                    Some(Terminator::Return(None)) => {
                        LLVMBuildRetVoid(self.builder);
                    }

                    Some(Terminator::Unreachable) => {
                        LLVMBuildUnreachable(self.builder);
                    }

                    // Block has no terminator — insert fallback
                    None => {
                        let last_bb = LLVMGetLastBasicBlock(func);
                        if LLVMGetBasicBlockTerminator(last_bb).is_null() {
                            match &expect.hint {
                                TypeHint::Void => {
                                    LLVMBuildRetVoid(self.builder);
                                }
                                _ => {
                                    let zero = LLVMConstInt(self.gen_type(expect), 0, 0);
                                    LLVMBuildRet(self.builder, zero);
                                }
                            }
                        }
                    }
                }

                // pop block scope (entry scope is popped when function finishes)
                if block.id != cfg.entry {
                    self.pop_scope();
                }
            }
        }
    }

    // ── instruction codegen ──────────────────────────────────────────────────

    fn gen_inst(&mut self, inst: &Inst) -> Option<LLVMValueRef> {
        match &inst.kind {
            // let x: T = expr
            InstKind::Assign {
                ty, name, value, ..
            } => {
                if value.kind == ExprKind::Undefined {
                    // uninitialized declaration — alloca only
                    unsafe {
                        let llvm_type = self.gen_type(ty);
                        let cname = self.ident_cstr(&name.ident);
                        let alloca = LLVMBuildAlloca(self.builder, llvm_type, cname.as_ptr());
                        self.declare(name.ident.clone(), Elem::Var(alloca));
                    }
                } else {
                    self.gen_var_decl(name, ty, value);
                }
                None
            }

            // x = expr  (VarInit — first assignment to uninitialized var)
            InstKind::Store {
                name,
                value,
                is_init,
                ..
            } => {
                unsafe {
                    let val = self.gen_expr(value);
                    let alloca = self
                        .lookup_var(&name.ident)
                        .expect(&format!("undefined variable: {}", name.ident));
                    LLVMBuildStore(self.builder, val, alloca);
                    // if is_init, mark the variable as now initialized
                    // (ownership tracking handled by borrow checker — nothing to do here)
                    let _ = is_init;
                }
                None
            }

            // x op= expr  (ReAssign)
            InstKind::ReStore { to_mut, val, .. } => {
                unsafe {
                    let ptr = self.gen_lvalue(to_mut);
                    let rhs = self.gen_expr(val);
                    let final_val = rhs;
                    // if let Some(op) = assign_op {
                    //     let cur = LLVMBuildLoad2(
                    //         self.builder,
                    //         LLVMTypeOf(ptr),
                    //         ptr,
                    //         c"loadtmp".as_ptr(),
                    //     );
                    //     self.gen_compound_op(cur, &op.kind, rhs)
                    // } else {
                    //     rhs
                    // };
                    LLVMBuildStore(self.builder, final_val, ptr);
                }
                None
            }

            // &x — shared borrow: borrower is just an alias alloca for now
            InstKind::BorrowShared { result, lender, .. } => {
                unsafe {
                    // store the lender's alloca pointer as the result's value
                    let lender_ptr = self
                        .lookup_var(&lender.ident)
                        .expect(&format!("undefined variable for borrow: {}", lender.ident));
                    let cname = self.ident_cstr(&result.ident);
                    let alloca =
                        LLVMBuildAlloca(self.builder, LLVMTypeOf(lender_ptr), cname.as_ptr());
                    LLVMBuildStore(self.builder, lender_ptr, alloca);
                    self.declare(result.ident.clone(), Elem::Var(alloca));
                }
                None
            }

            // &mut x — mutable borrow: same representation as shared for codegen
            InstKind::BorrowMut { result, lender, .. } => {
                unsafe {
                    let lender_ptr = self.lookup_var(&lender.ident).expect(&format!(
                        "undefined variable for mut borrow: {}",
                        lender.ident
                    ));
                    let cname = self.ident_cstr(&result.ident);
                    let alloca =
                        LLVMBuildAlloca(self.builder, LLVMTypeOf(lender_ptr), cname.as_ptr());
                    LLVMBuildStore(self.builder, lender_ptr, alloca);
                    self.declare(result.ident.clone(), Elem::Var(alloca));
                }
                None
            }

            // copy: kí ọsán b: a  (non-move read)
            InstKind::Use { used, by } => {
                unsafe {
                    // load from source, store into fresh alloca for `by`
                    let src_alloca = self
                        .lookup_var(&used.ident)
                        .expect(&format!("undefined variable in Use: {}", used.ident));
                    let loaded = LLVMBuildLoad2(
                        self.builder,
                        LLVMGetAllocatedType(src_alloca),
                        src_alloca,
                        self.ident_cstr(&used.ident).as_ptr(),
                    );
                    let by_type = LLVMTypeOf(loaded);
                    let cname = self.ident_cstr(&by.ident);
                    let alloca = LLVMBuildAlloca(self.builder, by_type, cname.as_ptr());
                    LLVMBuildStore(self.builder, loaded, alloca);
                    self.declare(by.ident.clone(), Elem::Var(alloca));
                }
                None
            }

            // move: invalidates source, binds destination
            InstKind::Move { from, to } => {
                unsafe {
                    let src_alloca = self
                        .lookup_var(&from.ident)
                        .expect(&format!("undefined variable in Move: {}", from.ident));
                    // transfer the alloca — `to` now owns the same storage
                    // (the borrow checker already validated this is safe)
                    let alloca_ty = LLVMGetAllocatedType(src_alloca);
                    let cname = self.ident_cstr(&to.ident);
                    let dst = LLVMBuildAlloca(self.builder, alloca_ty, cname.as_ptr());
                    let loaded =
                        LLVMBuildLoad2(self.builder, alloca_ty, src_alloca, c"movedtmp".as_ptr());
                    LLVMBuildStore(self.builder, loaded, dst);
                    self.declare(to.ident.clone(), Elem::Var(dst));
                    // remove `from` from symbol table so future uses panic
                    for scope in self.symbol_table.iter_mut().rev() {
                        if scope.remove(&from.ident).is_some() {
                            break;
                        }
                    }
                }
                None
            }

            // explicit drop — remove from symbol table
            InstKind::Drop { name } => {
                for scope in self.symbol_table.iter_mut().rev() {
                    if scope.remove(&name.ident).is_some() {
                        break;
                    }
                }
                None
            }

            // function call
            InstKind::Call { func, args, result } => {
                let arg_exprs: Vec<Expr> = args.iter().map(|(e, _)| e.clone()).collect();
                let call_val = self.gen_func_call(func, &arg_exprs);
                if let Some(res_name) = result {
                    // bind the result to a named alloca
                    unsafe {
                        let ty = LLVMTypeOf(call_val);
                        let cname = self.ident_cstr(&res_name.ident);
                        let alloca = LLVMBuildAlloca(self.builder, ty, cname.as_ptr());
                        LLVMBuildStore(self.builder, call_val, alloca);
                        self.declare(res_name.ident.clone(), Elem::Var(alloca));
                    }
                }
                Some(call_val)
            }
        }
    }

    // ── expression codegen ───────────────────────────────────────────────────

    pub fn gen_expr(&mut self, expr: &Expr) -> LLVMValueRef {
        match &expr.kind {
            ExprKind::Number(n) => panic!("Number expression {} should have been lowered to typed on line {}", n, expr.span.st.line),
            ExprKind::Str(s) => self.gen_string(s),
            ExprKind::Binary { lhs, op, rhs, .. } => self.gen_binary(lhs, op, rhs),
            ExprKind::Ident { ident, .. } => self.gen_ident(ident),
            ExprKind::ArgBuffer { name, args, .. } => self.gen_func_call(name, args),
            ExprKind::TruthVal(tv) => self.gen_bool(tv),
            ExprKind::Char(c) => self.gen_char(*c),
            ExprKind::StructVal { ident, vals } => {
                todo!(
                    "StructVal codegen not yet implemented: {:?} with fields {:?}",
                    ident,
                    vals
                )
            }
            ExprKind::Address(_, inner) => {
                match &inner.kind {
                    ExprKind::Ident { ident, .. } => {
                        match self.lookup(&ident.ident) {
                            Some(Elem::Var(alloca)) => {
                                let alloca = *alloca;
                                unsafe {
                                    // load the ptr stored in the alloca
                                    // gives us the actual *i8 pointing to data
                                    LLVMBuildLoad2(
                                        self.builder,
                                        LLVMPointerType(LLVMInt8Type(), 0),
                                        alloca,
                                        c"strptr".as_ptr(),
                                    )
                                }
                            }
                            _ => panic!("undefined: {}", ident.ident),
                        }
                    }
                    _ => todo!("complex address-of"),
                }
            }
            ExprKind::AsType { expr, ty } => {
                match expr.kind.clone() {
                    ExprKind::Number(n) => {
                        match ty.hint {
                            TypeHint::I8(_) => {
                                let val: Result<i8, std::num::ParseIntError> =
                                    if let Some(hex) = n.strip_prefix("0x") {
                                        i8::from_str_radix(hex, 16)
                                    } else if let Some(binary) = n.strip_prefix("0b") {
                                        i8::from_str_radix(binary, 2)
                                    } else {
                                        n.parse::<i8>()
                                    };

                                unsafe {
                                    if let Ok(i) = val {
                                        LLVMConstInt(LLVMInt8Type(), i as u64, 1)
                                    } else if let Err(err) = val {
                                        panic!("Err: {}", err);
                                    } else {
                                        panic!("{} is not a valid i32", n);
                                    }
                                }
                            }
                            TypeHint::I32(_) => {
                                let val: Result<i32, std::num::ParseIntError> =
                                    if let Some(hex) = n.strip_prefix("0x") {
                                        i32::from_str_radix(hex, 16)
                                    } else if let Some(binary) = n.strip_prefix("0b") {
                                        i32::from_str_radix(binary, 2)
                                    } else {
                                        n.parse::<i32>()
                                    };

                                unsafe {
                                    if let Ok(i) = val {
                                        LLVMConstInt(LLVMInt32Type(), i as u64, 1)
                                    } else if let Err(err) = val {
                                        panic!("Err: {}", err);
                                    } else {
                                        panic!("{} is not a valid i32", n);
                                    }
                                }
                            }
                            // unsafe {
                            //     if let Ok(i) = n.parse::<i32>() {
                            //         LLVMConstInt(LLVMInt32Type(), i as u64, 1)
                            //     } else {
                            //         panic!("invalid number literal: {}", n)
                            //     }
                            // },
                            TypeHint::U8(_) => {
                                let val: Result<u8, std::num::ParseIntError> =
                                    if let Some(hex) = n.strip_prefix("0x") {
                                        u8::from_str_radix(hex, 16)
                                    } else if let Some(binary) = n.strip_prefix("0b") {
                                        u8::from_str_radix(binary, 2)
                                    } else {
                                        n.parse::<u8>()
                                    };

                                unsafe {
                                    if let Ok(i) = val {
                                        LLVMConstInt(LLVMInt8Type(), i as u64, 1)
                                    } else if let Err(err) = val {
                                        panic!("Err: {}", err);
                                    } else {
                                        panic!("{} is not a valid u8", n);
                                    }
                                }
                            }
                            //     unsafe {
                            //         if let Ok(i) = n.parse::<u8>() {
                            //             LLVMConstInt(LLVMInt8Type(), i as u64, 1)
                            //         } else {
                            //             panic!("invalid number literal: {}", n)
                            //         }
                            //         // else if let Ok(f) = n.parse::<f64>() {
                            //         //     LLVMConstReal(LLVMDoubleType(), f)
                            //         // }
                            //     }
                            // }
                            TypeHint::U32(_) => {
                                let val: Result<u32, std::num::ParseIntError> =
                                    if let Some(hex) = n.strip_prefix("0x") {
                                        u32::from_str_radix(hex, 16)
                                    } else if let Some(binary) = n.strip_prefix("0b") {
                                        u32::from_str_radix(binary, 2)
                                    } else {
                                        n.parse::<u32>()
                                    };

                                unsafe {
                                    if let Ok(i) = val {
                                        LLVMConstInt(LLVMInt32Type(), i as u64, 1)
                                    } else if let Err(err) = val {
                                        panic!("Err: {}", err);
                                    } else {
                                        panic!("{} is not a valid u32", n);
                                    }
                                }
                            }
                            // unsafe {
                            //     if let Ok(i) = n.parse::<u32>() {
                            //         LLVMConstInt(LLVMInt32Type(), i as u64, 1)
                            //     } else {
                            //         panic!("invalid number literal: {}", n)
                            //     }
                            // }
                            TypeHint::F32(_) | TypeHint::F64(_) => {
                                // let val: Result<f64, std::num::ParseFloatError> =
                                //     if let Some(hex) = n.strip_prefix("0x") {
                                //         f64::from_str_radix(hex, 16)
                                //     } else if let Some(binary) = n.strip_prefix("0b") {
                                //         f64::from_str_radix(binary, 2)
                                //     } else {
                                //         n.parse::<f64>()
                                //     };

                                // unsafe {
                                //     if let Ok(i) = val {
                                //         LLVMConstInt(LLVMInt32Type(), i as u64, 1)
                                //     } else if let Err(err) = val {
                                //         panic!("Err: {}", err);
                                //     } else {
                                //         panic!("invalid floating number literal: {}", n);
                                //     }
                                // }
                                unsafe {
                                    if let Ok(f) = n.parse::<f64>() {
                                        LLVMConstReal(LLVMDoubleType(), f)
                                    } else {
                                        panic!("invalid floating number literal: {}", n)
                                    }
                                }
                            }
                            _ => self.gen_expr(&*expr),
                        }
                    }
                    _ => self.gen_expr(&*expr), // for now
                }
            }
            _ => todo!("expr variant not yet implemented: {}", expr.kind),
        }
    }

    // ── variables ────────────────────────────────────────────────────────────

    fn gen_var_decl(&mut self, name: &Ident, ty: &TypeSpec, value: &Expr) {
        unsafe {
            let llvm_type = self.gen_type(ty);
            let cname = self.ident_cstr(&name.ident);
            let alloca = LLVMBuildAlloca(self.builder, llvm_type, cname.as_ptr());
            let val = self.gen_expr(value);
            LLVMBuildStore(self.builder, val, alloca);
            self.declare(name.ident.clone(), Elem::Var(alloca));
        }
    }

    fn gen_ident(&mut self, ident: &Ident) -> LLVMValueRef {
        unsafe {
            match self.lookup(&ident.ident) {
                Some(Elem::Var(alloca)) => {
                    let alloca = *alloca;
                    let llvm_type = LLVMGetAllocatedType(alloca);
                    LLVMBuildLoad2(
                        self.builder,
                        llvm_type,
                        alloca,
                        self.ident_cstr(&ident.ident).as_ptr(),
                    )
                }
                Some(Elem::Global(global, ty)) => {
                    let (g, t) = (*global, *ty);
                    LLVMBuildLoad2(self.builder, t, g, self.ident_cstr(&ident.ident).as_ptr())
                }
                Some(Elem::Function(func, _)) => *func,
                Some(Elem::ExternFunc(func, _)) => *func,
                _ => panic!("undefined identifier: {}", ident.ident),
            }
        }
    }

    // ── function calls ───────────────────────────────────────────────────────

    fn gen_func_call(&mut self, name: &Ident, args: &[Expr]) -> LLVMValueRef {
        unsafe {
            let (func, fn_type) = self
                .lookup_function(&name.ident)
                .expect(&format!("undefined function: {}", name.ident));

            let mut llvm_args: Vec<LLVMValueRef> = args.iter().map(|a| self.gen_expr(a)).collect();

            let ret_type = LLVMGetReturnType(fn_type);
            let type_kind = LLVMGetTypeKind(ret_type);
            let call_name = if type_kind == LLVMVoidTypeKind {
                c"".as_ptr()
            } else {
                c"calltmp".as_ptr()
            };

            LLVMBuildCall2(
                self.builder,
                fn_type,
                func,
                llvm_args.as_mut_ptr(),
                llvm_args.len() as u32,
                call_name,
            )
        }
    }

    // ── extern functions ─────────────────────────────────────────────────────

    pub fn gen_extern_function(
        &mut self,
        name: &str,
        expect: &TypeSpec,
        params: &[InterfaceParam],
        is_variadic: bool,
    ) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            if let Some(cached) = self.lookup_function(name) {
                return cached;
            }

            let mut param_types: Vec<LLVMTypeRef> =
                params.iter().map(|p| self.gen_type(&p.ty)).collect();
            let ret_type = self.gen_type(expect);
            let fn_type = LLVMFunctionType(
                ret_type,
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                is_variadic as c_int,
            );

            let cname = self.ident_cstr(name);
            let func = LLVMAddFunction(self.module, cname.as_ptr(), fn_type);
            LLVMSetLinkage(func, LLVMExternalLinkage);

            self.declare_global(name.to_string(), Elem::ExternFunc(func, fn_type));
            (func, fn_type)
        }
    }

    // ── globals ───────────────────────────────────────────────────────────────

    pub fn _gen_global(&mut self, ty: &TypeSpec, name: &Ident, value: &Expr) -> LLVMValueRef {
        unsafe {
            let llvm_type = self.gen_type(ty);
            let cname = self.ident_cstr(&name.ident);
            let global = LLVMAddGlobal(self.module, llvm_type, cname.as_ptr());
            let init = self.gen_expr(value);
            LLVMSetInitializer(global, init);
            LLVMSetLinkage(global, LLVMInternalLinkage);
            self.declare_global(name.ident.clone(), Elem::Global(global, llvm_type));
            global
        }
    }

    // ── literals ─────────────────────────────────────────────────────────────

    // fn gen_number(&self, n: &str) -> LLVMValueRef {
    //     unsafe {
    //         let val: Result<i32, std::num::ParseIntError> = if let Some(hex) = n.strip_prefix("0x")
    //         {
    //             i32::from_str_radix(hex, 16)
    //         } else if let Some(binary) = n.strip_prefix("0b") {
    //             i32::from_str_radix(binary, 2)
    //         } else {
    //             n.parse::<i32>()
    //         };

    //         if let Ok(i) = val {
    //             LLVMConstInt(LLVMInt32Type(), i as u64, 1)
    //         } else if let Err(err) = val {
    //             panic!("Err: {}", err);
    //         } else {
    //             panic!("{} is not a valid i32", n);
    //         }

    //         // if let Ok(i) = n.parse::<i32>() {
    //         //     LLVMConstInt(LLVMInt32Type(), i as u64, 1)
    //         // } else if let Ok(f) = n.parse::<f64>() {
    //         //     LLVMConstReal(LLVMDoubleType(), f)
    //         // } else {
    //         //     panic!("invalid number literal: {}", n)
    //         // }
    //     }
    // }

    fn gen_string(&mut self, s: &str) -> LLVMValueRef {
        if let Some(&ptr) = self.strings.get(s) {
            return ptr;
        }
        unsafe {
            let cstr = std::ffi::CString::new(s).unwrap();
            let ptr = LLVMBuildGlobalStringPtr(self.builder, cstr.as_ptr(), c"str".as_ptr());
            self.strings.insert(s.to_string(), ptr);
            ptr
        }
    }

    fn gen_bool(&self, tv: &TruthVal) -> LLVMValueRef {
        unsafe {
            let val = match tv {
                TruthVal::True => 1u64,
                TruthVal::False => 0u64,
                TruthVal::Unknown => 0u64, // treat None as false for i1
                TruthVal::Both => 1u64,
            };
            LLVMConstInt(LLVMInt1Type(), val, 0)
        }
    }

    fn gen_char(&self, c: char) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt32Type(), c as u64, 0) }
    }

    // ── types ────────────────────────────────────────────────────────────────

    fn gen_type(&self, ty: &TypeSpec) -> LLVMTypeRef {
        unsafe {
            match &ty.hint {
                TypeHint::I8(_) | TypeHint::U8(_) => LLVMInt8Type(),
                TypeHint::I16(_) | TypeHint::U16(_) => LLVMInt16Type(),
                TypeHint::I32(_) | TypeHint::U32(_) => LLVMInt32Type(),
                TypeHint::I64(_) | TypeHint::U64(_) => LLVMInt64Type(),
                TypeHint::F32(_) => LLVMFloatType(),
                TypeHint::F64(_) => LLVMDoubleType(),
                TypeHint::Bool => LLVMInt1Type(),
                TypeHint::Void => LLVMVoidType(),
                TypeHint::Char => LLVMInt32Type(),
                TypeHint::Str => LLVMPointerType(LLVMInt8Type(), 0),
                TypeHint::RawPtr { tysp, .. } => {
                    let inner = self.gen_type(tysp);
                    LLVMPointerType(inner, 0)
                }
                TypeHint::Custom { ident, .. } => match self.lookup(&ident.ident) {
                    Some(Elem::Struct { ty, .. }) => *ty,
                    Some(Elem::Enum { ty, .. }) => *ty,
                    Some(Elem::Type(ty)) => *ty,
                    _ => panic!("unknown type: {}", ident.ident),
                },
                _ => todo!("type not yet implemented: {:?}", ty.hint),
            }
        }
    }

    // ── binary ops ───────────────────────────────────────────────────────────

    fn gen_binary(&mut self, lhs: &Expr, op: &FullTok, rhs: &Expr) -> LLVMValueRef {
        unsafe {
            let l = self.gen_expr(lhs);
            let r = self.gen_expr(rhs);
            match op.kind {
                Token::Plus => LLVMBuildAdd(self.builder, l, r, c"addtmp".as_ptr()),
                Token::Minus => LLVMBuildSub(self.builder, l, r, c"subtmp".as_ptr()),
                Token::Star => LLVMBuildMul(self.builder, l, r, c"multmp".as_ptr()),
                Token::Slash => LLVMBuildSDiv(self.builder, l, r, c"divtmp".as_ptr()),
                Token::Equal => LLVMBuildICmp(self.builder, LLVMIntEQ, l, r, c"cmptmp".as_ptr()),
                Token::NotEqual => LLVMBuildICmp(self.builder, LLVMIntNE, l, r, c"cmptmp".as_ptr()),
                Token::Less => LLVMBuildICmp(self.builder, LLVMIntSLT, l, r, c"cmptmp".as_ptr()),
                Token::LessEq => LLVMBuildICmp(self.builder, LLVMIntSLE, l, r, c"cmptmp".as_ptr()),
                Token::Greater => LLVMBuildICmp(self.builder, LLVMIntSGT, l, r, c"cmptmp".as_ptr()),
                Token::GreaterEq => {
                    LLVMBuildICmp(self.builder, LLVMIntSGE, l, r, c"cmptmp".as_ptr())
                }
                _ => todo!("op not yet implemented: {:?}", op.kind),
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Hoisting passes (type names → type bodies → function signatures)
    // ═════════════════════════════════════════════════════════════════════════

    pub fn hoist_type_names(&mut self, types: &HashMap<String, CustTypeInfo>) {
        for (name, info) in types {
            match info {
                CustTypeInfo::StructDeclInfo { .. } => unsafe {
                    let cname = self.cstring(name);
                    let ty = LLVMStructCreateNamed(LLVMGetGlobalContext(), cname.as_ptr());
                    self.declare_global(
                        name.clone(),
                        Elem::Struct {
                            ty,
                            fields: HashMap::new(),
                        },
                    );
                },
                CustTypeInfo::EnumDeclInfo { .. } => unsafe {
                    self.declare_global(
                        name.clone(),
                        Elem::Enum {
                            ty: LLVMInt32Type(),
                            variants: HashMap::new(),
                        },
                    );
                },
            }
        }
    }

    pub fn declare_foreign_struct_name(&mut self, name: &str) {
        unsafe {
            // check already declared
            if self.lookup(name).is_some() {
                return;
            }

            // create opaque named struct with SAME name as the defining module
            // LLVMLinkModules2 will fill in the body from the defining module
            let cname = self.cstring(name);
            let ty = LLVMStructCreateNamed(LLVMGetGlobalContext(), cname.as_ptr());
            // deliberately NO LLVMStructSetBody call — stays opaque

            self.declare_global(
                name.to_string(),
                Elem::Struct {
                    ty,
                    fields: HashMap::new(), // fields unknown — codegen can't use them directly
                },
            );
        }
    }

    pub fn hoist_type_bodies(&mut self, types: &HashMap<String, CustTypeInfo>) {
        for (name, info) in types {
            match info {
                CustTypeInfo::StructDeclInfo { vals, .. } => {
                    let mut field_types: Vec<LLVMTypeRef> =
                        vals.iter().map(|f| self.gen_type(&f.par.ty)).collect();
                    let mut field_map = HashMap::new();
                    for (i, field) in vals.iter().enumerate() {
                        field_map.insert(field.par.ident.ident.clone(), i as u32);
                    }
                    unsafe {
                        if let Some(Elem::Struct { ty, .. }) = self.lookup(name) {
                            let ty = *ty;
                            LLVMStructSetBody(
                                ty,
                                field_types.as_mut_ptr(),
                                field_types.len() as u32,
                                0,
                            );
                        }
                        let ty = self.lookup_struct(name).unwrap().0;
                        self.declare_global(
                            name.clone(),
                            Elem::Struct {
                                ty,
                                fields: field_map,
                            },
                        );
                    }
                }
                CustTypeInfo::EnumDeclInfo { vals, .. } => {
                    let mut variants = HashMap::new();
                    for (i, variant) in vals.iter().enumerate() {
                        let ident = match variant {
                            EnumVariants::Record { name, .. } => name.ident.clone(),
                            EnumVariants::Simple(name) => name.ident.clone(),
                            EnumVariants::Tuple { name, .. } => name.ident.clone(),
                        };
                        variants.insert(ident, i as u64);
                    }
                    self.declare_global(
                        name.clone(),
                        Elem::Enum {
                            ty: unsafe { LLVMInt32Type() },
                            variants,
                        },
                    );
                }
            }
        }
    }

    pub fn hoist_functions(&mut self, funcs: &HashMap<String, FuncDeclInfo>) {
        for (name, info) in funcs {
            // skip extern hoisting if function has a body
            // it will be handled by gen_function directly
            if info.is_extern.is_some() && matches!(info.block, FuncExpr::None) {
                // truly extern — no body — declare as external
                self.gen_extern_function(name, &info.expect, &info.par, false);
            } else if info.is_extern.is_none() {
                // regular function — just hoist the signature
                // gen_function will check LLVMGetNamedFunction before creating
                unsafe {
                    let mut param_types: Vec<LLVMTypeRef> =
                        info.par.iter().map(|p| self.gen_type(&p.ty)).collect();
                    let ret_type = self.gen_type(&info.expect);
                    let fn_type = LLVMFunctionType(
                        ret_type,
                        param_types.as_mut_ptr(),
                        param_types.len() as u32,
                        0,
                    );
                    let cname = self.cstring(name);
                    let func = LLVMAddFunction(self.module, cname.as_ptr(), fn_type);
                    if info.is_public {
                        LLVMSetLinkage(func, LLVMExternalLinkage);
                    } else {
                        LLVMSetLinkage(func, LLVMInternalLinkage);
                    }
                    self.declare_global(name.clone(), Elem::Function(func, fn_type));
                }
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Utilities
    // ═════════════════════════════════════════════════════════════════════════

    pub fn _dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }

    pub fn merge_all_modules(
        mut modules: HashMap<String, Codegen>,
        entry: String,
    ) -> LLVMModuleRef {
        let merged = {
            // remove main's module — use it as destination
            let main_cg = modules.remove(&entry);

            if main_cg.is_some() {
                let dest = main_cg.unwrap().module;

                // merge all other modules into main's module
                for (_, cg) in modules {
                    unsafe {
                        let result = LLVMLinkModules2(dest, cg.module);
                        if result != 0 {
                            eprintln!("module merge failed");
                            std::process::exit(1);
                        }
                    }
                }
                dest
            } else {
                panic!("entry module not found: {}", entry);
            }
        };

        merged
    }

    // fn find_msvc_libs() -> Vec<String> {
    //     let mut paths = vec![];

    //     // find Windows SDK
    //     let sdk_base = r"C:\Program Files (x86)\Windows Kits\10\Lib";
    //     if let Ok(entries) = std::fs::read_dir(sdk_base) {
    //         if let Some(entry) = entries.flatten().last() {
    //             let um = entry.path().join("um\\x64");
    //             let ucrt = entry.path().join("ucrt\\x64");
    //             if um.exists() {
    //                 paths.push(format!("/LIBPATH:{}", um.display()));
    //             }
    //             if ucrt.exists() {
    //                 paths.push(format!("/LIBPATH:{}", ucrt.display()));
    //             }
    //         }
    //     }

    //     // find MSVC
    //     let msvc_base = r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC";
    //     if let Ok(entries) = std::fs::read_dir(msvc_base) {
    //         if let Some(entry) = entries.flatten().last() {
    //             let lib = entry.path().join("lib\\x64");
    //             if lib.exists() {
    //                 paths.push(format!("/LIBPATH:{}", lib.display()));
    //             }
    //         }
    //     }

    //     paths
    // }

    pub fn link_windows(
        obj_path: &str,
        out_path: &str,
        exe_dir: PathBuf,
        cli: &ComdInfo,
    ) -> Output {
        if let Ok(metadata) = std::fs::metadata(&obj_path) {
            if metadata.len() == 0 {
                eprintln!("❌ Object file is empty: {}", obj_path);
                eprintln!("   No code was generated!");
                std::process::exit(1);
            }
            println!("📦 Object file size: {} bytes", metadata.len());
        }

        print!("\rLinking.  ");
        io::stdout().flush().unwrap();

        // let target_str = cli.target.as_deref().unwrap_or("x86_64-pc-windows-gnu");

        let lld = exe_dir.join("lib\\rust-lld.exe");
        let lld = if lld.exists() {
            lld
        } else {
            std::path::PathBuf::from(
                r"C:\Users\HP\.rustup\toolchains\nightly-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe",
            )
        };

        let lib_dir = exe_dir.join("lib\\system\\windows");
        let lib_dir = if lib_dir.exists() {
            lib_dir
        } else {
            std::path::PathBuf::from(
                r"C:\Users\HP\.rustup\toolchains\nightly-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained",
            )
        };

        // let (crt_obj, subsystem, entry) = if is_gui {
        //     (lib_dir.join("crtbegin.o"), "windows", "WinMainCRTStartup")
        // } else {
        //     (lib_dir.join("crt2.o"), "console", "mainCRTStartup")
        // };

        print!("\rLinking.. ");
        io::stdout().flush().unwrap();
        // thread::sleep(Duration::from_millis(5));

        // rust-lld has non-deterministic startup object selection
        let mut args = vec![
            //
            "-flavor", "gnu", //
            "-m", "i386pep", //
            "-o", out_path, //
            obj_path,
        ];

        if cli.win {
            // GUI CRT
            args.extend_from_slice(&[
                // "crt2.o",
                // use WinMain entry
                "-e",
                "WinMainCRTStartup",
                "--subsystem",
                "windows",
            ]);
        } else {
            // Console CRT
            args.extend_from_slice(&[
                // "crt2.o",
                // Use main entry
                "-e",
                "mainCRTStartup",
                "--subsystem",
                "console",
            ]);
        }

        // Library search paths
        let lib_dir_display = format!("-L{}", lib_dir.display());

        // Libraries
        args.extend_from_slice(&[
            &lib_dir_display,
            "-lkui_runtime",
            "-lgcc",
            "-lgcc_eh",
            "-lmingw32",
            "-lmsvcrt",
            // "-lgcc_s",
            "-lmingwex",
            "-lmoldname",
            "-lkernel32",
            "-luser32",
            "-ladvapi32",
            "-lshell32",
        ]);

        print!("\rLinking...");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(5));

        std::process::Command::new(&lld)
            .args(&args)
            .output()
            .expect("rust-lld not found")
    }
}
