// use std::collections::HashMap;

use std::collections::HashMap;

use crate::borrow_checker::MethodCfg;
// use crate::borrow_checker::{BorrowChecker, Var};
use crate::analyser::Analyser;
// use crate::borrow_checker::{BorrowChecker, Elem};
use crate::compiler::{Mod, NameSpace, SemanticError};
use crate::parser::{
    BridgeDecl, CustTyAss, CustomBlock, CustomLevel, DataState, EnumVariants, Generic, Ident,
    InterfaceParam, MacroGroup, Method, MethodSelf, OsArm, StmtInfo, StructParam, TopLevel,
    TraitImpl, Trinary, TypeKey, TypeSpec,
};
use crate::scanner::Span;
//

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ElemSignature {
    DocLines {
        lines: Vec<String>,
        span: Span,
    },
    ModlUse {
        alias: Ident,
        span: Span,
    },
    LibUse {
        ident: Ident,
        span: Span,
    },
    ModlTake {
        name: Ident,
        span: Span,
    },
    // TakeStmt {
    //     elem: Box<ElemSignature>,
    //     alias: Option<Ident>,
    //     from: Vec<Ident>,
    //     info: StmtInfo,
    // },
    GlobalDecl {
        interpret: bool,
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        public: bool,
        span: Span,
    },
    EnumDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<EnumVariants>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        span: Span,
    },
    StructDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<StructParam>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        span: Span,
    },
    GenericType {
        name: Ident,
        traits: Vec<Ident>,
    },
    Param {
        // into a function block
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        mutable: bool,
        info: StmtInfo,
    },
    FuncDecl {
        is_async: bool,
        expect: TypeSpec,
        ret_state: DataState,
        ident: Ident,
        generics: Vec<Generic>,
        is_visible: bool,
        is_public: bool,
        is_extern: Option<(Ident, Option<Span>)>,
        par: Vec<InterfaceParam>,
        worker_usage: Vec<(Ident, Vec<Ident>)>,
        span: Span,
    },
    Bridges(HashMap<String, HashMap<TypeKey, BridgeDecl>>),
    Methods(HashMap<TypeKey, HashMap<String, CustTyAss>>),
    TraitImpls(HashMap<TypeKey, HashMap<String, TraitImpl>>),
    PlatformNS {
        arms: Vec<OsArm>,
        span: Span,
    },
    WorkerDef {
        ident: Ident,
        func_name: Ident,
        public: bool,
        span: Span,
    },
    StaticDef {
        ident: Ident,
        public: bool,
        span: Span,
    },
    TraitDef {
        ident: Ident,
        generic_types: Option<Ident>,
        // interprets: Vec<BlockLevel>,
        methods: HashMap<String, MethodCfg>,
        is_public: bool,
        span: Span,
    },
    WorkerUsage {
        workers: Vec<Ident>,
        span: Span,
    },
    VarDecl {
        interpret: bool,
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        mutable: bool,
        span: Span,
    },
    UnInitVarDecl {
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        mutable: bool,
        span: Span,
    },
    ProjectLevel {},
    Foo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct MacroCaseSignature {
    pub pattern: MacroGroup,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum CustTyAssSignature {
    Method(MethSignature),
    AssFunction(AssFuncSignature),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct AssFuncSignature {
    pub worker_usage: Vec<Ident>,
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub ident: Ident,
    pub generics: Vec<Generic>,
    pub is_public: bool,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub par: Vec<InterfaceParam>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct MethSignature {
    pub worker_usage: Vec<Ident>,
    pub meth_self: (MethodSelf, Span),
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub ident: Ident,
    pub generics: Option<Vec<Generic>>,
    pub is_public: Trinary,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub par: Vec<InterfaceParam>,
    pub span: Span,
}

fn add_to_global_scope(
    ident: Ident,
    glo_elem: ElemSignature,
    modl: &String,
    analyser: &mut Analyser,
) {
    let symb_table = analyser.symbol_tables.get_mut(modl);

    if symb_table.is_some() {
        let symb = symb_table.unwrap();
        if symb[0].contains_key(&ident.ident) {
            analyser.errs.push(SemanticError {
                messages: vec![format!(
                    "There is a clash of name; Another global element already as the name '{}'",
                    ident.ident
                )],
                hints: vec![],
                span: ident.span.clone(),
            });
        } else {
            symb[0].insert(ident.ident, glo_elem);
        }
    }
}

fn check_cust_block_name_clash_with_outside(ident: Ident, modl: &String, analyser: &mut Analyser) {
    let symb_table = analyser.symbol_tables.get(modl);

    if symb_table.is_some() {
        let symb = symb_table.unwrap();
        if symb[0].contains_key(&ident.ident) {
            analyser.errs.push(SemanticError {
                messages: vec![format!(
                    "There is a clash of name; Another global element already as the name '{}'",
                    ident.ident
                )],
                hints: vec![],
                span: ident.span.clone(),
            });
        }
    }
}

fn scan_for_elem(file: &Mod, get: Ident, analyser: &mut Analyser) -> bool {
    let mut found = false;
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
                if get.ident == ident.ident {
                    // found it
                    found = true;
                    if !public.clone() {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("'{}', is not public", ident.ident)],
                            hints: vec![],
                            span: get.span.clone(),
                        });
                    }
                    break;
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
                if get.ident == ident.ident {
                    // found it
                    found = true;
                    if !*public {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("'{}', is not public", ident.ident)],
                            hints: vec![],
                            span: get.span.clone(),
                        });
                    }
                    break;
                }
            }
            _ => (),
        }
    }

    found
}

