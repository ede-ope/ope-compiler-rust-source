use std::collections::HashMap;

use crate::{
    analyser::blocklevel_analyser::{self, Expect},
    compiler::{ErrorHint, SemanticError},
    parser::{
        BlockLevel, DataState, Expr, ExprKind, Generic, InterfaceParam, MutStatus, StmtInfo,
        TopLevel, TypeHint, TypeSpec,
    },
    scanner::{Pos, Span, TruthVal},
    analyser::{Analyser, Element, Variable},
};

pub fn adjust_func_return(ret: &TypeSpec) -> TypeSpec {
    match ret.clone() {
        TypeSpec { hint, span } => {
            let empty_str = String::from("");
            // if it's not a reference then a raw value should be parsed to the type compatible check method
            // so it does not allow parsing of owned value to a ref
            match hint.clone() {
                TypeHint::Str => TypeSpec {
                    hint: TypeHint::StrLiteral,
                    span,
                },
                TypeHint::I8(_)
                | TypeHint::I16(_)
                | TypeHint::I32(_)
                | TypeHint::I64(_)
                | TypeHint::I128(_)
                | TypeHint::ISize(_)
                | TypeHint::U8(_)
                | TypeHint::U16(_)
                | TypeHint::U32(_)
                | TypeHint::U64(_)
                | TypeHint::U128(_)
                | TypeHint::USize(_)
                | TypeHint::F32(_)
                | TypeHint::F64(_) => TypeSpec {
                    hint: TypeHint::Number(empty_str),
                    span,
                },
                TypeHint::Bool | TypeHint::Trinary | TypeHint::Quaternary => TypeSpec {
                    hint: TypeHint::ArityLiteral(TruthVal::True),
                    span,
                }, // True works fine for the three of them
                TypeHint::Char => TypeSpec {
                    hint: TypeHint::CharLiteral,
                    span,
                },
                TypeHint::Custom {
                    import: _,
                    ident,
                    generics,
                } => TypeSpec {
                    hint: TypeHint::FuncGenericRet { ident, generics },
                    span,
                }, // works for all custom types
                oth => TypeSpec { hint: oth, span },
            }
        }
    }
}

pub fn get_actual_type(
    ty: &TypeSpec,
    generic_defs: &Vec<Generic>,
    par_genrs: &Vec<TypeSpec>,
    analyser: &mut Analyser,
) -> TypeSpec {
    let mut actual_expected_type = ty.clone();

    match &ty {
        TypeSpec {
            hint:
                TypeHint::Custom {
                    ident,
                    import: _,
                    generics: _, // same as par_genrs in the parameter of this method
                },
            span: _,
        }
        | TypeSpec {
            hint:
                TypeHint::FuncGenericRet {
                    ident,
                    generics: _, // same as par_genrs in the parameter of this method
                },
            span: _,
        } => {
            let mut index_of_generic = None;
            let mut traits_found = vec![];

            let mut i = 0;
            for genr in generic_defs {
                match genr {
                    Generic {
                        name: genr_name,
                        traits,
                    } => {
                        if genr_name.ident == ident.ident {
                            index_of_generic = Some(i);
                            traits_found = traits.clone();
                            break;
                        }
                    }
                }
                i += 1;
            }

            if index_of_generic.is_some() {
                // The type is a generic type
                // Check for the actual type from the

                actual_expected_type = par_genrs[index_of_generic.unwrap()].clone();

                // now if the type is generic, we will like to confirm it satisfied the trait arguments
                blocklevel_analyser::validate_traits(
                    &actual_expected_type,
                    &traits_found,
                    analyser,
                );
            }
            // let mut
            // if name.unwrap() ==  {

            // }
        }
        // TypeSpec {
        //     hint: TypeHint::Agent { tysp, mutb },
        //     span,
        // } => {
        //     match &tysp.clone().hint {
        //         TypeHint::Custom { ident, generics: _ }
        //         | TypeHint::FuncGenericRet { ident, generics: _ } => {
        //             let mut i = 0;
        //             for genr in generic_defs {
        //                 match genr {
        //                     Generic {
        //                         name: genr_name,
        //                         traits,
        //                     } => {
        //                         if genr_name.ident == ident.ident {
        //                             // The type is a generic type
        //                             // Check for the actual type from the
        //                             actual_expected_type = TypeSpec {
        //                                 hint: TypeHint::Agent {
        //                                     tysp: Box::new(par_genrs[i].clone()),
        //                                     mutb: mutb.clone(),
        //                                 },
        //                                 span: span.clone(),
        //                             };

        //                             // now if the type is generic, we will like to confirm it satisfied the trait arguments
        //                             validate_traits(&actual_expected_type, traits, analyser);
        //                             break;
        //                         }
        //                     }
        //                 }
        //                 i += 1;
        //             }
        //             // let mut
        //             // if name.unwrap() ==  {

        //             // }
        //         }
        //         _ => (),
        //     }
        // }
        _ => (),
    }

    actual_expected_type
}

