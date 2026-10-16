// use std::collections::HashMap;

use crate::borrow_checker::ModCFG;
// use crate::borrow_checker::{BorrowChecker, Var};
use crate::analyser::{Analyser, Element};
// use crate::borrow_checker::{BorrowChecker, Elem};
use crate::compiler::{ErrorHint, Mod, NameSpace, SemanticError};
use crate::parser::{
    BlockLevel, CustomBlock, CustomLevel, DataState, Expr, ExprKind, Ident, ModPath, PlatformArm,
    StmtInfo, TopLevel,
};
use crate::scanner::{FullTok, Pos, Span, Token};
//

fn add_to_global_scope(
    ident: Ident,
    glo_elem: TopLevel,
    // span: &Span,
    analyser: &mut Analyser,
) {
    if analyser.symbol_table[0].contains_key(&ident.ident) {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is a clash of name; Another global element already as the name '{}'",
                ident.ident
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    } else {
        analyser.symbol_table[0].insert(ident.ident, Element::Global(glo_elem));
    }
}

fn check_cust_block_name_clash_with_outside(ident: Ident, analyser: &mut Analyser) {
    if analyser.symbol_table[0].contains_key(&ident.ident) {
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

fn confirm_take_stm(
    take: (FullTok, Option<Ident>),
    from: ModPath,
    lib: Option<Ident>,
    analyser: &mut Analyser,
) -> Ident {
    let mod_ident = match &from {
        ModPath::Ident(ident) => ident.clone(),
        ModPath::Str(string) => string.clone(),
        ModPath::FromLibFolder(f, m) => Ident {
            ident: format!("{}/{}", f.ident, m.ident),
            span: Span {
                st: f.span.st,
                en: m.span.en,
                file: f.span.file.clone(),
            },
        },
        ModPath::UnExpected => Ident {
            ident: format!("Unknown"),
            span: Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: analyser.modl.clone(),
            },
        },
    };
    let ident;
    if take.1.is_some() {
        ident = take.1.clone().unwrap();
    } else {
        match take.0.clone() {
            FullTok {
                kind: Token::Identifier(i),
                span,
            } => {
                ident = Ident { ident: i, span };
            }
            _ => {
                ident = Ident {
                    ident: String::from(""),
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: analyser.modl.clone(),
                    },
                };
            }
        }
    }
    // We need to verify if they exist
    let mut found = false;

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
        key = lib.clone().unwrap().ident;
    }

    let module = analyser.unit.get(&key);
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
                            associateds: _,
                            bridges: _,
                            info,
                        } => {
                            match take.0.clone().kind {
                                Token::Identifier(s) => {
                                    if s == ident.ident {
                                        // found it
                                        found = true;
                                        if !public.clone() {
                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "'{}', is not public",
                                                    ident.ident
                                                )],
                                                hints: vec![ErrorHint {
                                                    hints: vec![format!(
                                                        "It was imported from here"
                                                    )],
                                                    span: mod_ident.span.clone(),
                                                }],
                                                span: info.span.clone(),
                                            });
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
                            associateds: _,
                            bridges: _,
                            info: _,
                        } => {
                            match take.0.clone().kind {
                                Token::Identifier(s) => {
                                    if s == ident.ident {
                                        // found it
                                        found = true;
                                        if !*public {
                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "'{}', is not public",
                                                    ident.ident
                                                )],
                                                hints: vec![ErrorHint {
                                                    hints: vec![format!(
                                                        "It was imported from here"
                                                    )],
                                                    span: mod_ident.span.clone(),
                                                }],
                                                span: ident.span.clone(),
                                            });
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
            NameSpace::Plural { name: _, files: _ } => {}
            NameSpace::Lib {
                name: _,
                manifest: _,
                entry,
                src: _,
            } => {
                if lib.is_none() {
                    // this is obviously a lib
                    // which means we are importing from it's entry
                    match entry {
                        ModCFG {
                            globals: _,
                            platform_n_s: _,
                            takes: _,
                            traits: _,
                            workers: _,
                            statics: _,
                            custom_tys: _,
                            methods: _,
                            static_meths: _,
                            bridges: _,
                            main_fn: _,
                            funcs,
                        } => {
                            for func in funcs {
                                if *func.0 == ident.ident {
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!("Can't find module '{}'", mod_ident.ident)],
            hints: vec![],
            span: mod_ident.span.clone(),
        }); // for now
    }

    if !found {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is no '{}' at module '{}'",
                ident.ident, mod_ident.ident
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    }

    ident
}

pub fn register_mod_level_elems(analyser: &mut Analyser, the_mod: &Mod) {
    if analyser.modl != String::from("<gbùngbùn>") {
        // we don't want to add elems to where the were originally defined
        let zero_span = Span {
            st: Pos { column: 0, line: 0 },
            en: Pos { column: 0, line: 0 },
            file: analyser.modl.clone(),
        };

        // Import everything in <gbùngbùn> with mú statement
        let core_mod = analyser.unit.get(&String::from("<gbùngbùn>"));

        if core_mod.is_some() {
            let core_mod = core_mod.unwrap();

            match core_mod.clone() {
                NameSpace::Singular { name: _, file } => match file {
                    Mod {
                        mod_docs: _,
                        main_fn: _,
                        macro_defs,
                        fns,
                        platform_n_s,
                        takes: _,
                        globals,
                        traits,
                        workers,
                        statics,
                        custom_tys,
                        lines: _,
                        scope: 0,
                    } => {
                        for sta in &statics {
                            match sta.clone() {
                                TopLevel::StaticDef {
                                    ident,
                                    public,
                                    block: _,
                                    info,
                                } => {
                                    if public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(
                                                            ident.ident.clone(),
                                                        ),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs,
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for wor in &workers {
                            match wor.clone() {
                                TopLevel::WorkerDef {
                                    ident,
                                    public,
                                    info,
                                    ..
                                } => {
                                    if public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(
                                                            ident.ident.clone(),
                                                        ),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs,
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for glob in &globals {
                            match glob.clone() {
                                TopLevel::GlobalDecl {
                                    name, public, info, ..
                                } => {
                                    if public {
                                        add_to_global_scope(
                                            name.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(name.ident.clone()),
                                                        span: name.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: name.span.clone(),
                                                    docs: info.docs,
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for func in &fns {
                            match func.clone() {
                                TopLevel::FuncDecl {
                                    ident,
                                    is_public,
                                    info,
                                    ..
                                } => {
                                    if is_public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(
                                                            ident.ident.clone(),
                                                        ),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs,
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for plat in &platform_n_s {
                            match plat.clone() {
                                TopLevel::PlatformNS { arms, info: _ } => {
                                    // Firstly: we need to get the functions that has síta flag
                                    if arms.len() > 0 {
                                        match &arms[0] {
                                            PlatformArm {
                                                target: _,
                                                block,
                                                span: _,
                                            } => match &block {
                                                CustomBlock { funcs, .. } => {
                                                    for func in funcs {
                                                        match func {
                                                            CustomLevel::FuncDecl {
                                                                ident,
                                                                is_visible,
                                                                info,
                                                                ..
                                                            } => {
                                                                if *is_visible {
                                                                    add_to_global_scope(
                                                                        ident.clone(),
                                                                        TopLevel::TakeStmt {
                                                                            take: (
                                                                                FullTok {
                                                                                    kind: Token::Identifier(ident.ident.clone()),
                                                                                    span: ident.span.clone(),
                                                                                },
                                                                                None,
                                                                            ),
                                                                            from: ModPath::Ident(Ident {
                                                                                ident: String::from("<gbùngbùn>"),
                                                                                span: zero_span.clone(),
                                                                            }),
                                                                            lib: None,
                                                                            info: StmtInfo { span: ident.span.clone(), docs: info.docs.clone() },
                                                                        },
                                                                        analyser,
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

                        for mac in &macro_defs {
                            match mac.clone() {
                                TopLevel::MacroDef {
                                    ident,
                                    is_public,
                                    cases: _,
                                    info,
                                } => {
                                    if is_public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::MacroCall(ident.ident),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs.clone(),
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for trs in &traits {
                            match trs {
                                TopLevel::TraitDef {
                                    ident,
                                    generic_types: _,
                                    interprets: _,
                                    methods: _,
                                    is_public,
                                    info,
                                } => {
                                    if *is_public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(
                                                            ident.ident.clone(),
                                                        ),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs.clone(),
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }

                        for cust in &custom_tys {
                            match cust {
                                TopLevel::EnumDecl {
                                    ident,
                                    generic_types: _,
                                    vals: _,
                                    traits: _,
                                    public,
                                    associateds: _,
                                    bridges: _,
                                    info,
                                }
                                | TopLevel::StructDecl {
                                    ident,
                                    generic_types: _,
                                    vals: _,
                                    traits: _,
                                    public,
                                    associateds: _,
                                    bridges: _,
                                    info,
                                } => {
                                    if *public {
                                        add_to_global_scope(
                                            ident.clone(),
                                            TopLevel::TakeStmt {
                                                take: (
                                                    FullTok {
                                                        kind: Token::Identifier(
                                                            ident.ident.clone(),
                                                        ),
                                                        span: ident.span.clone(),
                                                    },
                                                    None,
                                                ),
                                                from: ModPath::Ident(Ident {
                                                    ident: String::from("<gbùngbùn>"),
                                                    span: zero_span.clone(),
                                                }),
                                                lib: None,
                                                info: StmtInfo {
                                                    span: ident.span.clone(),
                                                    docs: info.docs.clone(),
                                                },
                                            },
                                            analyser,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                },
                _ => (), // core is singular
            }
        } else {
            //
        }
    }

    // for cust types in mod
    for each in &the_mod.custom_tys {
        match each {
            TopLevel::EnumDecl {
                ident,
                generic_types: _,
                vals: _,
                traits: _,
                public: _,
                associateds: _,
                bridges: _,
                info: _,
            }
            | TopLevel::StructDecl {
                ident,
                generic_types: _,
                vals: _,
                traits: _,
                public: _,
                associateds: _,
                bridges: _,
                info: _,
            } => {
                add_to_global_scope(ident.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    // now register the variables
    for each in &the_mod.globals {
        match each {
            TopLevel::GlobalDecl {
                interpret: _,
                state,
                ty: _,
                name, // should come with span, for precise error handling
                value,
                public: _,
                info: _,
            } => {
                match state {
                    DataState::GlobalImutRef => match value {
                        Expr {
                            kind: ExprKind::Ident { affix: _, ident: _ },
                            span: _,
                        } => (),
                        _ => {
                            analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Global ImutRef variable '{}' must be assigned to another variable",
                                        name.ident
                                    )],
                                    hints: vec![],
                                    span: name.span.clone(),
                                });
                        }
                    },
                    DataState::GlobalMutRef => match value {
                        Expr {
                            kind: ExprKind::Ident { affix: _, ident: _ },
                            span: _,
                        } => (),
                        _ => {
                            analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Global MutRef variable '{}' must be assigned to another variable",
                                        name.ident
                                    )],
                                    hints: vec![],
                                    span: name.span.clone(),
                                });
                        }
                    },
                    _ => (),
                }
                add_to_global_scope(name.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    // For Global functions
    for each in &the_mod.fns {
        match each {
            TopLevel::FuncDecl {
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
                add_to_global_scope(ident.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    // For takes
    for each in &the_mod.takes {
        match each.clone() {
            TopLevel::TakeStmt {
                take,
                from,
                lib,
                info: _,
            } => {
                let ident = confirm_take_stm(take, from, lib, analyser);

                add_to_global_scope(ident, each.clone(), analyser);
            }
            _ => (),
        }
    }

    // For traits
    for each in &the_mod.traits {
        match each {
            TopLevel::TraitDef {
                ident,
                generic_types: _,
                interprets: _,
                methods: _,
                is_public: _,
                info: _,
            } => {
                add_to_global_scope(ident.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    // For workers
    for each in &the_mod.workers {
        match each {
            TopLevel::WorkerDef {
                ident,
                func_name: _,
                public: _,
                block: _,
                info: _,
            } => {
                add_to_global_scope(ident.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    // For bundles
    for each in &the_mod.statics {
        match each {
            TopLevel::StaticDef {
                ident,
                public: _,
                block: _,
                info: _,
            } => {
                add_to_global_scope(ident.clone(), each.clone(), analyser);
            }
            _ => (),
        }
    }

    for each in &the_mod.platform_n_s {
        match each {
            TopLevel::PlatformNS { arms, info: _ } => {
                // We need to check if the names of elements in each arms
                // does not clash with the ones defined outside
                for arm in arms {
                    match &arm.block {
                        CustomBlock {
                            takes,
                            globals,
                            macro_def: _,
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
                                        lib,
                                        info: _,
                                    } => {
                                        let ident = confirm_take_stm(take, from, lib, analyser);

                                        check_cust_block_name_clash_with_outside(ident, analyser);
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
                                        associateds: _,
                                        bridges: _,
                                        info: _,
                                    }
                                    | CustomLevel::StructDecl {
                                        ident,
                                        generic_types: _,
                                        vals: _,
                                        traits: _,
                                        public: _,
                                        associateds: _,
                                        bridges: _,
                                        info: _,
                                    } => {
                                        check_cust_block_name_clash_with_outside(ident.clone(), analyser);
                                    }
                                    _ => (),
                                }
                            }

                            // now register the variables
                            for each in globals {
                                match each {
                                    CustomLevel::GlobalDecl {
                                        interpret: _,
                                        state,
                                        ty: _,
                                        name, // should come with span, for precise error handling
                                        value,
                                        public: _,
                                        info: _,
                                    } => {
                                        match state {
                                            DataState::GlobalImutRef => match value {
                                                Expr {
                                                    kind: ExprKind::Ident { affix: _, ident: _ },
                                                    span: _,
                                                } => (),
                                                _ => {
                                                    analyser.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Global ImutRef variable '{}' must be assigned to another variable",
                                                            name.ident
                                                        )],
                                                        hints: vec![],
                                                        span: name.span.clone(),
                                                    });
                                                }
                                            },
                                            DataState::GlobalMutRef => match value {
                                                Expr {
                                                    kind: ExprKind::Ident { affix: _, ident: _ },
                                                    span: _,
                                                } => (),
                                                _ => {
                                                    analyser.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Global MutRef variable '{}' must be assigned to another variable",
                                                            name.ident
                                                        )],
                                                        hints: vec![],
                                                        span: name.span.clone(),
                                                    });
                                                }
                                            },
                                            _ => (),
                                        }
                                        check_cust_block_name_clash_with_outside(name.clone(), analyser);
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
                                        check_cust_block_name_clash_with_outside(ident.clone(), analyser);
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
                                        interprets: _,
                                        methods: _,
                                        is_public: _,
                                        info: _,
                                    } => {
                                        check_cust_block_name_clash_with_outside(ident.clone(), analyser);
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
                                        check_cust_block_name_clash_with_outside(ident.clone(), analyser);
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
                                        check_cust_block_name_clash_with_outside(ident.clone(), analyser);
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
                        PlatformArm {
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
                                            block,
                                            info,
                                        } => {
                                            if is_visible {
                                                add_to_global_scope(
                                                    ident.clone(),
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
                                                        block: Expr {
                                                            // placeholder, just for analyses
                                                            //codegen will eventualy use the the correct block/expr
                                                            kind: ExprKind::Foo,
                                                            span: block.span,
                                                        },
                                                        info,
                                                    },
                                                    analyser,
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

// pub fn register_global_for_borrorw_checker(checker: &mut BorrowChecker, the_mod: &Mod) {
//     // there is no need to recheck for name clashes
//     let mut storage = HashMap::new();

//     // first register all global variables into storage
//     for each in &the_mod.globals {
//         match each {
//             TopLevel::GlobalDecl {
//                 interpret: _,
//                 state,
//                 ty: _,
//                 name, // should come with span, for precise error handling
//                 value,
//                 public: _,
//                 span: _,
//             } => {
//                 match state {
//                     DataState::Global => {
//                         storage.insert(
//                             name.ident.clone(),
//                             Var::Owner {
//                                 imut_borrs: vec![],
//                                 mut_borr: None,
//                                 ident_span: name.span.clone(),
//                             },
//                         );
//                     }
//                     DataState::GlobalImutRef => {
//                         match value {
//                             Expr { kind: ExprKind::Ident { affix: _, ident }, span: _ } => {
//                                 // most be ident
//                                 let var = storage.get(&ident.ident);

//                                 if var.is_some() {
//                                     match var.unwrap() {
//                                         Var::Owner {
//                                             imut_borrs: _,
//                                             mut_borr: _,
//                                             ident_span: _,
//                                         } => {
//                                             storage.insert(
//                                                 name.ident.clone(),
//                                                 Var::Borrower {
//                                                     owner: Some(ident.clone()), // for now
//                                                     assigned: ident.clone(),
//                                                     its_mut: false,
//                                                     ident_span: name.span.clone(),
//                                                 },
//                                             );
//                                         }
//                                         Var::Borrower {
//                                             owner,
//                                             assigned: _,
//                                             its_mut: _,
//                                             ident_span: _,
//                                         } => {
//                                             if owner.is_some() {
//                                                 storage.insert(
//                                                     name.ident.clone(),
//                                                     Var::Borrower {
//                                                         owner: Some(owner.clone().unwrap()),
//                                                         assigned: ident.clone(),
//                                                         its_mut: false,
//                                                         ident_span: name.span.clone(),
//                                                     },
//                                                 );
//                                             } else {
//                                                 storage.insert(
//                                                     name.ident.clone(),
//                                                     Var::Borrower {
//                                                         owner: None, // for now
//                                                         assigned: ident.clone(),
//                                                         its_mut: false,
//                                                         ident_span: name.span.clone(),
//                                                     },
//                                                 );
//                                             }
//                                         }
//                                     }
//                                 } else {
//                                     storage.insert(
//                                         name.ident.clone(),
//                                         Var::Borrower {
//                                             owner: None, // for now
//                                             assigned: ident.clone(),
//                                             its_mut: false,
//                                             ident_span: name.span.clone(),
//                                         },
//                                     );
//                                 }
//                             }
//                             _ => (),
//                         }
//                     }
//                     DataState::GlobalMutRef => {
//                         match value {
//                             Expr { kind: ExprKind::Ident { affix: _, ident }, span: _ } => {
//                                 // most be ident
//                                 let var = storage.get(&ident.ident);

//                                 if var.is_some() {
//                                     match var.unwrap() {
//                                         Var::Owner {
//                                             imut_borrs: _,
//                                             mut_borr: _,
//                                             ident_span: _,
//                                         } => {
//                                             storage.insert(
//                                                 name.ident.clone(),
//                                                 Var::Borrower {
//                                                     owner: Some(ident.clone()), // for now
//                                                     assigned: ident.clone(),
//                                                     its_mut: true,
//                                                     ident_span: name.span.clone(),
//                                                 },
//                                             );
//                                         }
//                                         Var::Borrower {
//                                             owner,
//                                             assigned: _,
//                                             its_mut: _,
//                                             ident_span: _,
//                                         } => {
//                                             if owner.is_some() {
//                                                 storage.insert(
//                                                     name.ident.clone(),
//                                                     Var::Borrower {
//                                                         owner: Some(owner.clone().unwrap()),
//                                                         assigned: ident.clone(),
//                                                         its_mut: true,
//                                                         ident_span: name.span.clone(),
//                                                     },
//                                                 );
//                                             } else {
//                                                 storage.insert(
//                                                     name.ident.clone(),
//                                                     Var::Borrower {
//                                                         owner: None, // for now
//                                                         assigned: ident.clone(),
//                                                         its_mut: true,
//                                                         ident_span: name.span.clone(),
//                                                     },
//                                                 );
//                                             }
//                                         }
//                                     }
//                                 } else {
//                                     storage.insert(
//                                         name.ident.clone(),
//                                         Var::Borrower {
//                                             owner: None, // for now
//                                             assigned: ident.clone(),
//                                             its_mut: true,
//                                             ident_span: name.span.clone(),
//                                         },
//                                     );
//                                 }
//                             }
//                             _ => (),
//                         }
//                     }
//                     _ => (),
//                 }
//             }
//             _ => (),
//         }
//     }

//     // We need to make sure thet the owner of each ref is identified

//     enum Attendance {
//         Aye,
//     }

//     loop {
//         let mut unresolved = vec![];

//         for each in storage.clone() {
//             match each.1 {
//                 Var::Borrower {
//                     owner,
//                     assigned,
//                     its_mut: _,
//                     ident_span: _,
//                 } => {
//                     if owner.is_none() {
//                         let ass = storage.get(&assigned.ident);

//                         if ass.is_some() {
//                             // should be there already
//                             match ass.unwrap().clone() {
//                                 Var::Borrower {
//                                     owner,
//                                     assigned,
//                                     its_mut,
//                                     ident_span,
//                                 } => {
//                                     if owner.is_some() {
//                                         storage.insert(
//                                             each.0.clone(),
//                                             Var::Borrower {
//                                                 owner,
//                                                 assigned,
//                                                 its_mut,
//                                                 ident_span,
//                                             },
//                                         );
//                                     } else {
//                                         unresolved.push(Attendance::Aye);
//                                     }
//                                 }
//                                 _ => (),
//                             }
//                         } else {
//                             panic!("Unexpected execution");
//                         }
//                     }
//                 }
//                 _ => (),
//             }
//         }

//         if unresolved.len() == 0 {
//             break;
//         }
//     }

//     // Now we register all the global variable into checker's symbol table
//     // but now knowning the owner of all the borrowers
//     for each in &storage {
//         checker.symbol_table[0].insert(each.0.clone(), each.1.clone());
//     }

//     // println!("{:#?}", checker.symbol_table[0].get(&format!("orò")));
// }

pub fn add_to_local_scope(
    ident: Ident,
    elem: BlockLevel,
    // span: &Span,
    analyser: &mut Analyser,
) {
    let index = analyser.symbol_table.len() - 1;

    if analyser.symbol_table[index].contains_key(&ident.ident) {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is a clash of name; Another local element already as the name '{}'",
                ident.ident.clone()
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    } else {
        analyser.symbol_table[index].insert(ident.ident, Element::Local(elem));
    }
}