fn confirm_take_stm(
    take: (Ident, Option<Ident>),
    from: Vec<Ident>, // len() of this should be at least == 1
    analyser: &mut Analyser,
) -> Ident {
    // this particular function is vulnarable when from is empty
    // so the parser needs to make sure that it's not
    // let get = take.0;
    let ident;
    if take.1.is_some() {
        ident = take.1.clone().unwrap();
    } else {
        ident = take.0
    }
    // We need to verify if they exist
    let mut found = false;

    // let key;
    // if lib == None {
    //     match from {
    //         ModPath::Ident(ident) => {
    //             key = ident.ident;
    //         }
    //         _ => {
    //             key = String::from("");
    //         }
    //     }
    // } else {
    //     key = lib.clone().unwrap().ident;
    // }

    // let is_proj_lev = analyser.lib_list.contains(&from[0].ident);

    let import = analyser.unit.get(&from[0].ident);
    if import.is_some() {
        let import = import.unwrap().clone();
        match import {
            NameSpace::Singular { name: _, file } => {
                // when it's a local singular module
                // then from can't have more than 2 Idents
                if from.len() == 1 {
                    // let frm = from.last().unwrap();
                    found = scan_for_elem(&file, ident.clone(), analyser);
                }
            }
            NameSpace::SecEntry {
                name: _,
                singulars: _,
                file: _,
            } => {}
            NameSpace::Lib {
                name: _,
                manifest: _,
                entry: _,
                ackn_panic: _,
                src: _,
            } => {

                // analyser.errs.push(SemanticError {
                //     messages: vec![format!("An attempt to take from an element that belongs here")],
                //     hints: vec![],
                //     span: mod_ident.span.clone(),
                // });
            }
        }
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!("Can't find module '{}'", from[0].ident)],
            hints: vec![],
            span: from[0].span.clone(),
        }); // for now
    }

    if !found {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is no '{}' at module '{}'",
                ident.ident,
                from.last().unwrap().ident
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    }

    ident
}

