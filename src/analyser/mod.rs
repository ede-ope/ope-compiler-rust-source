pub(crate) mod blocklevel_analyser;

use crate::borrow_checker::MethodCfg;
// use crate::borrow_checker::BorrowChecker;
use crate::compiler::{ErrorHint, Mod, NameSpace, PanicFn, SemanticError};
use crate::name_resolver::ElemSignature;
use crate::{name_resolver, parser, type_checker, ComdInfo};
// use crate::name_resolver::{self, *};
// use self::blocklevel_analyser::Expect;
use crate::parser::{
    AssFunc, BlockLevel, BridgeDecl, CustTyAss, CustomBlock, CustomLevel, DataState, EnumVariants,
    Expr, ExprKind, Generic, Ident, InterfaceParam, Method, ModPath, Os, OsArm, StmtInfo, TopLevel,
    TraitImpl, TypeHint, TypeKey, TypeSpec,
};
use crate::scanner::{FullTok, Pos, Span, Token};
// use crate::type_checker::{self, *};
use std::collections::{BTreeSet, HashMap};

// #[derive(Debug, Clone)]
// #[allow(dead_code)]
// pub enum Element {
//     Global(TopLevel),
//     Local(BlockLevel),
// }

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Variable {
    pub state: DataState,
    pub ty: TypeSpec,
    // pub value: Expr,
    pub is_mutable: Option<bool>, // global variables are None
    pub ident_span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Analyser {
    pub unit: HashMap<String, NameSpace>,
    pub entry: String,
    pub lib_list: Vec<String>,
    pub mod_list: Vec<String>,
    pub cli: ComdInfo,
    pub modl: String,
    pub cust_tys: HashMap<String, bool>,
    pub symbol_tables: HashMap<String, Vec<HashMap<String, ElemSignature>>>,
    //HashMap for each mod and the other for each scope
    pub symbol_table: Vec<HashMap<String, ElemSignature>>,
    pub panic_from: PanicFn,
    hir_index: usize,
    pub errs: Vec<SemanticError>,
}

impl Analyser {
    // Semantic analyses helpers
    //
    //

    pub fn new(
        entry: String,
        cli: ComdInfo,
        lib_list: Vec<String>,
        mod_list: Vec<String>,
        unit: HashMap<String, NameSpace>,
    ) -> Self {
        // takes refrence because Hir does not mutate anymore(after the macro expansion)
        //
        Self {
            unit,
            lib_list,
            mod_list,
            entry,
            cli,
            modl: String::new(),
            cust_tys: HashMap::new(),
            symbol_tables: HashMap::new(),
            symbol_table: vec![],
            panic_from: PanicFn::Owned, // tmp

            hir_index: 0,
            errs: Vec::new(),
        }
    }

    pub fn get_element(&mut self, ident: &Ident) -> Option<ElemSignature> {
        // from current module
        for scope in self.symbol_table.iter().rev() {
            let element = scope.get(&ident.ident);
            match element {
                Some(ElemSignature::TakeStmt { elem, .. }) => {
                    return Some(*elem.clone());
                }
                Some(elem) => {
                    return Some(elem.clone());
                }
                _ => (),
            }
        }

        return None; // by default
    }

    pub fn get_custom_type(
        &mut self,
        ident: &Ident,
        access: &Vec<Ident>,
        genrs: &Vec<Generic>,
    ) -> Option<ElemSignature> {
        let mut ret = None;

        let element = self.get_element(ident);

        if element.is_some() {
            let ty = element.unwrap();
            if access.len() == 0 {
                match ty {
                    ElemSignature::EnumDecl {
                        ident: _,
                        generic_types: _,
                        vals: _,
                        traits: _,
                        public: _,
                        span: _,
                    } => ret = Some(ty),
                    ElemSignature::StructDecl {
                        ident: _,
                        generic_types: _,
                        vals: _,
                        traits: _,
                        public: _,
                        span: _,
                    } => ret = Some(ty),
                    _ => (), // get_element method does not return a take stm raw
                }
            } else {
                match ty {
                    ElemSignature::ProjectLevel => {
                        ret = Some(ty);
                    }
                    _ => (), // get_element method does not return a take stm raw
                }
            }
        } else {
            for genr in genrs {
                match genr {
                    Generic { name, traits } => {
                        if ident.ident == name.ident {
                            // Found
                            ret = Some(ElemSignature::GenericType {
                                name: name.clone(),
                                traits: traits.clone(),
                            });
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn get_custom_from_take(
        &mut self,
        take: (FullTok, Option<Ident>),
        from: ModPath,
        lib: Option<Ident>,
    ) -> Option<TopLevel> {
        let mut ret = None;

        let key;
        if lib == None {
            match from {
                ModPath::Ident(ident) => {
                    key = ident.ident;
                }
                _ => {
                    key = String::from("");
                }
            }
        } else {
            key = lib.unwrap().ident.clone();
        }

        let module = self.unit.get(&key);
        if module.is_some() {
            let module = module.unwrap();
            match module {
                NameSpace::Singular { name: _, file } => {
                    // most be lib-None
                    for c_t in &file.custom_tys {
                        match c_t {
                            TopLevel::EnumDecl {
                                ident,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public,
                                info: _,
                            } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if public.clone() {
                                                ret = Some(c_t.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            TopLevel::StructDecl {
                                ident,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public,
                                info: _,
                            } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if public.clone() {
                                                ret = Some(c_t.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
                NameSpace::Lib { .. } => {}
            }
        }

        ret
    }

    pub fn get_namespace_from_take(
        &mut self,
        take: &(FullTok, Option<Ident>),
        from: &ModPath,
        lib: &Option<Ident>,
    ) -> Option<TopLevel> {
        let mut ret = None;

        let key;
        if *lib == None {
            match from {
                ModPath::Ident(ident) => {
                    key = ident.ident.clone();
                }
                _ => {
                    key = String::from("");
                }
            }
        } else {
            key = lib.clone().unwrap().ident;
        }

        let module = self.unit.get(&key);
        if module.is_some() {
            let module = module.unwrap();
            match module {
                NameSpace::Singular { name: _, file } => {
                    // most be lib-None
                    for c_t in &file.custom_tys {
                        match c_t {
                            TopLevel::EnumDecl {
                                ident,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public,
                                info: _,
                            } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if public.clone() {
                                                ret = Some(c_t.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            TopLevel::StructDecl {
                                ident,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public,
                                info: _,
                            } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if public.clone() {
                                                ret = Some(c_t.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            TopLevel::StaticDef { ident, public, .. } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if public.clone() {
                                                ret = Some(c_t.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
                NameSpace::Lib { .. } => {}
            }
        }

        ret
    }

    pub fn _get_func_from_take(
        &mut self,
        take: (FullTok, Option<Ident>),
        from: ModPath,
        lib: Option<Ident>,
    ) -> Option<TopLevel> {
        let mut ret = None;

        let key;
        if lib == None {
            match from {
                ModPath::Ident(ident) => {
                    key = ident.ident;
                }
                _ => {
                    key = String::from("");
                }
            }
        } else {
            key = lib.unwrap().ident.clone();
        }

        let module = self.unit.get(&key);
        if module.is_some() {
            let module = module.unwrap();
            match module {
                NameSpace::Singular { name: _, file } => {
                    // most be lib-None
                    for fun in &file.fns {
                        match fun {
                            TopLevel::FuncDecl {
                                ident, is_public, ..
                            } => {
                                match take.0.clone().kind {
                                    Token::Identifier(s) => {
                                        if s == ident.ident {
                                            // found it
                                            if is_public.clone() {
                                                ret = Some(fun.clone());
                                            }
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
                NameSpace::Lib { .. } => {}
            }
        }

        ret
    }

    pub fn get_var(&mut self, ident: &Ident) -> Option<Variable> {
        // let mut from_latest_scope = true;

        // for scope in self.symbol_table.iter().rev() {
        //     let element = scope.get(ident);
        //     if element.is_some() {
        //         return (element.unwrap().clone(), from_latest_scope);
        //     }
        //     from_latest_scope = false;
        // }

        // for scope in self.symbol_table.iter().rev() {
        //     let element = scope.get(&ident.ident);
        //     if element.is_some() {
        //         match element.unwrap().clone() {
        //             Element::Global(elem) => match elem {
        //                 TopLevel::GlobalDecl {
        //                     interpret: _,
        //                     state,
        //                     ty,
        //                     name,
        //                     value,
        //                     public: _,
        //                     info: _,
        //                 } => {
        //                     return Some((
        //                         Variable {
        //                             state,
        //                             ty,
        //                             value,
        //                             is_mutable: None,
        //                             ident_span: name.span,
        //                         },
        //                         from_latest_scope,
        //                     ));
        //                 }
        //                 TopLevel::TakeStmt {
        //                     take,
        //                     from,
        //                     lib,
        //                     info: _,
        //                 } => {
        //                     match &from {
        //                         ModPath::Ident(mod_ident) => {
        //                             let module = self.unit.get(&mod_ident.ident);
        //                             if module.is_some() {
        //                                 let module = module.unwrap();
        //                                 match module {
        //                                     NameSpace::Singular { name: _, file } => {
        //                                         if lib == None {
        //                                             for g in &file.globals {
        //                                                 match g {
        //                                                     TopLevel::GlobalDecl {
        //                                                         interpret: _,
        //                                                         state,
        //                                                         ty,
        //                                                         name,
        //                                                         value,
        //                                                         public,
        //                                                         info: _,
        //                                                     } => {
        //                                                         match take.0.clone().kind {
        //                                                             Token::Identifier(s) => {
        //                                                                 if s == name.ident {
        //                                                                     // found it
        //                                                                     if public.clone() {
        //                                                                         return Some((
        //                                                                             Variable {
        //                                                                                 state: state
        //                                                                                     .clone(),
        //                                                                                 ty: ty.clone(),
        //                                                                                 value: value
        //                                                                                     .clone(),
        //                                                                                 is_mutable: None,
        //                                                                                 ident_span: name
        //                                                                                     .span
        //                                                                                     .clone(),
        //                                                                             },
        //                                                                             from_latest_scope,
        //                                                                         ));
        //                                                                     } else {
        //                                                                         self.errs.push(SemanticError {
        //                                                                             messages: vec![format!(
        //                                                                                 "'{}', is not public",
        //                                                                                 name.ident
        //                                                                             )],
        //                                                                             hints: vec![],
        //                                                                             span: name.span.clone(),
        //                                                                         });
        //                                                                     }
        //                                                                 }
        //                                                             }
        //                                                             _ => (),
        //                                                         }
        //                                                     }
        //                                                     _ => (),
        //                                                 }
        //                                             }
        //                                         } else {
        //                                         }
        //                                     }
        //                                     NameSpace::Plural { name: _, files: _ } => {}
        //                                     NameSpace::Lib { .. } => {}
        //                                 }
        //                             } else {
        //                                 self.errs.push(SemanticError {
        //                                     messages: vec![format!(
        //                                         "Can't find module '{:?}'",
        //                                         from
        //                                     )],
        //                                     hints: vec![],
        //                                     span: mod_ident.span.clone(),
        //                                 }); // for now
        //                             }
        //                         }
        //                         ModPath::Str(_) => {
        //                             todo!("todo");
        //                         }
        //                         ModPath::UnExpected => {
        //                             //
        //                             panic!("Ewe")
        //                         }
        //                         ModPath::FromLibFolder(_, _) => {
        //                             panic!("Take from lib folder")
        //                         }
        //                     }
        //                 }
        //                 _ => (),
        //             },
        //             Element::Local(elem) => match elem {
        //                 BlockLevel::VarDecl {
        //                     interpret: _,
        //                     state,
        //                     ty,
        //                     name,
        //                     value,
        //                     mutable,
        //                     info: _,
        //                 } => {
        //                     return Some((
        //                         Variable {
        //                             state,
        //                             ty,
        //                             value,
        //                             is_mutable: Some(mutable.clone()),
        //                             ident_span: name.span,
        //                         },
        //                         from_latest_scope,
        //                     ));
        //                 }
        //                 BlockLevel::UnInitVarDecl {
        //                     state,
        //                     ty,
        //                     name,
        //                     mutable,
        //                     info,
        //                 } => {
        //                     return Some((
        //                         Variable {
        //                             state,
        //                             ty,
        //                             value: Expr {
        //                                 kind: ExprKind::Undefined,
        //                                 span: info.span,
        //                             },
        //                             is_mutable: Some(mutable.clone()),
        //                             ident_span: name.span,
        //                         },
        //                         from_latest_scope,
        //                     ));
        //                 }
        //                 BlockLevel::Param {
        //                     state,
        //                     ty,
        //                     name,
        //                     mutable,
        //                     info: _,
        //                 } => {
        //                     match state {
        //                         DataState::ImutRef | DataState::MutRef => {
        //                             return Some((
        //                                 Variable {
        //                                     state,
        //                                     ty,
        //                                     value: Expr {
        //                                         kind: ExprKind::Foo,
        //                                         span: name.span.clone(),
        //                                     },
        //                                     is_mutable: Some(mutable),
        //                                     ident_span: name.span,
        //                                 },
        //                                 false, // which is why the block can throw(return) it
        //                             ));
        //                         }
        //                         _ => {
        //                             return Some((
        //                                 Variable {
        //                                     state,
        //                                     ty,
        //                                     value: Expr {
        //                                         kind: ExprKind::Foo,
        //                                         span: name.span.clone(),
        //                                     },
        //                                     is_mutable: Some(mutable),
        //                                     ident_span: name.span,
        //                                 },
        //                                 true, // which is why the block cannot throw(return) it
        //                             ));
        //                         }
        //                     }
        //                 }
        //                 _ => (),
        //             },
        //         }
        //     }
        //     from_latest_scope = false;
        //     // i += 1;
        // }

        let elem = self.get_element(ident);

        if elem.is_some() {
            let elem = elem.unwrap();

            match elem {
                ElemSignature::GlobalDecl {
                    state, ty, name, ..
                } => {
                    return Some(Variable {
                        state,
                        ty,
                        is_mutable: None,
                        ident_span: name.span.clone(),
                    });
                }
                ElemSignature::VarDecl {
                    state,
                    ty,
                    name,
                    mutable,
                    ..
                } => {
                    return Some(Variable {
                        state,
                        ty,
                        is_mutable: Some(mutable),
                        ident_span: name.span.clone(),
                    });
                }
                _ => (),
            }
        }

        return None; // by default
    }

    fn for_globals(&mut self, globals: &Vec<TopLevel>) {
        for glo in globals {
            match glo {
                TopLevel::GlobalDecl {
                    interpret,
                    state,
                    ty,
                    name,
                    value,
                    public: _,
                    info: _,
                } => {
                    if *interpret {
                        match ty {
                            TypeSpec {
                                hint: TypeHint::Void,
                                span: _,
                            } => {
                                self.errs.push(SemanticError {
                                    messages: vec![
                                        format!(
                                            "The value of variable '{}' most be know at compile time, due to the use of the túmọ̀ keyword",
                                            name.ident.clone()
                                        ),
                                        format!("Therefore definite value most be binded to it")
                                    ],
                                    hints: vec![],
                                    span: name.span.clone(),
                                });
                            }
                            _ => (),
                        }
                    }

                    blocklevel_analyser::validate_assignment(
                        &ty,
                        state.clone(),
                        &value,
                        &vec![],
                        &vec![],
                        self,
                    );
                }
                _ => (),
            }
        }
    }

    fn for_funcs(&mut self, funcs: &Vec<TopLevel>) {
        for func in funcs {
            match func {
                TopLevel::FuncDecl {
                    is_async: _,
                    expect,
                    ret_state,
                    ident: _,
                    generics,
                    is_visible: _,
                    is_public: _,
                    is_extern: _,
                    par,
                    worker_usage: _,
                    block,
                    info: _,
                } => {
                    let genr;
                    if generics.is_none() {
                        genr = vec![];
                    } else {
                        genr = generics.clone().unwrap();
                    }
                    match expect.hint {
                        TypeHint::Void => {
                            blocklevel_analyser::check_for_val_compatibility(
                                block.clone(),
                                &blocklevel_analyser::Expect::Void,
                                &genr,
                                &par,
                                self,
                            );
                            // self.analyse_block(Expect::Void, &block);
                        }
                        _ => {
                            blocklevel_analyser::check_for_val_compatibility(
                                block.clone(),
                                &blocklevel_analyser::Expect::Required(
                                    expect.clone(),
                                    ret_state.clone(), // for it to accept raw value (values other than ident)
                                ),
                                &genr,
                                &par,
                                self,
                            );
                            // self.analyse_block(Expect::Required(expect.clone()), &block);
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn for_os_target(&mut self, platforms: &[TopLevel]) {
        for plat in platforms {
            match plat {
                TopLevel::OsTarget { arms, info } => {
                    let mut to_outside = vec![];
                    let mut to_outside_funcs = vec![];
                    let mut satisfied_variants = vec![];

                    let original_symbol_table = self.symbol_table.clone();
                    for arm in arms {
                        let mut arm_to_outside = vec![];
                        let mut arm_to_outside_funcs = vec![];
                        let target_expr;
                        match arm {
                            OsArm {
                                target,
                                block,
                                span: _,
                            } => {
                                // let zero_span = Span {
                                //     st: Pos { column: 0, line: 0 },
                                //     en: Pos { column: 0, line: 0 },
                                //     file: self.modl.clone(),
                                // };

                                // check the pattern based on the kind expected
                                // if let Os::Others(_) = target {
                                // } else {
                                // blocklevel_analyser::check_for_val_compatibility(
                                //     target.clone(),
                                //     &Expect::Required(
                                //         TypeSpec {
                                //             hint: TypeHint::Custom {
                                //                 get: None,
                                //                 access: Ident {
                                //                     ident: String::from("KM"),
                                //                     span: zero_span.clone(),
                                //                 },
                                //                 generics: vec![],
                                //             },
                                //             span: zero_span,
                                //         },
                                //         DataState::Owner,
                                //     ),
                                //     &vec![],
                                //     &vec![],
                                //     self,
                                // );
                                // Platform::Arch
                                // blocklevel_analyser::check_for_val_compatibility(
                                //     target.clone(),
                                //     &Expect::Required(
                                //         TypeSpec {
                                //             hint: TypeHint::Custom {
                                //                 import: None,
                                //                 ident: Ident {
                                //                     ident: String::from("IrúẸ̀rọ"),
                                //                     span: zero_span.clone(),
                                //                 },
                                //                 generics: vec![],
                                //             },
                                //             span: zero_span,
                                //         },
                                //         DataState::Owner,
                                //     ),
                                //     &vec![],
                                //     &vec![],
                                //     self,
                                // );
                                // }
                                // analyse the block
                                match block.clone() {
                                    CustomBlock {
                                        takes,
                                        globals,
                                        // macro_def,
                                        custom_tys,
                                        traits,
                                        workers,
                                        funcs,
                                        static_defs,
                                        platform_n_s: _,
                                    } => {
                                        let platform_mod_name = format!("--platform%{}", self.modl);
                                        // let mut top_macros = vec![];
                                        let mut top_fns = vec![];
                                        let mut top_takes = vec![];
                                        let mut top_globals = vec![];
                                        let mut top_traits = vec![];
                                        let mut top_workers = vec![];
                                        let mut top_customs = vec![];
                                        let mut top_static = vec![];
                                        let top_methods;
                                        let top_bridges;
                                        let top_trait_impls;

                                        {
                                            // 1. we need to add every element in arm first

                                            // for mac in macro_def {
                                            //     match mac {
                                            //         CustomLevel::MacroDef {
                                            //             ident,
                                            //             is_public,
                                            //             cases,
                                            //             info,
                                            //         } => {
                                            //             top_macros.push(TopLevel::MacroDef {
                                            //                 ident,
                                            //                 is_public,
                                            //                 cases,
                                            //                 info,
                                            //             });
                                            //         }
                                            //         _ => (),
                                            //     }
                                            // }

                                            for func in funcs {
                                                match func {
                                                    CustomLevel::FuncDecl {
                                                        is_async,
                                                        expect,
                                                        ret_state,
                                                        ident,
                                                        generics,
                                                        is_public,
                                                        is_visible,
                                                        is_extern,
                                                        par,
                                                        worker_usage,
                                                        block,
                                                        info,
                                                    } => {
                                                        top_fns.push(TopLevel::FuncDecl {
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
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for tk in takes {
                                                match tk {
                                                    CustomLevel::TakeStmt {
                                                        take,
                                                        from,
                                                        lib,
                                                        info,
                                                    } => {
                                                        top_takes.push(TopLevel::TakeStmt {
                                                            take,
                                                            from,
                                                            lib,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for glo in globals {
                                                match glo {
                                                    CustomLevel::GlobalDecl {
                                                        interpret,
                                                        state,
                                                        ty,
                                                        name,
                                                        value,
                                                        public,
                                                        info,
                                                    } => {
                                                        top_globals.push(TopLevel::GlobalDecl {
                                                            interpret,
                                                            state,
                                                            ty,
                                                            name,
                                                            value,
                                                            public,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for tr in traits {
                                                match tr {
                                                    CustomLevel::TraitDef {
                                                        ident,
                                                        generic_types,
                                                        // interprets,
                                                        methods,
                                                        is_public,
                                                        info,
                                                    } => {
                                                        top_traits.push(TopLevel::TraitDef {
                                                            ident,
                                                            generic_types,
                                                            // interprets,
                                                            methods,
                                                            is_public,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for wk in workers {
                                                match wk {
                                                    CustomLevel::WorkerDef {
                                                        ident,
                                                        func_name,
                                                        public,
                                                        block,
                                                        info,
                                                    } => {
                                                        top_workers.push(TopLevel::WorkerDef {
                                                            ident,
                                                            func_name,
                                                            public,
                                                            block,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for sta in static_defs {
                                                match sta {
                                                    CustomLevel::StaticDef {
                                                        ident,
                                                        public,
                                                        block,
                                                        info,
                                                    } => {
                                                        top_static.push(TopLevel::StaticDef {
                                                            ident,
                                                            public,
                                                            block,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            for cus in custom_tys {
                                                match cus {
                                                    CustomLevel::EnumDecl {
                                                        ident,
                                                        generic_types,
                                                        vals,
                                                        traits,
                                                        public,
                                                        info,
                                                    } => {
                                                        top_customs.push(TopLevel::EnumDecl {
                                                            ident,
                                                            generic_types,
                                                            vals,
                                                            traits,
                                                            public,
                                                            info,
                                                        });
                                                    }
                                                    CustomLevel::StructDecl {
                                                        ident,
                                                        generic_types,
                                                        vals,
                                                        traits,
                                                        public,
                                                        info,
                                                    } => {
                                                        top_customs.push(TopLevel::StructDecl {
                                                            ident,
                                                            generic_types,
                                                            vals,
                                                            traits,
                                                            public,
                                                            info,
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            // // 2. then the elements of the current module it self for proper analyses
                                            let modl = self.unit.get(&self.modl);

                                            if modl.is_some() {
                                                let modl = modl.unwrap();
                                                match modl {
                                                    NameSpace::Singular { name: _, file } => {
                                                        match file {
                                                            Mod {
                                                                mod_docs: _,
                                                                main_fn: _,
                                                                // macro_defs,
                                                                fns: _,
                                                                os_n_s: _,
                                                                arch_n_s: _,
                                                                takes: _,
                                                                globals: _,
                                                                traits: _,
                                                                workers: _,
                                                                statics: _,
                                                                custom_tys: _,
                                                                methods,
                                                                bridges,
                                                                trait_impls,
                                                                lines: _,
                                                                scope: _,
                                                            } => {
                                                                top_methods = methods.clone();
                                                                top_bridges = bridges.clone();
                                                                top_trait_impls =
                                                                    trait_impls.clone();
                                                            }
                                                        }
                                                    }
                                                    _ => panic!("Should be Singular"),
                                                }
                                            } else {
                                                todo!("module not found error")
                                            }
                                        }

                                        let platform_mod = Mod {
                                            mod_docs: None,
                                            main_fn: None,
                                            // macro_defs: top_macros,
                                            fns: top_fns,
                                            os_n_s: vec![],
                                            arch_n_s: vec![],
                                            takes: top_takes,
                                            globals: top_globals,
                                            traits: top_traits,
                                            workers: top_workers,
                                            statics: vec![],
                                            custom_tys: top_customs,
                                            methods: top_methods,
                                            bridges: top_bridges,
                                            trait_impls: top_trait_impls,
                                            lines: vec![],
                                            scope: 0,
                                        };

                                        // Initialise
                                        // add the elems in current arm to the the original table
                                        // this will override the current meths, bridges and trait_impls in self
                                        name_resolver::register_mod(
                                            &platform_mod_name,
                                            &platform_mod,
                                            self,
                                        );
                                        // Then analyse it...

                                        // Globals
                                        self.for_globals(&platform_mod.globals);

                                        // ...Functions
                                        // let's add the outside functions first
                                        for func in &platform_mod.fns {
                                            match func {
                                                TopLevel::FuncDecl {
                                                    ident, is_visible, ..
                                                } => {
                                                    if *is_visible {
                                                        arm_to_outside.push(ident.clone());
                                                        arm_to_outside_funcs.push(func.clone());
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                        self.for_funcs(&platform_mod.fns);

                                        // Custom types
                                        self.for_custom_types(&platform_mod.custom_tys);
                                    }
                                }

                                satisfied_variants.push(target.clone());
                                target_expr = target.clone();
                            }
                        }
                        to_outside.push((target_expr, arm_to_outside));
                        to_outside_funcs.push(arm_to_outside_funcs);
                    }

                    self.does_all_arms_have_the_same_outside_func(to_outside);

                    if self.check_os_arm_exhaustiveness(arms, info.span.clone()) {
                        // if every arm has the same outside functions
                        // we need to check if the signature matches
                        self.check_outside_func_signatures(to_outside_funcs);
                    }
                    self.symbol_table = original_symbol_table;
                }
                TopLevel::ArchTarget { arms: _, info: _ } => {
                    todo!("Arch target")
                }
                _ => (),
            }
        }
    }

    fn check_os_arm_exhaustiveness(
        &mut self,
        // kind: Platform,
        arms: &Vec<OsArm>,
        plat_header_span: Span,
    ) -> bool {
        let mut is_exhausted = true;
        //&[(Expr, Span)]
        let mut seen: HashMap<String, Span> = HashMap::new();
        let mut default_seen: Option<Span> = None;

        for OsArm {
            target, span: _, ..
        } in arms
        {
            // Anything after a default arm is unreachable, regardless of what it is.
            if let Some(default_span) = &default_seen {
                self.errs.push(SemanticError {
                    messages: vec!["This arm is unreachable — it comes after a default arm".into()],
                    hints: vec![ErrorHint {
                        hints: vec![
                            "the default arm here matches everything, so nothing after it can run"
                                .into(),
                        ],
                        span: default_span.clone(),
                    }],
                    span: self.get_os_span(target),
                });
                is_exhausted = false;
                continue;
            }

            match target {
                Os::Others(s) => {
                    default_seen = Some(s.clone());
                    continue;
                }
                _ => (),
            }

            let Some(variant_name) = self.extract_variant_name(target) else {
                continue; // malformed pattern — presumably reported elsewhere
            };

            if let Some(prev_span) = seen.insert(variant_name.to_string(), self.get_os_span(target))
            {
                self.errs.push(SemanticError {
                    messages: vec![format!("Duplicate arm for `{}`", variant_name)],
                    hints: vec![ErrorHint {
                        hints: vec!["already matched here".into()],
                        span: prev_span,
                    }],
                    span: self.get_os_span(target),
                });
                is_exhausted = false;
            }
        }

        // Exhaustiveness only matters if there's no default arm to catch the rest.
        if default_seen.is_none() {
            let vals = vec![
                format!("Linux"),
                format!("Darwin"),
                format!("Windows"),
                format!("Others"),
                format!("Undefined"),
            ];
            let mut name_of_missing_variants = String::new();
            let mut emit_error = false;
            let mut i = 0;
            for name in &vals {
                // let name = self.variant_name(variant);
                if !seen.contains_key(name) {
                    emit_error = true;
                    if i == 0 {
                        name_of_missing_variants = name.clone();
                    } else {
                        name_of_missing_variants = name_of_missing_variants + &", " + name.as_ref();
                    }
                    i += 1;
                }
            }

            if emit_error {
                self.errs.push(SemanticError {
                    messages: vec![format!(
                        "Not all variants are covered — missing {} (and no default `àrọ́sí` arm)",
                        name_of_missing_variants
                    )],
                    hints: vec![],
                    span: plat_header_span.clone(),
                });
                is_exhausted = false;
            }
        }

        is_exhausted
    }

    fn extract_variant_name(&self, target: &Os) -> Option<String> {
        match target {
            Os::Windows(_) => Some(format!("Windows")),
            Os::Linux(_) => Some(format!("Linux")),
            Os::Darwin(_) => Some(format!("Darwin")),
            Os::Others(_) => Some(format!("Others")),
            _ => None,
        }
    }

    fn variant_name(&self, v: &EnumVariants) -> String {
        match v {
            EnumVariants::Simple(ident) => ident.ident.clone(),
            EnumVariants::Tuple { name, .. } => name.ident.clone(),
            EnumVariants::Record { name, .. } => name.ident.clone(),
        }
    }

    fn get_os_span(&self, os: &Os) -> Span {
        match os.clone() {
            Os::Linux(span) => span,
            Os::Darwin(s) => s,
            Os::Windows(s) => s,
            Os::Others(s) => s,
            Os::Undefined => Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.modl.clone(),
            },
        }
    }

    fn does_all_arms_have_the_same_outside_func(&mut self, to_outside: Vec<(Os, Vec<Ident>)>) {
        if to_outside.len() <= 1 {
            return;
        }

        let sets: Vec<BTreeSet<&str>> = to_outside
            .iter()
            .map(|(_, idents)| idents.iter().map(|id| id.ident.as_str()).collect())
            .collect();

        let union: BTreeSet<&str> = sets.iter().flat_map(|s| s.iter().copied()).collect();
        let intersection: BTreeSet<&str> = union
            .iter()
            .copied()
            .filter(|name| sets.iter().all(|s| s.contains(name)))
            .collect();

        if union == intersection {
            return;
        }

        // For the hint: first arm (by index) that actually has a given name.
        let first_occurrence = |name: &str| -> Option<&Ident> {
            to_outside
                .iter()
                .flat_map(|(_, idents)| idents.iter())
                .find(|id| id.ident == name)
        };

        for (index, (arm_expr, _idents)) in to_outside.iter().enumerate() {
            let set = &sets[index];

            for name in union.difference(set) {
                let hint = first_occurrence(name).map(|id| ErrorHint {
                    hints: vec![format!("`{}` is defined here on another arm", id.ident)],
                    span: id.span.clone(),
                });

                self.errs.push(SemanticError {
                    messages: vec![format!(
                        "Function {} is missing on arm {:?}",
                        name, arm_expr
                    )],
                    hints: hint.into_iter().collect(),
                    span: self.get_os_span(arm_expr),
                });
            }
        }
    }

    fn check_outside_func_signatures(&mut self, groups: Vec<Vec<TopLevel>>) {
        // ident -> first function declaration encountered
        let mut functions: HashMap<String, &TopLevel> = HashMap::new();

        for group in &groups {
            for item in group {
                let TopLevel::FuncDecl {
                    is_async,
                    expect,
                    par,
                    ident,
                    ..
                } = item
                else {
                    continue;
                };

                let name = ident.ident.clone();

                if let Some(TopLevel::FuncDecl {
                    is_async: first_async,
                    expect: first_expect,
                    par: first_par,
                    ..
                }) = functions.get(&name)
                {
                    if *is_async != *first_async {
                        self.errs.push(SemanticError {
                            messages: vec![format!(
                                "function `{name}` has conflicting async signatures"
                            )],
                            hints: vec![],
                            span: ident.span.clone(),
                        });
                    }

                    if expect.hint != first_expect.hint {
                        self.errs.push(SemanticError {
                            messages: vec![format!(
                                "function `{name}` has conflicting return types"
                            )],
                            hints: vec![],
                            span: expect.span.clone(),
                        });
                    }

                    let mut i = 0;
                    for p in par {
                        if p.is_mut != first_par[i].is_mut {
                            self.errs.push(SemanticError {
                                messages: vec![format!(
                                    "function `{name}` has conflicting parameter mutable state"
                                )],
                                hints: vec![],
                                span: p.span.clone(),
                            });
                        }

                        if p.state != first_par[i].state {
                            self.errs.push(SemanticError {
                                messages: vec![format!(
                                    "function `{name}` has conflicting parameter ownership state"
                                )],
                                hints: vec![],
                                span: p.span.clone(),
                            });
                        }

                        if p.ty.hint != first_par[i].ty.hint {
                            self.errs.push(SemanticError {
                                messages: vec![format!(
                                    "function `{name}` has conflicting parameter type"
                                )],
                                hints: vec![ErrorHint {
                                    hints: vec![format!(
                                        "Expected type is {}",
                                        first_par[i].ty.hint
                                    )],
                                    span: first_par[i].ty.span.clone(),
                                }],
                                span: p.ty.span.clone(),
                            });
                        }

                        i += 1;
                    }
                } else {
                    functions.insert(name, item);
                }
            }
        }
    }

    fn get_methods(&mut self, key: &TypeKey) -> Option<HashMap<String, CustTyAss>> {
        let meths = self.get_element(&Ident {
            ident: format!("<methods>"),
            span: Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.modl.clone(),
            },
        });

        if meths.is_some() {
            match meths.unwrap() {
                ElemSignature::Methods(all) => {
                    let ret = all.get(key);

                    if ret.is_some() {
                        Some(ret.unwrap().clone())
                    } else {
                        None
                    }
                }
                _ => panic!("uncalled for"),
            }
        } else {
            todo!("methods not found error")
        }
    }

    fn get_bridges(&mut self, key: &String) -> Option<HashMap<TypeKey, BridgeDecl>> {
        let brgs = self.get_element(&Ident {
            ident: format!("<bridges>"),
            span: Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.modl.clone(),
            },
        });

        if brgs.is_some() {
            match brgs.unwrap() {
                ElemSignature::Bridges(all) => {
                    let ret = all.get(key);

                    if ret.is_some() {
                        Some(ret.unwrap().clone())
                    } else {
                        None
                    }
                }
                _ => panic!("uncalled for"),
            }
        } else {
            todo!("methods not found error")
        }
    }

    fn get_impls(&mut self, key: &TypeKey) -> Option<HashMap<String, TraitImpl>> {
        let brgs = self.get_element(&Ident {
            ident: format!("<impls>"),
            span: Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.modl.clone(),
            },
        });

        if brgs.is_some() {
            match brgs.unwrap() {
                ElemSignature::TraitImpls(all) => {
                    let ret = all.get(key);

                    if ret.is_some() {
                        Some(ret.unwrap().clone())
                    } else {
                        None
                    }
                }
                _ => panic!("uncalled for"),
            }
        } else {
            todo!("methods not found error")
        }
    }

    fn analyse_cust_ty_bridges(&mut self, ty_ident: &Ident) {
        let bridges = self.get_bridges(&ty_ident.ident);

        if bridges.is_some() {
            let bridges = bridges.unwrap();
            for bridge in bridges.clone() {
                match bridge.1.clone() {
                    BridgeDecl {
                        ass_state,
                        ass_type,
                        ass_ident,
                        new_ty,
                        public: _,
                        block,
                        info: _,
                    } => {
                        blocklevel_analyser::check_for_val_compatibility(
                            block,
                            &blocklevel_analyser::Expect::Required(
                                TypeSpec {
                                    hint: TypeHint::Custom {
                                        get: new_ty.clone(),
                                        access: vec![],
                                        generics: vec![],
                                    },
                                    span: new_ty.span,
                                },
                                DataState::Owner,
                            ),
                            &vec![],
                            &vec![InterfaceParam {
                                ident: ass_ident.clone(),
                                state: ass_state,
                                ty: ass_type,
                                is_mut: false,
                                span: ass_ident.span,
                            }],
                            self,
                        );
                    }
                }
            }
        } else {
            todo!("Type has no bridges error!")
        }
    }

    fn analyse_cust_ty_association(&mut self, association: &HashMap<String, CustTyAss>) {
        let mut enum_meths = HashMap::new();
        // enum_meths are the methods of the enums in hashmap for easy access
        let mut cust_func = Vec::new();

        for ass in association.clone() {
            // here we are to analyse each method before adding them to enum_meths hashmap
            match ass.1 {
                CustTyAss::Method(meth) => {
                    match &meth {
                        Method {
                            worker_usage: _,
                            meth_self: _,
                            // inher_val: _,
                            // is_mut: _,
                            expect,
                            ret_state,
                            ident,
                            generics,
                            is_public: _,
                            is_extern: _,
                            par,
                            block,
                            info: _,
                        } => {
                            let genr;
                            if generics.is_none() {
                                genr = vec![];
                            } else {
                                genr = generics.clone().unwrap();
                            }

                            if block.is_some() {
                                let block = block.clone().unwrap();
                                match expect.hint {
                                    TypeHint::Void => {
                                        blocklevel_analyser::check_for_val_compatibility(
                                            block.clone(),
                                            &blocklevel_analyser::Expect::Void,
                                            &genr,
                                            &par,
                                            self,
                                        );
                                        // self.analyse_block(Expect::Void, &block);
                                    }
                                    _ => {
                                        blocklevel_analyser::check_for_val_compatibility(
                                            block.clone(),
                                            &blocklevel_analyser::Expect::Required(
                                                expect.clone(),
                                                ret_state.clone(),
                                            ),
                                            &genr,
                                            &par,
                                            self,
                                        );
                                        // self.analyse_block(Expect::Required(expect.clone()), &block);
                                    }
                                }
                            } else {
                                self.errs.push(SemanticError {
                                        messages: vec![
                                            format!(
                                                "Method {} is missing it's expression",
                                                ident.ident.clone()
                                            ),
                                            format!("For custom types, method defination with no expression binding is obsolite")
                                        ],
                                        hints: vec![],
                                        span: ident.span.clone(),
                                    });
                            }

                            if enum_meths.contains_key(&ident.ident) {
                                self.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "A method with identifier {} already existed",
                                        ident.ident.clone()
                                    )],
                                    hints: vec![],
                                    span: ident.span.clone(),
                                });
                            } else {
                                enum_meths.insert(ident.ident.clone(), meth.clone());
                            }
                        }
                    }
                }
                CustTyAss::AssFunction(func) => {
                    match func {
                        AssFunc {
                            worker_usage: _,
                            expect,
                            ret_state,
                            ident,
                            generics: genr,
                            is_public: _,
                            is_extern: _,
                            par,
                            block,
                            info: _,
                        } => {
                            if block.is_some() {
                                let block = block.clone().unwrap();
                                match expect.hint {
                                    TypeHint::Void => {
                                        blocklevel_analyser::check_for_val_compatibility(
                                            block.clone(),
                                            &blocklevel_analyser::Expect::Void,
                                            &genr,
                                            &par,
                                            self,
                                        );
                                        // self.analyse_block(Expect::Void, &block);
                                    }
                                    _ => {
                                        blocklevel_analyser::check_for_val_compatibility(
                                            block.clone(),
                                            &blocklevel_analyser::Expect::Required(
                                                expect.clone(),
                                                ret_state.clone(),
                                            ),
                                            &genr,
                                            &par,
                                            self,
                                        );
                                        // self.analyse_block(Expect::Required(expect.clone()), &block);
                                    }
                                }
                            } else {
                                self.errs.push(SemanticError {
                                        messages: vec![
                                            format!(
                                                "Method {} is missing it's expression",
                                                ident.ident.clone()
                                            ),
                                            format!("For custom types, method defination with no expression binding is obsolite")
                                        ],
                                        hints: vec![],
                                        span: ident.span.clone(),
                                    });
                            }

                            if cust_func.contains(&ident) {
                                self.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "A Static function with identifier {} already existed",
                                        ident.ident.clone()
                                    )],
                                    hints: vec![],
                                    span: ident.span.clone(),
                                });
                            } else {
                                cust_func.push(ident.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn for_methods(&mut self, meths: &HashMap<TypeKey, Vec<CustTyAss>>) {
        // let association = self.get_methods(ty_key);
        for each in meths {
            let meth = self.get_methods(each.0);

            if meth.is_some() {
                let meth = meth.unwrap();
                self.analyse_cust_ty_association(&meth);
            } else {
                panic!("method not found on symbol table")
            }
        }
    }

    fn check_impl(&mut self, ty: &TypeKey, ty_span: Span, traits: &Vec<Ident>) {
        // We need to see if any trait methods needed to be implimented
        let mut all_trait_methods = Vec::new();
        // all_trait_methods is the method of all the traits
        // weither, it needed to be defined by the type or not
        for wk in traits {
            let in_scope = self.get_element(wk);

            let mut show_default_err = true;

            if in_scope.is_some() {
                let elem = in_scope.unwrap();

                let mut trait_ = None;
                // let file = None;

                match elem.clone() {
                    ElemSignature::TraitDef {
                        ident,
                        generic_types: _,
                        // interprets: _,
                        methods: _,
                        is_public: _,
                        span: _,
                    } => {
                        trait_ = Some((elem, ident.span.clone()));
                    }
                    ElemSignature::TakeStmt { elem, .. } => match elem.as_ref() {
                        ElemSignature::TraitDef {
                            ident,
                            generic_types: _,
                            // interprets: _,
                            methods: _,
                            is_public: _,
                            span: _,
                        } => trait_ = Some((*elem.clone(), ident.span.clone())),
                        _ => (),
                    },
                    _ => (),
                }

                if trait_.is_some() {
                    let trait_ = trait_.unwrap();
                    match trait_.0 {
                        ElemSignature::TraitDef {
                            ident,
                            generic_types: _,
                            // interprets: _,
                            methods,
                            is_public: _,
                            span: _,
                        } => {
                            if wk.ident == ident.ident {
                                // found it
                                show_default_err = false;

                                for meth in methods {
                                    match &meth.1 {
                                        MethodCfg {
                                            worker_usage: _,
                                            meth_self: _,
                                            generics: _,
                                            is_public: _,
                                            is_extern,
                                            block,
                                            ..
                                        } => {
                                            if all_trait_methods.contains(&ident.ident) {
                                                self.errs.push(SemanticError {
                                                    messages: vec![format!(
                                                        "There is a clash of method identifier {} probably from another different trait",
                                                        ident.ident.clone()
                                                    )],
                                                    hints: vec![],
                                                    span: ident.span.clone(),
                                                });
                                            } else {
                                                all_trait_methods.push(ident.ident.clone());
                                            }

                                            // We need to check if the method is not extern and yet has no expression
                                            // then their most be an implementation of the trait for the type that has the method

                                            if block.is_none() && is_extern.is_none() {
                                                let impls = self.get_impls(ty);
                                                if impls.is_some() {
                                                    let impls = impls.unwrap();
                                                    let imp = impls.get(&wk.ident);

                                                    if imp.is_some() {
                                                        let imp = imp.unwrap();
                                                        let mut emit_error = true;
                                                        match imp {
                                                            TraitImpl { methods, .. } => {
                                                                for m in methods {
                                                                    match m {
                                                                        Method {
                                                                            ident, ..
                                                                        } => {
                                                                            if ident.ident == meth.0
                                                                            {
                                                                                //found the method
                                                                                emit_error = false;
                                                                                // the for_trait_impls() method checks if the method signature match their declaration
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if emit_error {
                                                            self.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "Trait '{}' required '{}' to implement method '{}'",
                                                                    wk.ident.clone(),
                                                                    ty,
                                                                    ident.ident.clone()
                                                                )],
                                                                hints: vec![
                                                                    ErrorHint {
                                                                        hints: vec![
                                                                            format!("Trait was defined here")
                                                                        ],
                                                                        span: trait_.1.clone()
                                                                    }
                                                                ],
                                                                span: ty_span.clone(),
                                                            });
                                                        }
                                                    } else {
                                                        self.errs.push(SemanticError {
                                                            messages: vec![format!(
                                                                "'{}' doesn't implement '{}'",
                                                                ty,
                                                                wk.ident.clone()
                                                            )],
                                                            hints: vec![ErrorHint {
                                                                hints: vec![format!(
                                                                    "Trait was defined here"
                                                                )],
                                                                span: trait_.1.clone(),
                                                            }],
                                                            span: ty_span.clone(),
                                                        });
                                                    }
                                                } else {
                                                    self.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Trait '{}' required '{}' to implement method '{}'",
                                                            wk.ident.clone(),
                                                            ty,
                                                            ident.ident.clone()
                                                        )],
                                                        hints: vec![
                                                            ErrorHint {
                                                                hints: vec![
                                                                    format!("Trait was defined here")
                                                                ],
                                                                span: trait_.1.clone()
                                                            }
                                                        ],
                                                        span: ty_span.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                } else {
                }
            }

            if show_default_err {
                self.errs.push(SemanticError {
                    messages: vec![format!("Couldn't find the trait {} in scope", wk.ident)],
                    hints: vec![],
                    span: wk.span.clone(),
                });
            }
        }
    }

    fn for_custom_types(&mut self, custom_tys: &Vec<TopLevel>) {
        for cust in custom_tys {
            // For each custom types both of enum and struct forms
            match cust {
                TopLevel::EnumDecl {
                    ident: enum_ident,
                    generic_types: _,
                    vals: _,
                    traits,
                    public: _,
                    info: _,
                } => {
                    self.check_impl(
                        &TypeKey::Custom(enum_ident.ident.clone()),
                        enum_ident.span.clone(),
                        traits,
                    );
                }
                TopLevel::StructDecl {
                    ident: struct_ident,
                    generic_types: _,
                    vals: _,
                    traits,
                    public: _,
                    info: _,
                } => {
                    self.check_impl(
                        &TypeKey::Custom(struct_ident.ident.clone()),
                        struct_ident.span.clone(),
                        traits,
                    );
                }
                _ => (),
            }
        }
    }

    pub fn hash_bridges(
        &self,
        bridges: &HashMap<String, Vec<BridgeDecl>>,
    ) -> HashMap<String, HashMap<TypeKey, BridgeDecl>> {
        // this function needs to check for clashes
        let mut ret = HashMap::new();
        for brg in bridges {
            let mut all = HashMap::new();
            for each in brg.1 {
                match each {
                    BridgeDecl { ass_type, .. } => {
                        let res = parser::get_typekey(&ass_type.hint);

                        if let Ok(key) = res {
                            all.insert(key, each.clone());
                        } else if let Err(err) = res {
                            todo!("{:#?}", err);
                        } else {
                            panic!("Error")
                        }
                    }
                }
            }
            ret.insert(brg.0.clone(), all);
        }
        ret
    }

    pub fn hash_methods(
        &self,
        methods: &HashMap<TypeKey, Vec<CustTyAss>>,
    ) -> HashMap<TypeKey, HashMap<String, CustTyAss>> {
        let mut ret = HashMap::new();
        for meth in methods {
            let mut all = HashMap::new();
            for each in meth.1 {
                match each {
                    CustTyAss::Method(m) => match m {
                        Method { ident, .. } => {
                            all.insert(ident.ident.clone(), each.clone());
                        }
                    },
                    CustTyAss::AssFunction(func) => match func {
                        AssFunc { ident, .. } => {
                            all.insert(ident.ident.clone(), each.clone());
                        }
                    },
                }
            }
            ret.insert(meth.0.clone(), all);
        }
        ret
    }

    pub fn hash_trt_impls(
        &self,
        trait_impls: &HashMap<TypeKey, Vec<TraitImpl>>,
    ) -> HashMap<TypeKey, HashMap<String, TraitImpl>> {
        let mut ret = HashMap::new();
        for imp in trait_impls {
            let mut all = HashMap::new();
            for each in imp.1 {
                match each {
                    TraitImpl { trt, .. } => {
                        all.insert(trt.ident.clone(), each.clone());
                    }
                }
            }
            ret.insert(imp.0.clone(), all);
        }
        ret
    }

    fn go_through_mod(&mut self, this: &Mod) {
        match this {
            Mod {
                mod_docs: _,
                main_fn,
                panic_fn,
                // macro_defs: _,
                fns,
                os_n_s,
                arch_n_s: _,
                takes: _,
                globals,
                traits: _,
                workers: _,
                statics: _,
                custom_tys,
                methods,
                bridges: _,
                trait_impls: _,
                lines: _,
                scope: _,
            } => {
                // panic fn
                {
                    // then check if the included libraries have panic
                    let mut has_panic = vec![];
                    let mut all_ackn_panic = vec![];
                    for lib in &self.lib_list {
                        let dp = self.unit.get(lib);

                        if dp.is_some() {
                            let dp = dp.unwrap();
                            match dp {
                                NameSpace::Lib {
                                    name,
                                    manifest,
                                    entry: _,
                                    ackn_panic,
                                    src: _,
                                } => {
                                    if manifest.package.panic_fn == PanicFn::Owned {
                                        has_panic.push(name.clone());
                                    }

                                    if *ackn_panic {
                                        all_ackn_panic.push(name.clone());
                                    }
                                }
                                _ => panic!(""),
                            }
                        } else {
                            panic!("dp data not found")
                        }
                    }

                    if panic_fn.is_some() {
                        // this unit has it own panic function
                        if all_ackn_panic.len() == 0 {
                            // and non is choosen
                            self.panic_from = PanicFn::Owned;
                        } else {
                            // there are one or more acknowlegbements
                            self.errs.push(SemanticError {
                                messages: vec![
                                    format!("There is a local panic function; remove any gba_fòyà flag on included libraries")
                                ],
                                hints: vec![],
                                span: Span {
                                    st: Pos {
                                        column: 0,
                                        line: 0
                                    },
                                    en: Pos {
                                        column: 0,
                                        line: 0
                                    },
                                    file: self.entry.clone()
                                },
                            });
                        }
                    } else {
                        // if there is no local panic fn
                        // check if this is the primary entry module
                        if self.entry == self.modl {
                            if has_panic.len() == 0 {
                                // There is no panic at all
                                self.errs.push(SemanticError {
                                    messages: vec![
                                        format!("Provide a local panic function or include a library with one e.g ipl(std)")
                                    ],
                                    hints: vec![],
                                    span: Span {
                                        st: Pos {
                                            column: 0,
                                            line: 0
                                        },
                                        en: Pos {
                                            column: 0,
                                            line: 0
                                        },
                                        file: self.entry.clone()
                                    },
                                });
                            } else if has_panic.len() == 1 {
                                // the one will be used
                                self.panic_from = PanicFn::Borrowed(has_panic[0].clone());
                            } else {
                                // we have more that one libraries that have panic function
                                if all_ackn_panic.len() == 0 {
                                    // and non is choosen
                                    self.errs.push(SemanticError {
                                        messages: vec![
                                            format!("There are multiple libraries with panic defined. You can choose one with the gba_fòyà keyword")
                                        ],
                                        hints: vec![],
                                        span: Span {
                                            st: Pos {
                                                column: 0,
                                                line: 0
                                            },
                                            en: Pos {
                                                column: 0,
                                                line: 0
                                            },
                                            file: self.entry.clone()
                                        },
                                    });
                                } else if all_ackn_panic.len() > 1 {
                                    self.errs.push(SemanticError {
                                        messages: vec![
                                            format!("There is an attempt to use more than one panic implementations in this unit")
                                        ],
                                        hints: vec![],
                                        span: Span {
                                            st: Pos {
                                                column: 0,
                                                line: 0
                                            },
                                            en: Pos {
                                                column: 0,
                                                line: 0
                                            },
                                            file: self.entry.clone()
                                        },
                                    });
                                } else {
                                    // there is one acknowlegbement
                                    self.panic_from = PanicFn::Borrowed(all_ackn_panic[0].clone());
                                }
                            }
                        }
                    }
                }

                // main fn
                if main_fn.is_some() {
                    self.symbol_table.push(HashMap::new()); // initializing the entry function scope

                    match main_fn.clone().unwrap() {
                        TopLevel::Entry {
                            ident,
                            expect,
                            worker_usage,
                            block,
                            info: entry_info,
                        } => match &expect {
                            TypeSpec {
                                hint: TypeHint::Void,
                                span: _,
                            } => {
                                // void should be converted to 32d(i32)
                                blocklevel_analyser::check_for_val_compatibility(
                                    block.clone(),
                                    &blocklevel_analyser::Expect::Void,
                                    &vec![],
                                    &vec![],
                                    self,
                                );
                                // self.analyse_block(Expect::Void, &block);
                                //
                                // void should be converted to 32d(i32)
                                match block {
                                    Expr {
                                        kind: ExprKind::Block(stms),
                                        span,
                                    } => {
                                        // void always
                                        let mut new_stms = stms.clone();
                                        new_stms.push(BlockLevel::BindedVal {
                                            expr: Expr {
                                                kind: ExprKind::AsType {
                                                    expr: Box::new(Expr {
                                                        kind: ExprKind::Number(String::from("0")),
                                                        span: span.clone(),
                                                    }),
                                                    ty: Box::new(TypeSpec {
                                                        hint: TypeHint::I32(None),
                                                        span: span.clone(),
                                                    }),
                                                },
                                                span: span.clone(),
                                            },
                                            info: StmtInfo {
                                                span: span.clone(),
                                                docs: vec![],
                                            },
                                        });
                                        let new_block = Expr {
                                            kind: ExprKind::Block(new_stms),
                                            span: span.clone(),
                                        };
                                        let new_expect = TypeSpec {
                                            hint: TypeHint::I32(None),
                                            span: span,
                                        };
                                        // Note: the spans don't really matter anymore

                                        let mut_main_mod = self.unit.get_mut(&self.entry);

                                        if mut_main_mod.is_some() {
                                            match mut_main_mod.unwrap() {
                                                NameSpace::Singular { name: _, file } => {
                                                    file.main_fn = Some(TopLevel::Entry {
                                                        ident,
                                                        expect: new_expect,
                                                        worker_usage,
                                                        block: new_block,
                                                        info: entry_info.clone(),
                                                    })
                                                }
                                                _ => (),
                                            }
                                        }
                                    }
                                    _ => panic!("Void didn't have block expr"),
                                }
                            }
                            oth => {
                                if type_checker::is_num_type(&oth) {
                                    match &oth {
                                        TypeSpec {
                                            hint: TypeHint::F32(_),
                                            span,
                                        }
                                        | TypeSpec {
                                            hint: TypeHint::F64(_),
                                            span,
                                        } => {
                                            self.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "Unexpected Type: This type cannot be converted into i32 which is the exit code type for OS runtime"
                                                )],
                                                hints: vec![],
                                                span: span.clone(),
                                            });
                                        }
                                        _ => {
                                            blocklevel_analyser::check_for_val_compatibility(
                                                block.clone(),
                                                &blocklevel_analyser::Expect::Required(
                                                    expect.clone(),
                                                    DataState::Owner,
                                                ),
                                                &vec![],
                                                &vec![],
                                                self,
                                            );
                                        }
                                    }
                                } else {
                                    self.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Unexpected Type: This type cannot be converted into i32 which is the exit code type for OS runtime"
                                        )],
                                        hints: vec![],
                                        span: oth.span.clone(),
                                    });
                                }
                                // self.analyse_block(Expect::Required(expect.clone()), &block);
                            }
                        },
                        _ => (),
                    }
                }

                // Globals
                self.for_globals(globals);

                // Functions
                self.for_funcs(fns);

                // Custom Types
                self.for_custom_types(custom_tys);

                // methods
                self.for_methods(methods);

                self.for_os_target(os_n_s);
            }
        }
    }

    pub fn analyse(&mut self) {
        // SEMANTIC ANALYSES
        // Analyse the types first (Type checking)
        // Note: type_checker will not only be checking type compatibility but also resourse

        // Resolve the symbol table of all modules at the top level
        // This is done at ones to make importation of top elements easier
        name_resolver::register_mod_level_elems(self);

        // For each Mod
        for this_mod in self.unit.clone() {
            self.modl = this_mod.0.clone();
            let table = self.symbol_tables.get(&this_mod.0);

            if table.is_some() {
                let table = table.unwrap();
                self.symbol_table = table.to_owned();
                // self.symbol_table.push(HashMap::new()); // initializing the global scope
                // there is only the global scope at this stage

                match this_mod.1 {
                    NameSpace::Singular { name: _, file } => {
                        self.go_through_mod(&file);
                        self.symbol_table = vec![]; // clear for the cfg to have a fresh start
                                                    // And do it after borrrow checkings
                    }
                    _ => (),
                }
            } else {
                todo!("module not found in symbol table error")
            }
        }
    }
}