pub fn check_for_val_type(
    expr: Expr,
    expect: &Expect,
    genrs: &Vec<Generic>,
    pars: &Vec<InterfaceParam>,
    analyser: &mut Analyser,
) -> Option<TypeSpec> {
    match expr.clone() {
        Expr {
            kind: ExprKind::Number(num),
            span,
        } => Some(TypeSpec {
            hint: TypeHint::Number(num),
            span,
        }),
        Expr {
            kind: ExprKind::Str(_),
            span,
        } => Some(TypeSpec {
            hint: TypeHint::StrLiteral,
            span,
        }),
        Expr {
            kind: ExprKind::TruthVal(val),
            span,
        } => Some(TypeSpec {
            hint: TypeHint::ArityLiteral(val),
            span,
        }),
        Expr {
            kind: ExprKind::Char(_),
            span,
        } => Some(TypeSpec {
            hint: TypeHint::CharLiteral,
            span,
        }),
        Expr {
            kind: ExprKind::Ident { affix: _, ident },
            ..
        } => {
            // look for it in scope
            let var = analyser.get_var(&ident);

            if var.is_some() {
                let ident_type = get_var_type(var.unwrap().0);

                if ident_type.is_some() {
                    Some(ident_type.unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr {
            kind:
                ExprKind::Binary {
                    lhs: _,
                    op: _,
                    rhs: _,
                },
            ..
        } => {
            // self.check_for_val_compatibility(
            //     *lhs,
            //     Expect::Void,
            //     &vec![],
            //     &vec![],
            //     symbol_table,
            // )?;
            // self.check_for_val_compatibility(
            //     *rhs,
            //     Expect::Void,
            //     &vec![],
            //     &vec![],
            //     symbol_table,
            // )?;

            // match op {
            //     Token::Plus => None,
            //     _ => None,
            // }
            None
        }
        Expr {
            kind: ExprKind::StructVal { ident, vals },
            span,
        } => Some(TypeSpec {
            hint: TypeHint::StructLiteral { ident, args: vals },
            span,
        }),
        Expr {
            kind: ExprKind::ElemNamespaceAccess { ident, access },
            span,
        } => Some(TypeSpec {
            hint: TypeHint::EnumLiteral {
                ident,
                kind: *access,
            },
            span,
        }),
        Expr {
            kind: ExprKind::ObjAssess { obj, fields },
            span: _,
        } => {
            // let name;

            let obj_type = check_for_val_type(*obj, expect, genrs, pars, analyser);

            if obj_type.is_some() {
                let mut prev_type = obj_type.unwrap();

                let mut success = None;

                let mut i = 0;
                for field in &fields {
                    let has_method =
                        blocklevel_analyser::get_field_type(&prev_type, &field, genrs, analyser);

                    if has_method.0 {
                        // confirm the parameter
                        prev_type = has_method.1.clone();
                        if i == fields.len() - 1 {
                            // The last field
                            success = Some(has_method.1);
                        }
                    } else {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Method {:?} not found", field)],
                            hints: vec![],
                            span: Span {
                                st: Pos { column: 0, line: 0 },
                                en: Pos { column: 0, line: 0 },
                                file: analyser.modl.clone(),
                            }, //self.get_span_of_expr(ty),
                        });
                    }

                    i += 1;
                }

                success
            } else {
                None
            }
        }
        Expr {
            kind: ExprKind::Block(block),
            span: _,
        } => {
            // create a scope for the parameters
            analyser.symbol_table.push(HashMap::new());
            let index = analyser.symbol_table.len() - 1;

            for par in pars {
                // populating the parameter scope
                match par.clone() {
                    InterfaceParam {
                        ident,
                        state: _,
                        ty,
                        is_mut,
                        span,
                    } => {
                        analyser.symbol_table[index].insert(
                            ident.ident.clone(),
                            Element::Local(BlockLevel::Param {
                                state: DataState::Owner,
                                ty,
                                name: ident,
                                mutable: is_mut,
                                info: StmtInfo { span, docs: vec![] },
                            }),
                        );
                    }
                }
            }

            let ret = blocklevel_analyser::analyse_block(&expect, block, genrs, pars, analyser); // always some for fun

            analyser.symbol_table.pop(); // the parameter scope
            ret
        }
        Expr {
            kind:
                ExprKind::ArgBuffer {
                    asy: _,
                    generics: buffer_genrs,
                    args,
                    name,
                },
            span: _,
        } => {
            let func = blocklevel_analyser::get_func_def(&name, analyser);

            if func.is_some() {
                match func.unwrap() {
                    TopLevel::FuncDecl {
                        is_async: _,
                        expect: func_exp,
                        ret_state: _,
                        ident,
                        generics,
                        is_visible: _,
                        is_public: _,
                        is_extern: _,
                        par,
                        worker_usage: _,
                        block: _,
                        info: _,
                    } => {
                        let genr_def = if generics.is_none() {
                            vec![]
                        } else {
                            generics.unwrap()
                        };
                        if genr_def.len() != buffer_genrs.len() {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Expected {} generic(s) but found {}",
                                    genr_def.len(),
                                    buffer_genrs.len()
                                )],
                                hints: vec![ErrorHint {
                                    hints: vec![format!("The function was defined here")],
                                    span: ident.span.clone(),
                                }],
                                span: name.span.clone(),
                            });
                            None
                        } else {
                            let mut i = 0;
                            for param in &par {
                                match param {
                                    InterfaceParam {
                                        ident: _,
                                        state,
                                        ty,
                                        is_mut: _,
                                        span: _,
                                    } => {
                                        blocklevel_analyser::validate_assignment(
                                            ty,
                                            state.clone(),
                                            &args[i].clone(),
                                            &genr_def,
                                            &buffer_genrs,
                                            analyser,
                                        );
                                    }
                                }
                                i += 1;
                            }

                            // Check for the actual type of the function return type

                            let ret =
                                get_actual_type(&func_exp, &genr_def, &buffer_genrs, analyser);

                            Some(ret)
                        }
                    }
                    _ => None, // default anyways
                }
            } else {
                None
            }
        }
        Expr {
            kind: ExprKind::Address(status, expr),
            span,
        } => {
            let ty = check_for_val_type(*expr, expect, genrs, pars, analyser);
            if ty.is_some() {
                Some(TypeSpec {
                    hint: TypeHint::RawPtr {
                        tysp: Box::new(ty.unwrap()),
                        mutb: match status {
                            MutStatus::Const => false,
                            _ => true,
                        },
                    },
                    span,
                })
            } else {
                None
            }
        }
        Expr {
            kind: ExprKind::AsType { expr, ty },
            span: _,
        } => {
            validate_ty_conversion(expect, *expr, *ty.clone(), analyser);
            Some(*ty)
        }
        // Expr::FromStatic { static_ident, call } => {
        //     // We need to get the Static defination first
        //     let def = self.get_static_def(&static_ident, symbol_table);

        //     if def.is_some() {
        //         match def.unwrap() {
        //             TopLevel::StaticDef {
        //                 ident: _,
        //                 public: _,
        //                 block,
        //                 span: _,
        //             } => {
        //                 match block {
        //                     StaticBlock {
        //                         takes: _,
        //                         globals: _,
        //                         interprs: _,
        //                         macro_def: _,
        //                         custom_tys: _,
        //                         type_converters: _,
        //                         traits: _,
        //                         workers: _,
        //                         funcs,
        //                         platform_n_s: _,
        //                     } => {
        //                         let mut static_symbol_table: Vec<HashMap<String, Element>> =
        //                             Vec::new();

        //                         static_symbol_table.push(HashMap::new()); // init global scope

        //                         match &*call {
        //                             Expr::ArgBuffer {
        //                                 asy: _,
        //                                 generics: buffer_genrs,
        //                                 args,
        //                                 name,
        //                             } => {
        //                                 // let mut genrs = vec![];
        //                                 // let mut pars = vec![];
        //                                 // we only need to populate the symbol table with functions here
        //                                 for func in funcs {
        //                                     match func {
        //                                         StaticLevel::FuncDecl {
        //                                             is_async,
        //                                             expect,
        //                                             ident,
        //                                             generics,
        //                                             is_public,
        //                                             is_extern,
        //                                             par,
        //                                             worker_usage,
        //                                             block,
        //                                             span,
        //                                         } => {
        //                                             let ident_extended = ident.clone();
        //                                             let span_extended = span.clone();
        //                                             // if generics.is_some() {
        //                                             //     genrs = generics.clone().unwrap();
        //                                             // };
        //                                             // pars = par.clone();

        //                                             let func_def = TopLevel::FuncDecl {
        //                                                 is_async,
        //                                                 expect,
        //                                                 ident,
        //                                                 generics,
        //                                                 is_public,
        //                                                 is_extern,
        //                                                 par,
        //                                                 worker_usage,
        //                                                 block,
        //                                                 span,
        //                                             };

        //                                             self.add_to_global_scope(
        //                                                 ident_extended,
        //                                                 func_def,
        //                                                 &span_extended,
        //                                                 &mut static_symbol_table,
        //                                             );
        //                                             // static_symbol_table[0].insert(k, v)
        //                                         }
        //                                         _ => (),
        //                                     }
        //                                 }

        //                                 let func =
        //                                     self.get_func_def(&name, &mut static_symbol_table);

        //                                 if func.is_some() {
        //                                     match func.unwrap() {
        //                                         TopLevel::FuncDecl {
        //                                             is_async: _,
        //                                             expect: func_exp,
        //                                             ident: _,
        //                                             generics,
        //                                             is_public: _,
        //                                             is_extern: _,
        //                                             par,
        //                                             worker_usage: _,
        //                                             block: _,
        //                                             span: _,
        //                                         } => {
        //                                             let genr_def = if generics.is_none() {
        //                                                 vec![]
        //                                             } else {
        //                                                 generics.unwrap()
        //                                             };
        //                                             if genr_def.len() != buffer_genrs.len() {
        //                                                 self.errs.push(SemanticError {
        //                                                    file: self.hirs[0].name.clone(),
        //                                                    message: format!(
        //                                                        "Expected {} generic(s) but found {}",
        //                                                        genr_def.len(),
        //                                                        buffer_genrs.len()
        //                                                    ),
        //                                                    span: Span {
        //                                                        st: 0,
        //                                                        en: 0,
        //                                                        line: 0,
        //                                                    },
        //                                                });
        //                                             } else {
        //                                                 let mut i = 0;
        //                                                 for param in &par {
        //                                                     match param {
        //                                                         InterfaceParam {
        //                                                             ident: _,
        //                                                             ty,
        //                                                             is_mut: _,
        //                                                             default_val: _,
        //                                                         } => {
        //                                                             self.validate_assignment(
        //                                                                 ty,
        //                                                                 &args[i].clone(),
        //                                                                 &genr_def,
        //                                                                 &buffer_genrs,
        //                                                                 symbol_table,
        //                                                                 // we need to use the module table because because this is were argument value came from including identifiers
        //                                                             );
        //                                                         }
        //                                                     }
        //                                                     i += 1;
        //                                                 }

        //                                                 // Check for the actual type of the function return type

        //                                                 let actual_type = self.get_actual_type(
        //                                                     &func_exp,
        //                                                     &genr_def,
        //                                                     &buffer_genrs,
        //                                                 );
        //                                                 match expect {
        //                                                     Expect::Option(ty)
        //                                                     | Expect::Required(ty) => {
        //                                                         self.is_type_compatible(
        //                                                             &ty,
        //                                                             &actual_type,
        //                                                             &mut static_symbol_table,
        //                                                         );
        //                                                     }
        //                                                     Expect::Void => {
        //                                                         self.errs.push(SemanticError {
        //                                                             file: self.modl.clone(),
        //                                                             message: format!("Expected Void but found '{:?}'", expr),
        //                                                             span: Span {
        //                                                                 st: 0,
        //                                                                 en: 0,
        //                                                                 line: 0,
        //                                                             },
        //                                                         });
        //                                                     }
        //                                                 }
        //                                             }
        //                                         }
        //                                         _ => (), // default anyways
        //                                     }
        //                                 } else {
        //                                     self.errs.push(SemanticError {
        //                                         file: self.hirs[0].name.clone(),
        //                                         message: format!(
        //                                             "Function '{:?}' cannot be found in static {:#?}",
        //                                             name,
        //                                             static_ident
        //                                         ),
        //                                         span: Span {
        //                                             st: 0,
        //                                             en: 0,
        //                                             line: 0,
        //                                         },
        //                                     });
        //                                 }

        //                                 // self.check_for_val_compatibility(
        //                                 //     *call,
        //                                 //     expect,
        //                                 //     &genrs,
        //                                 //     &pars,
        //                                 //     &mut static_symbol_table,
        //                                 // );
        //                             }
        //                             Expr::Block(block) => {
        //                                 // populate the static_symbol_table
        //                                 let static_block = self.static_hirs.get(&static_ident);

        //                                 if static_block.is_some() {
        //                                     {
        //                                         let cloned_static_block =
        //                                             static_block.unwrap().clone();

        //                                         self.init_global_scope(
        //                                             None,
        //                                             Some(&cloned_static_block),
        //                                             // using cloned_static_block because i can't both mutable and immutable *self at the same time
        //                                             &mut static_symbol_table,
        //                                         );
        //                                     }

        //                                     // self.check_for_val_compatibility(
        //                                     //     *call.clone(),
        //                                     //     expect,
        //                                     //     genrs,
        //                                     //     pars,
        //                                     //     static_symbol_table,
        //                                     // );
        //                                     // The block unlike function call needs analyses here since were could't analyse macro defination
        //                                     let block_type = self.analyse_block(
        //                                         expect.clone(),
        //                                         block.clone(),
        //                                         genrs,
        //                                         pars,
        //                                         &mut static_symbol_table,
        //                                     );

        //                                     if block_type.is_some() {
        //                                         match expect {
        //                                             Expect::Option(ty)
        //                                             | Expect::Required(ty) => {
        //                                                 self.is_type_compatible(
        //                                                     &ty,
        //                                                     &block_type.clone().unwrap(),
        //                                                     &mut static_symbol_table,
        //                                                 );
        //                                             }
        //                                             Expect::Void => {
        //                                                 self.errs.push(SemanticError {
        //                                                     file: self.modl.clone(),
        //                                                     message: format!("Expected Void but found '{:?}'", expr),
        //                                                     span: Span {
        //                                                         st: 0,
        //                                                         en: 0,
        //                                                         line: 0,
        //                                                     },
        //                                                 });
        //                                             }
        //                                         }
        //                                     }
        //                                 } else {
        //                                     self.errs.push(SemanticError {
        //                                         file: self.hirs[0].name.clone(),
        //                                         message: format!(
        //                                             "Static {:?} not found in this scope",
        //                                             static_ident
        //                                         ),
        //                                         span: Span {
        //                                             st: 0,
        //                                             en: 0,
        //                                             line: 0,
        //                                         },
        //                                     });
        //                                 }
        //                                 // else {
        //                                 //     self.errs.push(SemanticError {
        //                                 //         file: self.hirs[0].name.clone(),
        //                                 //         message: format!(
        //                                 //             "Static {:?} not found in this scope",
        //                                 //         ),
        //                                 //         span: Span {
        //                                 //             st: 0,
        //                                 //             en: 0,
        //                                 //             line: 0,
        //                                 //         },
        //                                 //     });
        //                                 // }
        //                             }
        //                             _ => (),
        //                         }
        //                     }
        //                 }
        //             }
        //             _ => (),
        //         }
        //     } else {
        //         self.errs.push(SemanticError {
        //             file: self.hirs[0].name.clone(),
        //             message: format!("Static {:?} not found in this scope", static_ident),
        //             span: Span {
        //                 st: 0,
        //                 en: 0,
        //                 line: 0,
        //             },
        //         });
        //     }
        //     // self.check_for_val_compatibility(*call, expect, genrs, pars)
        // }
        _ => None,
    }
}