pub fn register_mod(name: &String, file: &Mod, analyser: &mut Analyser) {
    match file {
        Mod {
            mods,
            mod_docs: _,
            main_fn: _,
            panic_fn,
            // macro_defs: _,
            fns,
            os_n_s,
            arch_n_s: _,
            takes,
            globals,
            traits,
            workers,
            statics,
            custom_tys,
            bridges: _,
            methods: _,
            trait_impls: _,
            lines: _,
            scope: _,
        } => {
            // for each use stm
            for each in mods {
                match each {
                    TopLevel::UseStmt {
                        path: _,
                        is_sec_entry: _,
                        alias,
                        is_public: _,
                        info,
                    } => {
                        let elem = ElemSignature::ModlUse {
                            alias: alias.clone(),
                            span: info.span.clone(),
                        };
                        add_to_global_scope(alias.clone(), elem, name, analyser);
                    }
                    TopLevel::UseLibStmt {
                        include,
                        ackn_panic: _,
                        info,
                    } => {
                        let elem = ElemSignature::LibUse {
                            ident: include.clone(),
                            span: info.span.clone(),
                        };
                        add_to_global_scope(include.clone(), elem, name, analyser);
                    }
                    _ => (),
                }
            }

            // for takes

            for each in takes {
                match each {
                    TopLevel::TakeStmt { take, from, info } => {
                        // from has at least one element
                        if analyser.lib_list.contains(&from[0].ident)
                            || analyser.mod_list.contains(&from[0].ident)
                        {
                            let namespace = analyser.unit.get(&from[0].ident);

                            match namespace {
                                Some(NameSpace::Singular { name: _, file }) => {
                                    if from.len() > 1 {
                                        
                                    } else {}
                                }
                                Some(NameSpace::SecEntry {
                                    name,
                                    singulars,
                                    file,
                                }) => {}
                                Some(NameSpace::Lib {
                                    name,
                                    manifest,
                                    entry,
                                    ackn_panic,
                                    src,
                                }) => {}
                                None => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "'{}' not found in compilation unit",
                                            from[0].ident,
                                        )],
                                        hints: vec![],
                                        span: from[0].span.clone(),
                                    });
                                }
                            }
                            let elem = ElemSignature::ModlTake {
                                name: from[0].clone(),
                                span: info.span.clone(),
                            };
                            add_to_global_scope(from[0].clone(), elem, name, analyser);
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "There is no library or module with'{}' in this unit'{}'",
                                    take.0.ident,
                                    from.last().unwrap().ident
                                )],
                                hints: vec![],
                                span: take.0.span.clone(),
                            });
                        }
                    }
                    _ => (),
                }
            }

            // for panic function in mod
            if panic_fn.is_some() {
                match panic_fn.clone().unwrap() {
                    TopLevel::Panic {
                        ident,
                        expect,
                        worker_usage,
                        generics,
                        public,
                        params,
                        block: _,
                        info,
                    } => {
                        // if expect.hint != TypeHint::End {
                        //     analyser.errs.push(SemanticError {
                        //         messages: vec![format!(
                        //             "The panic function most have a return type of òpin"
                        //         )],
                        //         hints: vec![],
                        //         span: ident.span,
                        //     });
                        // }

                        // if params.len() != 2 {
                        //     analyser.errs.push(SemanticError {
                        //         messages: vec![format!(
                        //             "Panic expects 2 parameters but {} was found".
                        //             params.len()
                        //         )],
                        //         hints: vec![/* todo */],
                        //         span: ident.span,
                        //     });
                        // } else {
                        //     match params[0] {
                        //         // str type
                        //         InterfaceParam {
                        //             ident: _,
                        //             state,
                        //             ty,
                        //             is_mut,
                        //             span,
                        //         } => {
                        //             match state {
                        //                 DataState::ImutRef
                        //             }
                        //         }
                        //     }
                        // }
                        let elem = ElemSignature::FuncDecl {
                            is_async: false,
                            expect,
                            ret_state: DataState::Owner,
                            ident: ident.clone(),
                            generics,
                            is_visible: false,
                            is_public: public,
                            is_extern: None,
                            par: params,
                            worker_usage,
                            span: info.span.clone(),
                        };
                        add_to_global_scope(ident.clone(), elem, name, analyser);
                    }
                    _ => (),
                }
            }
            // for cust types in mod
            for each in custom_tys {
                match each.clone() {
                    TopLevel::EnumDecl {
                        ident,
                        generic_types,
                        vals,
                        traits,
                        public,
                        info,
                    } => {
                        // let mut ass: HashMap<String, CustTyAss> = HashMap::new();
                        // for asso in associateds {
                        //     match asso {
                        //         CustTyAss::AssFunction(ass_func) => {
                        //             ass.insert(
                        //                 ass_func.ident.ident.clone(),
                        //                 CustTyAss::AssFunction(ass_func),
                        //             );
                        //         }
                        //         CustTyAss::Method(meth) => {
                        //             ass.insert(meth.ident.ident.clone(), CustTyAss::Method(meth));
                        //         }
                        //     }
                        // }
                        // let mut brgs = HashMap::new();
                        // for brg in bridges {
                        //     let type_string = format!("{}", brg.ass_type.hint);
                        //     brgs.insert(type_string, brg);
                        // }

                        let elem = ElemSignature::EnumDecl {
                            ident: ident.clone(),
                            generic_types,
                            vals,
                            traits,
                            public,
                            span: info.span.clone(),
                        };
                        add_to_global_scope(ident, elem, name, analyser);
                    }
                    TopLevel::StructDecl {
                        ident,
                        generic_types,
                        vals,
                        traits,
                        public,
                        info,
                    } => {
                        // let mut ass: HashMap<String, CustTyAss> = HashMap::new();
                        // for asso in associateds {
                        //     match asso {
                        //         CustTyAss::AssFunction(ass_func) => {
                        //             ass.insert(
                        //                 ass_func.ident.ident.clone(),
                        //                 CustTyAss::AssFunction(ass_func),
                        //             );
                        //         }
                        //         CustTyAss::Method(meth) => {
                        //             ass.insert(meth.ident.ident.clone(), CustTyAss::Method(meth));
                        //         }
                        //     }
                        // }
                        // let mut brgs = HashMap::new();
                        // for brg in bridges {
                        //     let type_string = format!("{}", brg.ass_type.hint);
                        //     brgs.insert(type_string, brg);
                        // }

                        let elem = ElemSignature::StructDecl {
                            ident: ident.clone(),
                            generic_types,
                            vals,
                            traits,
                            public,
                            span: info.span.clone(),
                        };
                        add_to_global_scope(ident, elem, name, analyser);
                    }
                    _ => (),
                }
            }

            // now register the variables
            for each in globals {
                match each.clone() {
                    TopLevel::GlobalDecl {
                        interpret,
                        state,
                        ty,
                        name: ident, // should come with span, for precise error handling
                        value: _,
                        public,
                        info,
                    } => {
                        let elem = ElemSignature::GlobalDecl {
                            interpret,
                            state,
                            ty,
                            name: ident.clone(),
                            public,
                            span: info.span.clone(),
                        };
                        add_to_global_scope(ident.clone(), elem, name, analyser);
                    }
                    _ => (),
                }
            }

            // For Global functions
            for each in fns {
                match each.clone() {
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
                        block: _,
                        info,
                    } => {
                        let elem = ElemSignature::FuncDecl {
                            is_async,
                            expect,
                            ret_state,
                            ident: ident.clone(),
                            generics,
                            is_visible,
                            is_public,
                            is_extern,
                            par,
                            worker_usage,
                            span: info.span.clone(),
                        };
                        add_to_global_scope(ident.clone(), elem, name, analyser);
                    }
                    _ => (),
                }
            }

            // For traits
            for each in traits {
                match each.clone() {
                    TopLevel::TraitDef {
                        ident,
                        generic_types,
                        methods,
                        is_public,
                        info,
                    } => {
                        let mut meth_cfg = HashMap::new();
                        for meth in methods {
                            match meth {
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
                                    block: _,
                                    info,
                                } => {
                                    meth_cfg.insert(
                                        ident.ident.clone(),
                                        MethodCfg {
                                            ident_span: ident.span,
                                            worker_usage,
                                            meth_self,
                                            expect,
                                            ret_state,
                                            generics,
                                            is_extern,
                                            is_public,
                                            par,
                                            block: None,
                                            stm_span: info.span,
                                        },
                                    );
                                }
                            }
                        }
                        let elem = ElemSignature::TraitDef {
                            ident: ident.clone(),
                            generic_types,
                            // interprets,
                            methods: meth_cfg,
                            is_public,
                            span: info.span,
                        };
                        add_to_global_scope(ident, elem, &name, analyser);
                    }
                    _ => (),
                }
            }

            // For workers
            for each in workers {
                match each.clone() {
                    TopLevel::WorkerDef {
                        ident,
                        func_name,
                        public,
                        block: _,
                        info,
                    } => {
                        let elem = ElemSignature::WorkerDef {
                            ident: ident.clone(),
                            func_name,
                            public,
                            span: info.span,
                        };
                        add_to_global_scope(ident, elem, &name, analyser);
                    }
                    _ => (),
                }
            }

            // For bundles
            for each in statics {
                match each.clone() {
                    TopLevel::StaticDef {
                        ident,
                        public,
                        block: _,
                        info,
                    } => {
                        let elem = ElemSignature::StaticDef {
                            ident: ident.clone(),
                            public,
                            span: info.span,
                        };
                        add_to_global_scope(ident, elem, &name, analyser);
                    }
                    _ => (),
                }
            }

            for each in os_n_s {
                match each {
                    TopLevel::OsTarget { arms, info: _ } => {
                        // We need to check if the names of elements in each arms
                        // does not clash with the ones defined outside
                        for arm in arms {
                            match &arm.block {
                                CustomBlock {
                                    takes,
                                    globals,
                                    // macro_def: _,
                                    custom_tys,
                                    traits,
                                    workers,
                                    funcs,
                                    platform_n_s: _,
                                    static_defs,
                                } => {
                                    // For takes
                                    for each in takes {
                                        match each.clone() {
                                            CustomLevel::TakeStmt {
                                                take,
                                                from,
                                                info: _,
                                            } => {
                                                let ident = confirm_take_stm(take, from, analyser);

                                                check_cust_block_name_clash_with_outside(
                                                    ident, &name, analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // for cust types in mod
                                    for each in custom_tys {
                                        match each {
                                            CustomLevel::EnumDecl {
                                                ident,
                                                generic_types: _,
                                                vals: _,
                                                traits: _,
                                                public: _,
                                                info: _,
                                            }
                                            | CustomLevel::StructDecl {
                                                ident,
                                                generic_types: _,
                                                vals: _,
                                                traits: _,
                                                public: _,
                                                info: _,
                                            } => {
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // now register the variables
                                    for each in globals {
                                        match each {
                                            CustomLevel::GlobalDecl {
                                                name: ident, // should come with span, for precise error handling
                                                ..
                                            } => {
                                                // match state {
                                                //     DataState::Ref => match value {
                                                //         Expr {
                                                //             kind:
                                                //                 ExprKind::Ident { affix: _, ident: _ },
                                                //             span: _,
                                                //         } => (),
                                                //         _ => {
                                                //             analyser.errs.push(SemanticError {
                                                //                                 messages: vec![format!(
                                                //                                     "Global ImutRef variable '{}' must be assigned to another variable",
                                                //                                     ident.ident
                                                //                                 )],
                                                //                                 hints: vec![],
                                                //                                 span: ident.span.clone(),
                                                //                             });
                                                //         }
                                                //     },
                                                //     DataState::GlobalMutRef => match value {
                                                //         Expr {
                                                //             kind:
                                                //                 ExprKind::Ident { affix: _, ident: _ },
                                                //             span: _,
                                                //         } => (),
                                                //         _ => {
                                                //             analyser.errs.push(SemanticError {
                                                //                                 messages: vec![format!(
                                                //                                     "Global MutRef variable '{}' must be assigned to another variable",
                                                //                                     ident.ident
                                                //                                 )],
                                                //                                 hints: vec![],
                                                //                                 span: ident.span.clone(),
                                                //                             });
                                                //         }
                                                //     },
                                                //     _ => (),
                                                // }
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // For Global functions
                                    for each in funcs {
                                        match each {
                                            CustomLevel::FuncDecl {
                                                is_async: _,
                                                expect: _,
                                                ret_state: _,
                                                ident,
                                                generics: _,
                                                is_visible: _,
                                                is_public: _,
                                                is_extern: _,
                                                par: _,
                                                worker_usage: _,
                                                block: _,
                                                info: _,
                                            } => {
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // For traits
                                    for each in traits {
                                        match each {
                                            CustomLevel::TraitDef {
                                                ident,
                                                generic_types: _,
                                                // interprets: _,
                                                methods: _,
                                                is_public: _,
                                                info: _,
                                            } => {
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // For workers
                                    for each in workers {
                                        match each {
                                            CustomLevel::WorkerDef {
                                                ident,
                                                func_name: _,
                                                public: _,
                                                block: _,
                                                info: _,
                                            } => {
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    // For bundles
                                    for each in static_defs {
                                        match each {
                                            CustomLevel::StaticDef {
                                                ident,
                                                public: _,
                                                block: _,
                                                info: _,
                                            } => {
                                                check_cust_block_name_clash_with_outside(
                                                    ident.clone(),
                                                    &name,
                                                    analyser,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            }
                        }

                        // then we need to add outside function signatures
                        if arms.len() > 0 {
                            match arms[0].clone() {
                                OsArm {
                                    target: _,
                                    block,
                                    span: _,
                                } => match block {
                                    CustomBlock { funcs, .. } => {
                                        for func in funcs {
                                            match func.clone() {
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
                                                    block: _,
                                                    info,
                                                } => {
                                                    let elem = ElemSignature::FuncDecl {
                                                        is_async,
                                                        expect,
                                                        ret_state,
                                                        ident: ident.clone(),
                                                        generics,
                                                        is_visible,
                                                        is_public,
                                                        is_extern,
                                                        par,
                                                        worker_usage,
                                                        span: info.span,
                                                    };
                                                    if is_visible {
                                                        add_to_global_scope(
                                                            ident, elem, &name, analyser,
                                                        );
                                                    }
                                                }
                                                _ => (),
                                            }
                                        }
                                    }
                                },
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

pub fn register_mod_level_elems(analyser: &mut Analyser) {
    let unit = analyser.unit.clone();
    for modl in &unit {
        match modl.1 {
            NameSpace::Singular { name, file } => {
                // if !analyser.cli.no_dgd {
                //     // we want to add dgd(core lib) elems to the mod symbol table
                //     let zero_span = Span {
                //         st: Pos { column: 0, line: 0 },
                //         en: Pos { column: 0, line: 0 },
                //         file: analyser.modl.clone(),
                //     };
                // }
                register_mod(name, file, analyser);
            }
            NameSpace::SecEntry {
                name,
                singulars: _,
                file,
            } => {
                register_mod(name, file, analyser);
            }
            NameSpace::Lib {
                name: _,
                manifest: _,
                entry: _,
                ackn_panic: _,
                src: _,
            } => {
                // AST will be regenrated for toplevel elements here
                // which is just their signatures
            }
        }
    }
}

pub fn add_to_local_scope(
    ident: Ident,
    elem: ElemSignature,
    modl: &String,
    analyser: &mut Analyser,
) {
    let md = analyser.symbol_tables.get_mut(modl);

    if md.is_some() {
        let md = md.unwrap();
        let index = md.len() - 1;

        if md[index].contains_key(&ident.ident) {
            analyser.errs.push(SemanticError {
                messages: vec![format!(
                    "There is a clash of name; Another local element already as the name '{}'",
                    ident.ident.clone()
                )],
                hints: vec![],
                span: ident.span.clone(),
            });
        } else {
            md[index].insert(ident.ident, elem);
        }
    }
}
