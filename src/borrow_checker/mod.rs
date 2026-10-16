use std::collections::HashMap;

use crate::cfg_builder::CFGBuilder;
use crate::cfg_resourses::{BorrowError, Terminator, CFG};
use crate::compiler::{Mod, NameSpace, SemanticError};

use crate::parser::{
    AssFunc, BlockLevel, BridgeDecl, CustTyAss, CustomBlock, CustomLevel, DataState, EnumVariants, Expr, ExprKind, Generic, Ident, InterfaceParam, Method, MethodSelf, ModPath, PlatformArm, StructParam, TopLevel, Trinary, TypeHint, TypeSpec,
};
use crate::scanner::{FullTok, Span, Token};

use crate::analyser::Analyser;
//===================================================
// Generative code
//===================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Resource {
    File(ModCFG),
    Folder(Vec<ModCFG>),
    Package {
        entry: ModCFG,
        src: HashMap<String, ModCFG>,
    },
}

// In this HIR each element type as own custom type unlike having a single
// like TopLevel

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TakeStmtInfo {
    pub take: (FullTok, Option<Ident>), // "as" as an option
    pub from: ModPath,
    pub lib: Option<Ident>,
    pub stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GlobalDeclInfo {
    ident_span: Span,
    interpret: bool,
    state: DataState,
    ty: TypeSpec,
    value: CFG,
    public: bool,
    stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TraitDefInfo {
    ident_span: Span,
    generic_types: Option<Ident>,
    interprets: Vec<BlockLevel>,
    methods: Vec<String>,
    is_public: bool,
    stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformArmInfo {
    pub target: Expr,
    pub block: ModCFG,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformNSInfo {
    pub arms: Vec<PlatformArmInfo>,
    pub stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkerDefInfo {
    func_name: Ident,
    public: bool,
    block: CFG,
    stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FuncExpr {
    Cfg(CFG),
    None,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FuncDeclInfo {
    pub is_async: bool,
    pub ident_span: Span,
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub generics: Option<Vec<Generic>>,
    pub par: Vec<InterfaceParam>,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub is_public: bool,
    pub is_visible: bool,
    pub worker_usage: Vec<Ident>,
    pub block: FuncExpr,
    pub stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StaticDefInfo {
    ident_span: Span,
    public: bool,
    block: ModCFG,
    stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MethodCfg {
    ident_span: Span,
    worker_usage: Vec<Ident>,
    meth_self: (MethodSelf, Span),
    expect: TypeSpec,
    ret_state: DataState,
    generics: Option<Vec<Generic>>,
    is_extern: Option<(Ident, Option<Span>)>,
    is_public: Trinary,
    par: Vec<InterfaceParam>,
    block: Option<CFG>,
    stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StaticFunc {
    pub ident_span: Span,
    pub worker_usage: Vec<Ident>,
    //also known as AssFunc in the parser
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub generics: Vec<Generic>,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub is_public: bool,
    pub par: Vec<InterfaceParam>,
    pub block: Option<CFG>,
    pub stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BridgeCfg {
    ass_state: DataState,
    ass_type: TypeSpec,
    ass_ident: Ident,
    new_ty: Ident,
    pub is_public: bool,
    pub block: CFG,
    pub stm_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CustTypeInfo {
    EnumDeclInfo {
        ident_span: Span,
        is_public: bool,
        generic_types: Vec<Generic>,
        vals: Vec<EnumVariants>,
        traits: Vec<Ident>,
        ass_func_strs: Vec<Ident>,
        meth_strs: Vec<Ident>,
        stm_span: Span,
    },
    StructDeclInfo {
        ident_span: Span,
        is_public: bool,
        generic_types: Vec<Generic>,
        vals: Vec<StructParam>,
        traits: Vec<Ident>,
        ass_func_strs: Vec<Ident>,
        meth_strs: Vec<Ident>,
        stm_span: Span,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModCFG {
    pub globals: HashMap<String, GlobalDeclInfo>,
    pub platform_n_s: Vec<PlatformNSInfo>,
    pub takes: HashMap<String, TakeStmtInfo>,
    pub traits: HashMap<String, TraitDefInfo>,
    pub workers: HashMap<String, WorkerDefInfo>,
    pub statics: HashMap<String, StaticDefInfo>,
    pub custom_tys: HashMap<String, CustTypeInfo>,
    pub methods: HashMap<String, HashMap<String, MethodCfg>>,
    pub static_meths: HashMap<String, HashMap<String, StaticFunc>>,
    pub bridges: HashMap<String, Vec<BridgeCfg>>,
    pub main_fn: Option<FuncDeclInfo>,
    pub funcs: HashMap<String, FuncDeclInfo>,
}

impl ModCFG {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            platform_n_s: Vec::new(),
            takes: HashMap::new(),
            traits: HashMap::new(),
            workers: HashMap::new(),
            statics: HashMap::new(),
            custom_tys: HashMap::new(),
            methods: HashMap::new(),
            static_meths: HashMap::new(),
            bridges: HashMap::new(),
            main_fn: None,
            funcs: HashMap::new(),
        }
    }
}

fn add_errors(errors: Vec<BorrowError>, analyser: &mut Analyser) {
    for err in errors {
        match err {
            BorrowError::BorrowOutlivesLender {
                lender,
                borrower,
                loan_id: _,
            } => {
                analyser.errs.push(SemanticError {
                    messages: vec![format!(
                        "{} does not live long enough; it was outlived by {}",
                        lender.ident, borrower.ident
                    )],
                    hints: vec![],
                    span: borrower.span,
                });
            }
            _ => {
                panic!("{:#?}", err);
            }
        }
    }
}

fn build_block(
    ident: String,
    ret_state: DataState,
    expect: TypeSpec,
    block: Expr,
    modl: String,
) -> CFG {
    let mut cfg_builder = CFGBuilder::new(ident.as_str(), modl);
    // cfg_builder.build_function(&ident);
    let tmp = cfg_builder.build_expr(ret_state, expect.clone(), &block);

    // Handle termination

    match &block {
        Expr {
            kind: ExprKind::Block(_),
            span: _,
        } => {
            if !cfg_builder.cfg.block(cfg_builder.current).is_terminated() {
                cfg_builder.terminate(Terminator::Return(None));
            }
        }
        _ => {
            if tmp.is_some() {
                let tmp = tmp.unwrap();
                cfg_builder.terminate(Terminator::Return(Some(Expr {
                    span: tmp.span.clone(),
                    kind: ExprKind::Ident {
                        affix: None,
                        ident: tmp,
                    },
                })));
            } else {
                panic!("Shouldn't happen");
            }
        }
    }

    cfg_builder.cfg
}

fn for_each_take(takes: &Vec<TopLevel>, mod_cfg: &mut ModCFG) {
    for take in takes {
        match take.clone() {
            TopLevel::TakeStmt {
                take,
                from,
                lib,
                info,
            } => match take.0.kind.clone() {
                Token::Identifier(ident) => {
                    mod_cfg.takes.insert(
                        ident,
                        TakeStmtInfo {
                            take,
                            from,
                            lib,
                            stm_span: info.span,
                        },
                    );
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn for_each_func(
    mod_name: &String,
    fns: &Vec<TopLevel>,
    mod_cfg: &mut ModCFG,
    analyser: &mut Analyser,
) {
    for func in fns {
        match func.clone() {
            TopLevel::FuncDecl {
                is_async,
                expect,
                ret_state,
                ident,
                generics,
                is_visible,
                is_public,
                is_extern,
                par,
                worker_usage,
                block,
                info,
            } => {
                if is_extern.is_none() {
                    let mut cfg = build_block(
                        ident.ident.clone(),
                        ret_state.clone(),
                        expect.clone(),
                        block,
                        mod_name.clone(),
                    );

                    cfg.compute_liveness();

                    // refine loan expiry points now that liveness is known
                    cfg.finalize_loan_lifetimes(); //

                    // run checks in dependency order
                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                    // collect all errors
                    let all_errors: Vec<BorrowError> = move_errors
                        .into_iter()
                        .chain(borrow_errors)
                        .chain(lifetime_errs)
                        .collect();

                    add_errors(all_errors, analyser);

                    mod_cfg.funcs.insert(
                        ident.ident.clone(),
                        FuncDeclInfo {
                            ident_span: ident.span,
                            is_async,
                            expect,
                            ret_state,
                            generics,
                            par,
                            is_extern: None,
                            is_public,
                            is_visible,
                            worker_usage,
                            block: FuncExpr::Cfg(cfg),
                            stm_span: info.span,
                        },
                    );
                } else {
                    mod_cfg.funcs.insert(
                        ident.ident.clone(),
                        FuncDeclInfo {
                            ident_span: ident.span,
                            is_async,
                            expect,
                            ret_state,
                            generics,
                            par,
                            is_extern,
                            is_public,
                            is_visible,
                            worker_usage,
                            block: FuncExpr::None,
                            stm_span: info.span,
                        },
                    );
                }
            }
            _ => (),
        }
    }
}

fn for_each_cust_ty(mod_name: &String, custom_tys: &Vec<TopLevel>, mod_cfg: &mut ModCFG, analyser: &mut Analyser) {
    for each in custom_tys {
        match each.clone() {
            TopLevel::EnumDecl {
                ident,
                generic_types,
                vals,
                traits,
                public: is_enum_pub,
                associateds,
                bridges,
                info,
            } => {
                let mut methods = HashMap::new();
                let mut ass_functions = HashMap::new();
                let mut ass_func_strs = vec![];
                let mut meth_strs = vec![];

                for ass in associateds {
                    match ass {
                        CustTyAss::AssFunction(func) => match func.clone() {
                            AssFunc {
                                worker_usage,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                ass_func_strs.push(ident);
                            }
                        },
                        CustTyAss::Method(meth) => match meth.clone() {
                            Method {
                                worker_usage,
                                meth_self,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                meth_strs.push(ident);
                            }
                        },
                    }
                }

                let mut bgs = vec![];
                for each in &bridges {
                    match each.clone() {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let mut cfg = build_block(
                                ident.ident.clone(),
                                DataState::Owner,
                                TypeSpec {
                                    hint: TypeHint::Custom {
                                        import: None,
                                        ident: new_ty.clone(),
                                        generics: vec![],
                                    },
                                    span: new_ty.span.clone(),
                                },
                                block.clone(),
                                mod_name.clone(),
                            );

                            cfg.compute_liveness();

                            // refine loan expiry points now that liveness is known
                            cfg.finalize_loan_lifetimes(); //

                            // run checks in dependency order
                            let move_errors = cfg.check_use_after_move(); // needs: nothing
                            let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                            let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                            // collect all errors
                            let all_errors: Vec<BorrowError> = move_errors
                                .into_iter()
                                .chain(borrow_errors)
                                .chain(lifetime_errs)
                                .collect();

                            add_errors(all_errors, analyser);

                            bgs.push(BridgeCfg {
                                ass_ident,
                                ass_state,
                                ass_type,
                                new_ty,
                                is_public: public,
                                block: cfg,
                                stm_span: info.span,
                            });
                        }
                    }
                }

                mod_cfg.custom_tys.insert(
                    ident.ident.clone(),
                    CustTypeInfo::EnumDeclInfo {
                        ident_span: ident.span,
                        is_public: is_enum_pub,
                        generic_types,
                        vals,
                        traits,
                        ass_func_strs,
                        meth_strs,
                        stm_span: info.span,
                    },
                );
                mod_cfg.methods.insert(ident.ident.clone(), methods);
                mod_cfg
                    .static_meths
                    .insert(ident.ident.clone(), ass_functions);
                mod_cfg.bridges.insert(ident.ident.clone(), bgs);
            }
            TopLevel::StructDecl {
                ident,
                generic_types,
                vals,
                traits,
                public: is_struct_pub,
                associateds,
                bridges,
                info,
            } => {
                let mut methods = HashMap::new();
                let mut ass_functions = HashMap::new();
                let mut ass_func_strs = vec![];
                let mut meth_strs = vec![];
                let mut brg_strs = vec![];

                for ass in associateds {
                    match ass {
                        CustTyAss::AssFunction(func) => match func.clone() {
                            AssFunc {
                                worker_usage,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                ass_func_strs.push(ident);
                            }
                        },
                        CustTyAss::Method(meth) => match meth.clone() {
                            Method {
                                worker_usage,
                                meth_self,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                meth_strs.push(ident);
                            }
                        },
                    }
                }

                let mut bgs = vec![];
                for each in &bridges {
                    match each.clone() {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let mut cfg = build_block(
                                ident.ident.clone(),
                                DataState::Owner,
                                TypeSpec {
                                    hint: TypeHint::Custom {
                                        import: None,
                                        ident: new_ty.clone(),
                                        generics: vec![],
                                    },
                                    span: new_ty.span.clone(),
                                },
                                block.clone(),
                                mod_name.clone(),
                            );

                            cfg.compute_liveness();

                            // refine loan expiry points now that liveness is known
                            cfg.finalize_loan_lifetimes(); //

                            // run checks in dependency order
                            let move_errors = cfg.check_use_after_move(); // needs: nothing
                            let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                            let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                            // collect all errors
                            let all_errors: Vec<BorrowError> = move_errors
                                .into_iter()
                                .chain(borrow_errors)
                                .chain(lifetime_errs)
                                .collect();

                            add_errors(all_errors, analyser);

                            bgs.push(BridgeCfg {
                                ass_ident,
                                ass_state,
                                ass_type,
                                new_ty,
                                is_public: public,
                                block: cfg,
                                stm_span: info.span,
                            });
                            brg_strs.push(ident.clone());
                        }
                    }
                }

                mod_cfg.custom_tys.insert(
                    ident.ident.clone(),
                    CustTypeInfo::StructDeclInfo {
                        ident_span: ident.span,
                        is_public: is_struct_pub,
                        generic_types,
                        vals,
                        traits,
                        ass_func_strs,
                        meth_strs,
                        stm_span: info.span,
                    },
                );
                mod_cfg.methods.insert(ident.ident.clone(), methods);
                mod_cfg
                    .static_meths
                    .insert(ident.ident.clone(), ass_functions);
                mod_cfg.bridges.insert(ident.ident.clone(), bgs);
            }
            _ => (),
        }
    }
}

fn for_each_cust_block_take(takes: &Vec<CustomLevel>, mod_cfg: &mut ModCFG) {
    for take in takes {
        match take.clone() {
            CustomLevel::TakeStmt {
                take,
                from,
                lib,
                info,
            } => match take.0.kind.clone() {
                Token::Identifier(ident) => {
                    mod_cfg.takes.insert(
                        ident,
                        TakeStmtInfo {
                            take,
                            from,
                            lib,
                            stm_span: info.span,
                        },
                    );
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn for_each_cust_block_func(
    mod_name: &String,
    fns: &Vec<CustomLevel>,
    mod_cfg: &mut ModCFG,
    analyser: &mut Analyser,
) {
    for func in fns {
        match func.clone() {
            CustomLevel::FuncDecl {
                is_async,
                expect,
                ret_state,
                ident,
                generics,
                is_visible,
                is_public,
                is_extern,
                par,
                worker_usage,
                block,
                info,
            } => {
                if is_extern.is_none() {
                    let mut cfg = build_block(
                        ident.ident.clone(),
                        ret_state.clone(),
                        expect.clone(),
                        block,
                        mod_name.clone(),
                    );

                    cfg.compute_liveness();

                    // refine loan expiry points now that liveness is known
                    cfg.finalize_loan_lifetimes(); //

                    // run checks in dependency order
                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                    // collect all errors
                    let all_errors: Vec<BorrowError> = move_errors
                        .into_iter()
                        .chain(borrow_errors)
                        .chain(lifetime_errs)
                        .collect();

                    add_errors(all_errors, analyser);

                    mod_cfg.funcs.insert(
                        ident.ident.clone(),
                        FuncDeclInfo {
                            ident_span: ident.span,
                            is_async,
                            expect,
                            ret_state,
                            generics,
                            par,
                            is_extern: None,
                            is_public,
                            is_visible,
                            worker_usage,
                            block: FuncExpr::Cfg(cfg),
                            stm_span: info.span,
                        },
                    );
                } else {
                    mod_cfg.funcs.insert(
                        ident.ident.clone(),
                        FuncDeclInfo {
                            ident_span: ident.span,
                            is_async,
                            expect,
                            ret_state,
                            generics,
                            par,
                            is_extern,
                            is_public,
                            is_visible,
                            worker_usage,
                            block: FuncExpr::None,
                            stm_span: info.span,
                        },
                    );
                }
            }
            _ => (),
        }
    }
}

fn for_each_cust_block_cust_ty(mod_name: &String, custom_tys: &Vec<CustomLevel>, mod_cfg: &mut ModCFG, analyser: &mut Analyser) {
    for each in custom_tys {
        match each.clone() {
            CustomLevel::EnumDecl {
                ident,
                generic_types,
                vals,
                traits,
                public: is_enum_pub,
                associateds,
                bridges,
                info,
            } => {
                let mut methods = HashMap::new();
                let mut ass_functions = HashMap::new();
                let mut ass_func_strs = vec![];
                let mut meth_strs = vec![];

                for ass in associateds {
                    match ass {
                        CustTyAss::AssFunction(func) => match func.clone() {
                            AssFunc {
                                worker_usage,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                ass_func_strs.push(ident);
                            }
                        },
                        CustTyAss::Method(meth) => match meth.clone() {
                            Method {
                                worker_usage,
                                meth_self,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                meth_strs.push(ident);
                            }
                        },
                    }
                }

                let mut bgs = vec![];
                for each in &bridges {
                    match each.clone() {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let mut cfg = build_block(
                                ident.ident.clone(),
                                DataState::Owner,
                                TypeSpec {
                                    hint: TypeHint::Custom {
                                        import: None,
                                        ident: new_ty.clone(),
                                        generics: vec![],
                                    },
                                    span: new_ty.span.clone(),
                                },
                                block.clone(),
                                mod_name.clone(),
                            );

                            cfg.compute_liveness();

                            // refine loan expiry points now that liveness is known
                            cfg.finalize_loan_lifetimes(); //

                            // run checks in dependency order
                            let move_errors = cfg.check_use_after_move(); // needs: nothing
                            let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                            let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                            // collect all errors
                            let all_errors: Vec<BorrowError> = move_errors
                                .into_iter()
                                .chain(borrow_errors)
                                .chain(lifetime_errs)
                                .collect();

                            add_errors(all_errors, analyser);

                            bgs.push(BridgeCfg {
                                ass_ident,
                                ass_state,
                                ass_type,
                                new_ty,
                                is_public: public,
                                block: cfg,
                                stm_span: info.span,
                            });
                        }
                    }
                }

                mod_cfg.custom_tys.insert(
                    ident.ident.clone(),
                    CustTypeInfo::EnumDeclInfo {
                        ident_span: ident.span,
                        is_public: is_enum_pub,
                        generic_types,
                        vals,
                        traits,
                        ass_func_strs,
                        meth_strs,
                        stm_span: info.span,
                    },
                );
                mod_cfg.methods.insert(ident.ident.clone(), methods);
                mod_cfg
                    .static_meths
                    .insert(ident.ident.clone(), ass_functions);
                mod_cfg.bridges.insert(ident.ident.clone(), bgs);
            }
            CustomLevel::StructDecl {
                ident,
                generic_types,
                vals,
                traits,
                public: is_struct_pub,
                associateds,
                bridges,
                info,
            } => {
                let mut methods = HashMap::new();
                let mut ass_functions = HashMap::new();
                let mut ass_func_strs = vec![];
                let mut meth_strs = vec![];
                let mut brg_strs = vec![];

                for ass in associateds {
                    match ass {
                        CustTyAss::AssFunction(func) => match func.clone() {
                            AssFunc {
                                worker_usage,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    ass_functions.insert(
                                        ident.ident.clone(),
                                        StaticFunc {
                                            ident_span: ident.span.clone(),
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                ass_func_strs.push(ident);
                            }
                        },
                        CustTyAss::Method(meth) => match meth.clone() {
                            Method {
                                worker_usage,
                                meth_self,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block,
                                info,
                            } => {
                                if block.is_some() {
                                    let block = block.unwrap();
                                    let mut cfg = build_block(
                                        ident.ident.clone(),
                                        ret_state.clone(),
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: Some(cfg),
                                            stm_span: info.span,
                                        },
                                    );
                                } else {
                                    methods.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span.clone(),
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            par,
                                            is_extern,
                                            is_public,
                                            worker_usage,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }

                                meth_strs.push(ident);
                            }
                        },
                    }
                }

                let mut bgs = vec![];
                for each in &bridges {
                    match each.clone() {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let mut cfg = build_block(
                                ident.ident.clone(),
                                DataState::Owner,
                                TypeSpec {
                                    hint: TypeHint::Custom {
                                        import: None,
                                        ident: new_ty.clone(),
                                        generics: vec![],
                                    },
                                    span: new_ty.span.clone(),
                                },
                                block.clone(),
                                mod_name.clone(),
                            );

                            cfg.compute_liveness();

                            // refine loan expiry points now that liveness is known
                            cfg.finalize_loan_lifetimes(); //

                            // run checks in dependency order
                            let move_errors = cfg.check_use_after_move(); // needs: nothing
                            let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                            let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                            // collect all errors
                            let all_errors: Vec<BorrowError> = move_errors
                                .into_iter()
                                .chain(borrow_errors)
                                .chain(lifetime_errs)
                                .collect();

                            add_errors(all_errors, analyser);

                            bgs.push(BridgeCfg {
                                ass_ident,
                                ass_state,
                                ass_type,
                                new_ty,
                                is_public: public,
                                block: cfg,
                                stm_span: info.span,
                            });
                            brg_strs.push(ident.clone());
                        }
                    }
                }

                mod_cfg.custom_tys.insert(
                    ident.ident.clone(),
                    CustTypeInfo::StructDeclInfo {
                        ident_span: ident.span,
                        is_public: is_struct_pub,
                        generic_types,
                        vals,
                        traits,
                        ass_func_strs,
                        meth_strs,
                        stm_span: info.span,
                    },
                );
                mod_cfg.methods.insert(ident.ident.clone(), methods);
                mod_cfg
                    .static_meths
                    .insert(ident.ident.clone(), ass_functions);
                mod_cfg.bridges.insert(ident.ident.clone(), bgs);
            }
            _ => (),
        }
    }
}

pub fn gen_unit_cfg(analyser: &mut Analyser) -> HashMap<String, Resource> {
    let mut ret = HashMap::new();

    for each in analyser.unit.clone() {
        match each.1 {
            NameSpace::Singular {
                name: mod_name,
                file,
            } => {
                match &file {
                    Mod {
                        mod_docs: _,
                        main_fn,
                        macro_defs: _,
                        fns,
                        platform_n_s,
                        takes,
                        globals: _,
                        traits: _,
                        workers: _,
                        statics: _,
                        custom_tys,
                        lines: _,
                        scope: _,
                    } => {
                        let mut mod_cfg = ModCFG {
                            globals: HashMap::new(),
                            platform_n_s: Vec::new(),
                            takes: HashMap::new(),
                            traits: HashMap::new(),
                            workers: HashMap::new(),
                            statics: HashMap::new(),
                            custom_tys: HashMap::new(),
                            main_fn: None,
                            funcs: HashMap::new(),
                            methods: HashMap::new(),
                            static_meths: HashMap::new(),
                            bridges: HashMap::new(),
                        };

                        // FOR TAKES
                        for_each_take(takes, &mut mod_cfg);

                        // FOR MAIN FUNCTION
                        if main_fn.is_some() {
                            let name = String::from("<main>");

                            match main_fn.clone().unwrap() {
                                TopLevel::Entry {
                                    ident,
                                    expect,
                                    worker_usage,
                                    block,
                                    info,
                                } => {
                                    let mut cfg = build_block(
                                        name,
                                        DataState::Owner,
                                        expect.clone(),
                                        block,
                                        mod_name.clone(),
                                    );

                                    cfg.compute_liveness();

                                    // refine loan expiry points now that liveness is known
                                    cfg.finalize_loan_lifetimes(); //

                                    // run checks in dependency order
                                    let move_errors = cfg.check_use_after_move(); // needs: nothing
                                    let borrow_errors = cfg.check_borrow_conflicts(); // needs: loans populated
                                    let lifetime_errs = cfg.check_loan_lifetimes(); // needs: live_ranges + loans

                                    // collect all errors
                                    let all_errors: Vec<BorrowError> = move_errors
                                        .into_iter()
                                        .chain(borrow_errors)
                                        .chain(lifetime_errs)
                                        .collect();

                                    add_errors(all_errors, analyser);

                                    mod_cfg.main_fn = Some(FuncDeclInfo {
                                        is_async: false,
                                        ident_span: ident.span,
                                        expect,
                                        ret_state: DataState::Owner,
                                        generics: None,
                                        par: vec![],
                                        is_extern: None,
                                        is_public: false,
                                        is_visible: false,
                                        worker_usage,
                                        block: FuncExpr::Cfg(cfg),
                                        stm_span: info.span,
                                    });
                                }
                                _ => (),
                            }
                        }

                        for_each_func(&mod_name, fns, &mut mod_cfg, analyser);

                        for_each_cust_ty(&mod_name, custom_tys, &mut mod_cfg, analyser);

                        for platform in platform_n_s {
                            match platform {
                                TopLevel::PlatformNS { arms, info } => {
                                    let mut arms_infos = vec![];

                                    for arm in arms {
                                        match arm.clone() {
                                            PlatformArm {
                                                target,
                                                block,
                                                span,
                                            } => match &block {
                                                CustomBlock {
                                                    takes,
                                                    globals: _,
                                                    macro_def: _,
                                                    custom_tys,
                                                    traits: _,
                                                    workers: _,
                                                    funcs,
                                                    platform_n_s: _,
                                                    static_defs: _,
                                                } => {
                                                    let mut platform_mod_cfg = ModCFG {
                                                        globals: HashMap::new(),
                                                        platform_n_s: Vec::new(),
                                                        takes: HashMap::new(),
                                                        traits: HashMap::new(),
                                                        workers: HashMap::new(),
                                                        statics: HashMap::new(),
                                                        custom_tys: HashMap::new(),
                                                        main_fn: None,
                                                        funcs: HashMap::new(),
                                                        methods: HashMap::new(),
                                                        static_meths: HashMap::new(),
                                                        bridges: HashMap::new(),
                                                    };
                                                
                                                    // FOR TAKES
                                                    for_each_cust_block_take(takes, &mut platform_mod_cfg);
                                                
                                                    for_each_cust_block_func(&mod_name, funcs, &mut platform_mod_cfg, analyser);
                                                
                                                    for_each_cust_block_cust_ty(&mod_name, custom_tys, &mut platform_mod_cfg, analyser);

                                                    arms_infos.push(
                                                        PlatformArmInfo { target, block: platform_mod_cfg, span }
                                                    );
                                                }
                                            },
                                        }
                                    }

                                    mod_cfg.platform_n_s.push(
                                        PlatformNSInfo { arms: arms_infos, stm_span: info.span.clone() }
                                    );
                                }
                                _ => (),
                            }
                        }

                        ret.insert(mod_name.clone(), Resource::File(mod_cfg));
                    }
                }
            }
            NameSpace::Lib {
                name,
                manifest: _,
                entry,
                src,
            } => {
                // already in CFG
                ret.insert(name, Resource::Package { entry, src });
            }
            NameSpace::Plural { name: _, files: _ } => (),
        }
    }

    ret
}