pub fn get_var_type(var: Variable) -> Option<TypeSpec> {
    match var {
        Variable {
            state: _,
            ty,
            value: _,
            is_mutable: _,
            ident_span: _,
        } => Some(ty),
    }
}

pub fn is_num_type(type_spec: &TypeSpec) -> bool {
    match type_spec {
        TypeSpec {
            hint: TypeHint::I8(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::I16(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::I32(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::I64(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::I128(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::ISize(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::U8(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::U16(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::U32(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::U64(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::U128(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::USize(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::F32(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::F64(_),
            span: _,
        }
        | TypeSpec {
            hint: TypeHint::Number(_),
            span: _,
        } => true,
        _ => false,
    }
}

pub fn is_type_compatible(
    par_ty: &TypeSpec,
    val_ty: &TypeSpec,
    _genrs: &Vec<Generic>,
    analyser: &mut Analyser,
) {
    // This compare only scaler types
    // ...

    let mut show_default_error = true;
    // par_ty = Parameter type / Expected type
    // val_ty = The Value type
    match &par_ty.hint {
        TypeHint::I8(_) => match &val_ty.hint {
            TypeHint::I8(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::I16(_) => match &val_ty.hint {
            TypeHint::I16(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::I32(_) => match &val_ty.hint {
            TypeHint::I32(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::I64(_) => match &val_ty.hint {
            TypeHint::I64(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::I128(_) => match &val_ty.hint {
            TypeHint::I128(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::ISize(_) => match &val_ty.hint {
            TypeHint::ISize(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::U8(_) => match &val_ty.hint {
            TypeHint::U8(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::U16(_) => match &val_ty.hint {
            TypeHint::U16(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::U32(_) => match &val_ty.hint {
            TypeHint::U32(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::U64(_) => match &val_ty.hint {
            TypeHint::U64(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::U128(_) => match &val_ty.hint {
            TypeHint::U128(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::USize(_) => match &val_ty.hint {
            TypeHint::USize(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::F32(_) => match &val_ty.hint {
            TypeHint::F32(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::F64(_) => match &val_ty.hint {
            TypeHint::F64(_) | TypeHint::Number(_) => show_default_error = false,
            _ => (),
        },
        TypeHint::Bool => match &val_ty.hint {
            TypeHint::Bool => {
                show_default_error = false;
            }
            TypeHint::ArityLiteral(val) => match val {
                TruthVal::True | TruthVal::False => {
                    show_default_error = false;
                }
                _ => (),
            },
            _ => (),
        },
        TypeHint::Str => match &val_ty.hint {
            TypeHint::Str | TypeHint::StrLiteral => show_default_error = false,
            _ => (),
        },
        TypeHint::Custom {
            import,
            ident,
            generics,
        } => {
            // we have to check if the custom can be found in scope

            match &val_ty.hint {
                TypeHint::Custom {
                    import: val_ty_imp,
                    ident: val_ty_ident,
                    generics: val_ty_genetics,
                } => {
                    if import.is_none() && val_ty_imp.is_none() {
                        if ident.ident == val_ty_ident.ident {
                            if *generics == *val_ty_genetics {
                                show_default_error = false
                            }
                        }
                    } else {
                        // in this case they might actually be referering to different Type,
                        // when they have the same name, but different path
                        panic!("Not yet");
                    }
                }
                _ => (),
            }
        }
        TypeHint::RawPtr {
            tysp: par_raw,
            mutb: _,
        } => {
            // Here we can actually have a pointer to a pointer and so on (recuressive pointers e.g ***ọsán)
            // if is_num_type(val_ty) {
            //     show_default_error = false;
            // }
            match &val_ty.hint {
                TypeHint::RawPtr {
                    tysp: val_raw,
                    mutb: _,
                } => {
                    is_type_compatible(par_raw, val_raw, &vec![], analyser);
                    show_default_error = false;
                }
                _ => (),
            }
        }
        TypeHint::MVMTy(_hints) => (),
        TypeHint::MVSTy(_hint, _volumn) => (),
        _ => (), //for now
    }

    if show_default_error {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "Expected {} but found {}",
                par_ty.hint, val_ty.hint
            )],
            hints: vec![ErrorHint {
                hints: vec![format!("Here is the expected type")],
                span: par_ty.span.clone(),
            }],
            span: val_ty.span.clone(),
        });
    }
}

pub fn is_type_convertable(from: &TypeSpec, to: &TypeSpec, analyser: &mut Analyser) {
    // This compare only scaler types
    // ...

    let mut show_default_error = true;
    if from.hint != to.hint {
        match &from.hint {
            TypeHint::Number(_) => {
                let is_num = is_num_type(&to);

                if is_num {
                    show_default_error = false;
                }
            }
            TypeHint::I8(_) => match &to.hint {
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::I16(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::I32(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::I64(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::I128(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::ISize(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::U8(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::U16(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::U32(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::U64(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::U128(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::USize(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::USize(_) => match &to.hint {
                TypeHint::I8(_) => show_default_error = false,
                TypeHint::I16(_) => show_default_error = false,
                TypeHint::I32(_) => show_default_error = false,
                TypeHint::I64(_) => show_default_error = false,
                TypeHint::I128(_) => show_default_error = false,
                TypeHint::ISize(_) => show_default_error = false,
                // Need checks
                TypeHint::U8(_) => show_default_error = false,
                TypeHint::U16(_) => show_default_error = false,
                TypeHint::U32(_) => show_default_error = false,
                TypeHint::U64(_) => show_default_error = false,
                TypeHint::U128(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::F32(_) => match &to.hint {
                TypeHint::F64(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::F64(_) => match &to.hint {
                TypeHint::F32(_) => show_default_error = false,
                _ => (),
            },
            TypeHint::RawPtr { tysp, mutb: _ } => {
                let ty = *tysp.clone();

                match &ty.hint {
                    TypeHint::Str => match &to.hint {
                        TypeHint::RawPtr { tysp, mutb: _ } => {
                            let to_ty = *tysp.clone();

                            if to_ty.hint == TypeHint::I8(None) || to_ty.hint == TypeHint::Char {
                                show_default_error = false;
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
            _ => todo!("convertion not yet implemented!"),
        }
    } else {
        // no need to check if the are equal
        // can return a warning
        show_default_error = false;
    }

    if show_default_error {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "Expression of type {} can not be converted to {}",
                from.hint, to.hint
            )],
            hints: vec![],
            span: to.span.clone(),
        });
    }
}

pub fn validate_ty_conversion(expect: &Expect, expr: Expr, to: TypeSpec, analyser: &mut Analyser) {
    let expr_ty = check_for_val_type(expr.clone(), expect, &vec![], &vec![], analyser);

    if expr_ty.is_some() {
        let expr_ty = expr_ty.unwrap();

        is_type_convertable(&expr_ty, &to, analyser);
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "Could not resolve the expression '{}'",
                expr.kind
            )],
            hints: vec![ErrorHint {
                hints: vec![format!("The expression was defined here")],
                span: expr.span.clone(),
            }],
            span: expr.span.clone(),
        });
    }
}
