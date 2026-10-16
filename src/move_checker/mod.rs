use crate::{
    compiler::SemanticError,
    parser::{BlockLevel, DataState, Ident, TopLevel, TypeHint, TypeSpec},
    scanner::Span,
    analyser::{Analyser, Element},
};

pub fn check_ty_copy_impl(ty: &TypeSpec, analyser: &mut Analyser) -> bool {
    match ty {
        TypeSpec { hint, span: _ } => match hint {
            TypeHint::Custom { import, ident, generics: _ } => {
                let cust = analyser.get_custom_type(import, ident, &vec![]);

                if cust.is_some() {
                    //
                    match cust.clone().unwrap() {
                        TopLevel::StructDecl {
                            ident: _,
                            generic_types: _,
                            vals: _,
                            traits,
                            public: _,
                            associateds: _,
                            bridges: _,
                            info: _,
                        }
                        | TopLevel::EnumDecl {
                            ident: _,
                            generic_types: _,
                            vals: _,
                            traits,
                            public: _,
                            associateds: _,
                            bridges: _,
                            info: _,
                        } => {
                            let mut ret = false;
                            for tr in &traits {
                                match tr {
                                    Ident { ident, span: _ } => {
                                        if *ident == String::from("Ìdàkọ") {
                                            ret = true;
                                        }
                                    }
                                }
                            }

                            ret
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => true,
        },
    }
}

pub fn move_var(ident: &Ident, move_kw_span: Span, analyser: &mut Analyser) {
    let mut i = analyser.symbol_table.len();
    let mut b_elem = BlockLevel::Foo;
    for scope in analyser.symbol_table.iter().rev() {
        i -= 1;
        let element = scope.get(&ident.ident);
        if element.is_some() {
            match element.clone().unwrap() {
                Element::Local(elem) => match elem {
                    BlockLevel::VarDecl {
                        interpret: _,
                        state: _,
                        ty: _,
                        name: _,
                        value: _,
                        mutable: _,
                        info: _,
                    } => {
                        b_elem = elem.clone();
                        break;
                    }
                    BlockLevel::UnInitVarDecl {
                        state: _,
                        ty: _,
                        name,
                        mutable: _,
                        info: _,
                    } => {
                        //
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("'{:?}' as not yet been initialized, therefore there is no data to move", ident.ident)],
                            hints: vec![],
                            span: name.span.clone(),
                        });
                        break;
                    }
                    BlockLevel::Param {
                        state: _,
                        ty: _,
                        name: _,
                        mutable: _,
                        info: _,
                    } => {
                        b_elem = elem.clone();
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        // i += 1;
    }

    match b_elem {
        BlockLevel::VarDecl {
            interpret,
            state: _,
            ty,
            name,
            value,
            mutable,
            info,
        } => {
            analyser.symbol_table[i].insert(
                name.ident.clone(),
                Element::Local(BlockLevel::VarDecl {
                    interpret,
                    state: DataState::Moved(move_kw_span),
                    ty,
                    name,
                    value,
                    mutable,
                    info,
                }),
            );
        }
        BlockLevel::Param {
            state: _,
            ty,
            name,
            mutable,
            info,
        } => {
            analyser.symbol_table[i].insert(
                name.ident.clone(),
                Element::Local(BlockLevel::Param {
                    state: DataState::Moved(move_kw_span),
                    ty,
                    name,
                    mutable,
                    info,
                }),
            );
        }
        _ => (),
    }
}
