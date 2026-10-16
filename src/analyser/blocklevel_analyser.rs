use crate::borrow_checker::FuncDeclInfo;
use crate::compiler::{ErrorHint, Mod, NameSpace, SemanticError};
use crate::move_checker;
use crate::name_resolver;
use crate::parser::{
    BlockLevel, Case, CustTyAss, CustomBlock, CustomLevel, DataState, ElseIf, EnumVariants, Expr,
    ExprKind, FieldAssignm, ForAssign, Generic, Ident, IdentAffix, InterfaceParam, Method, ModPath,
    MutStatus, Parameter, StmtInfo, StructParam, TopLevel, TypeHint, TypeSpec,
};
use crate::scanner::{FullTok, Pos, Span, Token};
use crate::analyser::{Analyser, Element, Variable};
use crate::type_checker;
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IdentRep {
    Variable,
    CustomType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expect {
    Required(TypeSpec, DataState),
    Option(TypeSpec, DataState),
    Void,
}

pub fn validate_num_of_par_generics(
    defs: &Vec<Generic>,
    par: &Vec<TypeSpec>,
    name: Ident,
    analyser: &mut Analyser,
) {
    if defs.len() != par.len() {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "Type {:?} as {} number of generics but found {}",
                name,
                defs.len(),
                par.len()
            )],
            hints: vec![],
            span: name.span.clone(),
        });
    }
}

pub fn validate_assignment(
    par_ty: &TypeSpec,
    par_state: DataState,
    expr: &Expr,
    generic_defs: &Vec<Generic>,
    par_genrs: &Vec<TypeSpec>,
    analyser: &mut Analyser,
) {
    match par_ty.hint {
        TypeHint::Void => {
            check_for_val_compatibility(expr.clone(), &Expect::Void, &vec![], &vec![], analyser);
            // self.analyse_block(Expect::Void, &block);
        }
        _ => {
            let actual_expected_type =
                type_checker::get_actual_type(par_ty, generic_defs, par_genrs, analyser);

            check_for_val_compatibility(
                expr.clone(),
                &Expect::Required(actual_expected_type, par_state),
                &vec![],
                &vec![],
                analyser,
            );
            // self.analyse_block(Expect::Required(expect.clone()), &block);
        }
    }
}

pub fn get_field_type(
    ty: &TypeSpec,
    expr: &Expr,
    genrs: &Vec<Generic>,
    analyser: &mut Analyser,
) -> (bool, TypeSpec, Vec<InterfaceParam>) {
    let empty_span = Span {
        st: Pos { column: 0, line: 0 },
        en: Pos { column: 0, line: 0 },
        file: analyser.modl.clone(),
    };

    match &ty.hint {
        TypeHint::Custom { import, ident, .. } => {
            match expr {
                Expr {
                    kind:
                        ExprKind::ArgBuffer {
                            asy: _,
                            generics: buffer_generics,
                            args: buffer_args,
                            name: buffer_name,
                        },
                    span: _,
                } => {
                    let cus_ty = analyser.get_custom_type(import, &ident, genrs);
                    let mut meth_type = None;
                    let mut parameters = None;

                    if cus_ty.is_some() {
                        match cus_ty.unwrap() {
                            TopLevel::EnumDecl {
                                ident: _,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public: _,
                                associateds,
                                bridges: _,
                                info: _,
                            }
                            | TopLevel::StructDecl {
                                ident: _,
                                generic_types: _,
                                vals: _,
                                traits: _,
                                public: _,
                                associateds,
                                bridges: _,
                                info: _,
                            } => {
                                for meth in associateds {
                                    match meth {
                                        CustTyAss::Method(method) => {
                                            match method {
                                                Method {
                                                    worker_usage: _,
                                                    meth_self: _,
                                                    // inher_val: _,
                                                    // is_mut: _,
                                                    expect,
                                                    ret_state: _,
                                                    ident,
                                                    generics,
                                                    is_public: _,
                                                    is_extern: _,
                                                    par,
                                                    block: _,
                                                    info: _,
                                                } => {
                                                    let expect = expect.clone();
                                                    if ident.ident == buffer_name.ident {
                                                        let genr_vec = if generics.is_none() {
                                                            Vec::new()
                                                        } else {
                                                            generics.clone().unwrap()
                                                        };
                                                        if buffer_generics.len() != genr_vec.len() {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "Expected {} generics, but found {}",
                                                                    genr_vec.len(), buffer_generics.len()
                                                                )],
                                                                hints: vec![
                                                                    ErrorHint {
                                                                        hints: vec![format!("The generic types are declared here for this type")],
                                                                        span: ident.span.clone()
                                                                    }
                                                                ],
                                                                span: buffer_name.span.clone(),
                                                            });
                                                        }

                                                        if buffer_args.len() != par.len() {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "Expected {} argument(s), but found {}",
                                                                    par.len(),
                                                                    buffer_args.len()
                                                                )],
                                                                hints: vec![ErrorHint {
                                                                    hints: vec![format!(
                                                                        "Method is defined here"
                                                                    )],
                                                                    span: ident.span.clone(),
                                                                }],
                                                                span: buffer_name.span.clone(),
                                                            });
                                                        }
                                                        // argument types need to be varified
                                                        let mut j = 0;
                                                        for param in &par {
                                                            match param {
                                                                InterfaceParam {
                                                                    ident: _,
                                                                    state,
                                                                    ty,
                                                                    is_mut: _,
                                                                    span: _,
                                                                } => {
                                                                    validate_assignment(
                                                                        ty,
                                                                        state.clone(),
                                                                        &buffer_args[j],
                                                                        &vec![],
                                                                        &vec![],
                                                                        analyser,
                                                                    );
                                                                }
                                                            }
                                                            j += 1;
                                                        }

                                                        meth_type = Some(expect);
                                                        parameters = Some(par);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        CustTyAss::AssFunction(_) => {
                                            todo!("tode");
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }

                    if meth_type.is_some() {
                        (true, meth_type.unwrap(), parameters.unwrap())
                    } else {
                        (
                            false,
                            TypeSpec {
                                hint: TypeHint::Void,
                                span: empty_span,
                            },
                            vec![],
                        )
                    }
                }
                _ => {
                    (
                        false,
                        TypeSpec {
                            hint: TypeHint::Void,
                            span: empty_span,
                        },
                        vec![],
                    ) // TypeSpec is only used if bool is true
                      // for now to_do()
                }
            }
        }
        // TypeSpec::Scaler(_) => {
        //     (true, ty.clone()) // for now to_do()
        // }
        _ => {
            (
                false,
                TypeSpec {
                    hint: TypeHint::Void,
                    span: empty_span,
                },
                vec![],
            ) // TypeSpec is only used if bool is true
              // for now to_do()
        }
    }
}

fn does_type_has_trait(_ty: &TypeSpec, _d_trait: &String) -> bool {
    false // for now to_do()
}

pub fn validate_traits(expected_type: &TypeSpec, traits: &Vec<Ident>, analyser: &mut Analyser) {
    for each in traits {
        if !does_type_has_trait(expected_type, &each.ident) {
            analyser.errs.push(SemanticError {
                messages: vec![format!(
                    "Type {:?} does not impliment trait {:?}",
                    expected_type, each.ident
                )],
                hints: vec![],
                span: each.span.clone(),
            });
        }
    }
}

fn validate_fields(
    ident: &Ident,
    vals: &Vec<StructParam>,
    assigns: &Vec<FieldAssignm>,
    generic_defs: &Vec<Generic>,
    _par_genrs: &Vec<TypeSpec>,
    analyser: &mut Analyser,
) {
    if vals.len() == assigns.len() {
        let mut assigned_fields = Vec::new();
        for ass in assigns {
            match ass {
                FieldAssignm {
                    ident: val_ident,
                    expr,
                } => {
                    if assigned_fields.contains(val_ident) {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "An attempt had already been made to assign value to field {:?}",
                                val_ident
                            )],
                            hints: vec![],
                            span: val_ident.span.clone(),
                        });
                    } else {
                        assigned_fields.push(val_ident.clone());
                        let mut matched_param = None;
                        for struct_par in vals {
                            match struct_par {
                                StructParam { par, is_public: _ } => match par {
                                    Parameter {
                                        ident,
                                        state: _,
                                        ty: _,
                                    } => {
                                        if ident.ident == val_ident.ident {
                                            matched_param = Some(struct_par.clone());
                                        }
                                    }
                                },
                            }
                        }

                        if matched_param.is_some() {
                            let ty;
                            let param_state;

                            match matched_param.unwrap() {
                                StructParam { par, is_public: _ } => match par {
                                    Parameter {
                                        ident: _,
                                        state,
                                        ty: par_ty,
                                    } => {
                                        ty = par_ty;
                                        param_state = state;
                                    }
                                },
                            }

                            check_for_val_compatibility(
                                expr.clone(),
                                &Expect::Required(ty, param_state),
                                generic_defs,
                                &vec![],
                                analyser,
                            );
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!("Type not found")],
                                hints: vec![],
                                span: val_ident.span.clone(),
                            });
                        }
                    }
                }
            }
        }
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "Struct {:?} has {} fields, but found {}",
                ident.ident,
                vals.len(),
                assigns.len()
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    }
}

fn validate_enum_variant(
    ident: &Ident,
    vals: &Vec<EnumVariants>,
    variant: &Expr,
    generic_defs: &Vec<Generic>,
    par_genrs: &Vec<TypeSpec>,
    analyser: &mut Analyser,
) {
    match variant {
        Expr {
            kind:
                ExprKind::ArgBuffer {
                    asy: _,
                    generics: _,
                    args,
                    name,
                },
            span: _,
        } => {
            // Example: AnEnum::Variant()
            //                  _________ -> here is the ArgBuffer
            let mut variant = false;
            for val in vals {
                match val {
                    EnumVariants::Tuple {
                        name: tuple_def_name,
                        fields,
                    } => {
                        if tuple_def_name.ident == name.ident {
                            if fields.len() != args.len() {
                                analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Expected {} argument(s) but found {}",
                                        fields.len(),
                                        args.len()
                                    )],
                                    hints: vec![ErrorHint {
                                        hints: vec![format!("The variant is declared here")],
                                        span: tuple_def_name.span.clone(),
                                    }],
                                    span: name.span.clone(),
                                });
                                variant = true; // to give on the error message above
                                                // because it's the variant but with incorrect num of args
                            } else {
                                let mut i = 0;
                                for arg in args {
                                    validate_assignment(
                                        &fields[i].1,
                                        fields[i].0.clone(),
                                        arg,
                                        generic_defs,
                                        par_genrs,
                                        analyser,
                                    );

                                    variant = true;
                                    i += 1;
                                    let _ = i;
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }

            if !variant {
                analyser.errs.push(SemanticError {
                    messages: vec![format!(
                        "Couldn't find variant {} in Enum {}",
                        name.ident.clone(),
                        ident.ident.clone()
                    )],
                    hints: vec![ErrorHint {
                        hints: vec![format!("The Enum is defined here")],
                        span: ident.span.clone(),
                    }],
                    span: name.span.clone(),
                });
            }
        }
        Expr {
            kind:
                ExprKind::Ident {
                    affix: _,
                    ident: value_variant_name,
                },
            span,
        } => {
            // A simple variant
            // Example: AnEnum::Variant
            //                  _______-> here is the Ident
            let mut variant = false;
            for val in vals {
                match val {
                    EnumVariants::Simple(variant_def_name) => {
                        if variant_def_name.ident == *value_variant_name.ident {
                            variant = true;
                        }
                    }
                    _ => (),
                }
            }

            if !variant {
                analyser.errs.push(SemanticError {
                    messages: vec![format!(
                        "Couldn't find variant {} in Enum '{}'",
                        value_variant_name.ident, ident.ident
                    )],
                    hints: vec![ErrorHint {
                        hints: vec![format!("The Enum is defined here")],
                        span: ident.span.clone(),
                    }],
                    span: span.clone(),
                });
            }
        }
        Expr {
            kind: ExprKind::StructVal { ident: _, vals: _ },
            span: _,
        } => {
            // A struct value
            // Example: AnEnum::Variant{ a: "here" }
            //                  _________ -> here is the StructVal
        }
        _ => (),
    }
}

fn _get_type_origin(ident: &Ident, analyser: &mut Analyser) -> String {
    let element = analyser.get_element(ident);

    if element.is_some() {
        let ty = element.unwrap();

        match ty {
            Element::Global(toplevel) => match toplevel {
                TopLevel::TakeStmt {
                    take: _,
                    from,
                    lib,
                    info: _,
                } => {
                    let ret;
                    let mod_str = match &from {
                        ModPath::Ident(ident) => ident.clone().ident,
                        ModPath::Str(string) => string.clone().ident,
                        ModPath::FromLibFolder(f, m) => format!("{}/{}", f.ident, m.ident),
                        ModPath::UnExpected => format!("Unknown"),
                    };

                    if lib.is_none() {
                        ret = mod_str;
                    } else {
                        ret = format!("{}::{}", mod_str, lib.unwrap().ident);
                    }

                    ret
                }
                _ => analyser.modl.clone(),
            },
            Element::Local(block_level) => match block_level {
                BlockLevel::TakeStmt {
                    take: _,
                    from,
                    lib,
                    info: _,
                } => {
                    let ret;
                    let mod_str = match &from {
                        ModPath::Ident(ident) => ident.clone().ident,
                        ModPath::Str(string) => string.clone().ident,
                        ModPath::FromLibFolder(f, m) => format!("{}/{}", f.ident, m.ident),
                        ModPath::UnExpected => format!("Unknown"),
                    };

                    if lib.is_none() {
                        ret = mod_str;
                    } else {
                        ret = format!("{}::{}", mod_str, lib.unwrap().ident);
                    }

                    ret
                }
                _ => analyser.modl.clone(),
            },
        }
    } else {
        analyser.modl.clone()
    }
}

pub fn validate_custom_type_value(
    type_def: &TopLevel,
    par_type: &TypeSpec,
    expr: &Expr,
    analyser: &mut Analyser,
) {
    // expr: Expr is the value
    let genrs; // generics defined in the parameter e.g kí Èyí<ọsán>; genrs = vec[ọsán]
    match &par_type.hint {
        TypeHint::Custom { generics, .. } => {
            genrs = generics.clone();
        }
        _ => {
            genrs = vec![];
        }
    }

    match type_def {
        TopLevel::EnumDecl {
            ident,
            generic_types,
            vals,
            traits: _,
            public: _,
            associateds: _,
            bridges: _,
            info: _,
        } => {
            match expr {
                Expr {
                    kind:
                        ExprKind::ElemNamespaceAccess {
                            ident: lit_ident,
                            access,
                        },
                    span,
                } => {
                    if lit_ident.is_some() {
                        if lit_ident.clone().unwrap().ident != ident.ident {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Expected type {:?} but found {:?}",
                                    ident.ident,
                                    lit_ident.clone().unwrap().ident
                                )],
                                hints: vec![],
                                span: span.clone(),
                            });
                            return;
                        }
                    }

                    validate_enum_variant(ident, vals, *&access, generic_types, &genrs, analyser);
                }
                // Expr {
                //     kind:
                //         ExprKind::Custom {
                //             ident: var_ty,
                //             import: _,
                //             generics: _,
                //         },
                //     span,
                // } => {
                //     if var_ty.ident != ident.ident {
                //         analyser.errs.push(SemanticError {
                //             file: analyser.modl.clone(),
                //             messages: vec![format!(
                //                 "Expected type {:?} but found {:?}",
                //                 ident.ident, var_ty.ident
                //             )],
                //             hints: vec![],
                //             span: span.clone(),
                //         });
                //         return;
                //     }
                // }
                Expr {
                    kind:
                        ExprKind::FuncGenericRet {
                            ident: _,
                            generics: _,
                        },
                    span: _,
                } => (), // Can accept this
                oths => {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!("A variant of Enum '{}', is expected", ident.ident)],
                        hints: vec![],
                        span: oths.span.clone(),
                    });
                }
            }
        }
        TopLevel::StructDecl {
            ident,
            generic_types,
            vals,
            traits: _,
            public: _,
            associateds: _,
            bridges: _,
            info: _,
        } => {
            match expr {
                Expr {
                    kind:
                        ExprKind::StructVal {
                            ident: lit_ident,
                            vals: args,
                        },
                    span: _,
                } => {
                    if lit_ident.is_some() {
                        if lit_ident.clone().unwrap().ident == ident.ident {
                            // then it's most align
                            validate_fields(ident, vals, args, generic_types, &genrs, analyser);
                        }
                    } else {
                        //if infered
                        // then the structure value most align with the defination
                        validate_fields(ident, vals, args, generic_types, &genrs, analyser);
                    }
                }
                // TypeSpec {
                //     hint:
                //         TypeHint::Custom {
                //             import: None,
                //             ident: var_ty,
                //             generics: _,
                //         },
                //     span,
                // } => {
                //     if var_ty.ident != ident.ident {
                //         analyser.errs.push(SemanticError {
                //             file: analyser.modl.clone(),
                //             messages: vec![format!(
                //                 "Expected type {:?} but found {:?}",
                //                 ident.ident, var_ty.ident
                //             )],
                //             hints: vec![],
                //             span: span.clone(),
                //         });
                //         return;
                //     }
                // }
                Expr {
                    kind:
                        ExprKind::FuncGenericRet {
                            ident: _,
                            generics: _,
                        },
                    span: _,
                } => (), // Can accept this
                oth => {
                    let span = oth.span.clone();

                    analyser.errs.push(SemanticError {
                        messages: vec![format!("Expect struct field(s) for '{}'", ident.ident)],
                        hints: vec![ErrorHint {
                            hints: vec![format!("The Type is defined here")],
                            span: ident.span.clone(),
                        }],
                        span,
                    });
                }
            }
        }
        _ => (),
    }
}

fn validate_assess_expr(
    obj: &Expr,
    fields: Vec<Expr>,
    expect: &Expect,
    genrs: &Vec<Generic>,
    expr: Expr, // the whole assess e.g obj.field.meth()
    pars: &Vec<InterfaceParam>,
    analyser: &mut Analyser,
) {
    match expect {
        Expect::Option(ty, expect_state) | Expect::Required(ty, expect_state) => {
            let obj_type =
                type_checker::check_for_val_type(obj.clone(), expect, genrs, pars, analyser);

            if obj_type.is_some() {
                let mut prev_type = obj_type.unwrap();

                let mut success = None;

                let mut i = 0;
                for method in &fields {
                    let has_method = get_field_type(&prev_type, &method, genrs, analyser);

                    if has_method.0 {
                        // confirm the parameter
                        prev_type = has_method.1.clone();
                        if i == fields.len() - 1 {
                            // The last field
                            success = Some(has_method.1);
                        }
                    } else {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Method {} not found", method.kind)],
                            hints: vec![],
                            span: method.span.clone(), //self.get_span_of_expr(ty),
                        });
                    }

                    i += 1;
                }

                if success.is_some() {
                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Expr {} is not owned by any variable which is what a reference requires", expr.kind)],
                            hints: vec![],
                            span: expr.span
                        });
                    } else {
                        type_checker::is_type_compatible(
                            &ty,
                            &success.clone().unwrap(),
                            genrs,
                            analyser,
                        );
                    }
                } else {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!(
                            "The return type of this expression was unresolved"
                        )],
                        hints: vec![],
                        span: expr.span,
                    });
                }
            }
        }
        _ => (),
    }
}

fn validate_fn_call_expr(
    name: Ident,
    expect: &Expect,
    genrs: &Vec<Generic>,
    buffer_genrs: Vec<TypeSpec>,
    args: Vec<Expr>,
    is_stmt: bool, // wheither its from pitú statement or not
    analyser: &mut Analyser,
) {
    let func = get_func_def(&name, analyser);

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
                                validate_assignment(
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

                    let ret = type_checker::get_actual_type(
                        &func_exp,
                        &genr_def,
                        &buffer_genrs,
                        analyser,
                    );
                    let adjusted_func_ret = type_checker::adjust_func_return(&ret);

                    if !is_stmt {
                        // if its not from the call statement, the return value/type matters
                        match expect {
                            Expect::Option(ty, expect_state)
                            | Expect::Required(ty, expect_state) => {
                                if *expect_state == DataState::ImutRef
                                    || *expect_state == DataState::MutRef
                                {
                                    analyser.errs.push(SemanticError {
                                            messages: vec![format!("Function call {} is not owned by any variable which is what a reference requires", name.ident)],
                                            hints: vec![],
                                            span: Span {
                                                st: Pos { column: 0, line: 0 },
                                                en: Pos { column: 0, line: 0 },
                                                file: analyser.modl.clone()
                                            }, //self.get_span_of_expr(ty),
                                        });
                                } else {
                                    type_checker::is_type_compatible(
                                        &ty,
                                        &adjusted_func_ret,
                                        genrs,
                                        analyser,
                                    );
                                }
                            }
                            Expect::Void => {}
                        }
                    }
                }
            }
            _ => (), // default anyways
        }
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!("Function {} not found in this scope", name.ident)],
            hints: vec![],
            span: name.span.clone(), //self.get_span_of_expr(ty),
        });
    }
}

fn validate_static_access(
    expect: &Expect,
    pars: &Vec<InterfaceParam>,
    ident: &Ident,
    ty: &TypeSpec,
    static_block: CustomBlock,
    kind: &Expr,
    expect_state: &DataState,
    genrs: &Vec<Generic>,
    analyser: &mut Analyser,
) {
    // The block is like a seperate module with it's own namespace
    match &static_block {
        CustomBlock {
            takes: _,
            globals: _,
            macro_def: _,
            custom_tys: _,
            traits: _,
            workers: _,
            funcs,
            platform_n_s: _,
            static_defs: _,
        } => {
            // let mut static_symbol_table = Vec::new();

            match kind {
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
                    // let mut genrs = vec![];
                    // let mut pars = vec![];
                    // we only need to populate the symbol table with functions here
                    // If it's a static function call
                    let mut func_def = None;
                    for func in funcs {
                        match func.clone() {
                            CustomLevel::FuncDecl {
                                is_async,
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_visible: _,
                                is_extern,
                                par,
                                worker_usage,
                                block,
                                info,
                            } => {
                                if ident.ident == name.ident {
                                    func_def = Some(TopLevel::FuncDecl {
                                        is_async,
                                        expect,
                                        ret_state,
                                        ident,
                                        generics,
                                        is_visible: false,
                                        is_public,
                                        is_extern,
                                        par,
                                        worker_usage,
                                        block,
                                        info,
                                    });
                                }
                                break;
                                // static_symbol_table[0].insert(k, v)
                            }
                            _ => (),
                        }
                    }

                    if func_def.is_some() {
                        match func_def.unwrap() {
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
                                            hints: vec![format!(
                                                "The function was defined in here"
                                            )],
                                            span: ident.span.clone(),
                                        }],
                                        span: name.span.clone(),
                                    });
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
                                                validate_assignment(
                                                    ty,
                                                    state.clone(),
                                                    &args[i].clone(),
                                                    &genr_def,
                                                    &buffer_genrs,
                                                    analyser,
                                                    // we need to use the module table because because this is were argument value came from including identifiers
                                                );
                                            }
                                        }
                                        i += 1;
                                    }

                                    // Check for the actual type of the function return type

                                    let actual_type = type_checker::get_actual_type(
                                        &func_exp,
                                        &genr_def,
                                        &buffer_genrs,
                                        analyser,
                                    );
                                    if *expect_state == DataState::ImutRef
                                        || *expect_state == DataState::MutRef
                                    {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!("Function call {} is not owned by any variable which is what a reference requires", name.ident)],
                                            hints: vec![],
                                            span: name.span.clone(), //self.get_span_of_expr(ty),
                                        });
                                    } else {
                                        type_checker::is_type_compatible(
                                            &ty,
                                            &actual_type,
                                            genrs,
                                            analyser,
                                        );
                                    }
                                }
                            }
                            _ => (), // default anyways
                        }
                    } else {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "Function '{}' cannot be found in static {}",
                                name.ident, ident.ident
                            )],
                            hints: vec![],
                            span: name.span.clone(), //self.get_span_of_expr(ty),
                        });
                    }

                    // self.check_for_val_compatibility(
                    //     *call,
                    //     expect,
                    //     &genrs,
                    //     &pars,
                    //     &mut static_symbol_table,
                    // );
                }
                Expr {
                    kind: ExprKind::Block(block),
                    span,
                } => {
                    // For blocks, we need to analyse the block from here
                    // We still need the unit for imported elements/resources
                    let former_symbol_table = analyser.symbol_table.clone();
                    analyser.symbol_table = vec![];
                    let former_modl = analyser.modl.clone();
                    analyser.modl = ident.ident.clone();

                    // populate the static_symbol_table
                    init_global_scope(
                        None,
                        Some(&static_block),
                        // using cloned_static_block because i can't both mutable and immutable *self at the same time
                        analyser,
                    );

                    // self.check_for_val_compatibility(
                    //     *call.clone(),
                    //     expect,
                    //     genrs,
                    //     pars,
                    //     static_symbol_table,
                    // );
                    // The block unlike function call needs analyses here since were could't analyse macro defination
                    let block_type = analyse_block(expect, block.clone(), genrs, pars, analyser);

                    if block_type.is_some() {
                        if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef
                        {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Block Expr from line {} to {} is not owned by any variable which is what a reference requires",
                                    span.st.line, span.en.line
                                )],
                                hints: vec![],
                                span: span.clone()
                            });
                        } else {
                            type_checker::is_type_compatible(
                                &ty,
                                &block_type.clone().unwrap(),
                                genrs,
                                analyser,
                            );
                        }
                    }

                    analyser.modl = former_modl;
                    analyser.symbol_table = former_symbol_table;
                }
                _ => (),
            }
        }
    }
}

fn add_pars_scope(
    // a seperate scope for parameters in non blocks
    pars: &Vec<InterfaceParam>,
    analyser: &mut Analyser,
) {
    // create a scope for the parameters
    if pars.len() > 0 {
        analyser.symbol_table.push(HashMap::new());
        let index = analyser.symbol_table.len() - 1;

        for par in pars {
            // populating the parameter scope
            match par.clone() {
                InterfaceParam {
                    ident,
                    state,
                    ty,
                    is_mut,
                    span,
                } => {
                    analyser.symbol_table[index].insert(
                        ident.ident.clone(),
                        Element::Local(BlockLevel::Param {
                            state,
                            ty,
                            name: ident,
                            mutable: is_mut,
                            info: StmtInfo { span, docs: vec![] },
                        }),
                    );
                }
            }
        }
    }
}

fn validate_assoc_fn_call(_expr: Expr, _associations: Vec<CustTyAss>, _analyser: &mut Analyser) {}

pub fn check_for_val_compatibility(
    expr: Expr,
    expect: &Expect,
    genrs: &Vec<Generic>,
    pars: &Vec<InterfaceParam>,
    analyser: &mut Analyser,
) {
    match expect {
        Expect::Option(ty, expect_state) | Expect::Required(ty, expect_state) => {
            match expr.clone() {
                Expr {
                    kind: ExprKind::Number(num),
                    span,
                } => {
                    add_pars_scope(pars, analyser);

                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("The number expr {} is not owned by any variable which is what a reference requires", num)],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        type_checker::is_type_compatible(
                            &ty,
                            &TypeSpec {
                                hint: TypeHint::Number(num),
                                span,
                            },
                            genrs,
                            analyser,
                        );
                    }
                    // self.validate_assignment(&ty, &expr, &vec![], &vec![], symbol_table);

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::TruthVal(truth_val),
                    span,
                } => {
                    add_pars_scope(pars, analyser);
                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Expr {} is not owned by any variable which is what a reference requires", expr.kind)],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        type_checker::is_type_compatible(
                            &ty,
                            &TypeSpec {
                                hint: TypeHint::ArityLiteral(truth_val),
                                span,
                            },
                            genrs,
                            analyser,
                        );
                    }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::Str(_),
                    span,
                } => {
                    add_pars_scope(pars, analyser);

                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Expr {} is not owned by any variable which is what a reference requires", expr.kind)],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        type_checker::is_type_compatible(
                            &ty,
                            &TypeSpec {
                                hint: TypeHint::StrLiteral,
                                span,
                            },
                            genrs,
                            analyser,
                        );
                    }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::Char(_),
                    span,
                } => {
                    add_pars_scope(pars, analyser);

                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("This char Expr is not owned by any variable which is what a reference requires")],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        type_checker::is_type_compatible(
                            &ty,
                            &TypeSpec {
                                hint: TypeHint::CharLiteral,
                                span,
                            },
                            genrs,
                            analyser,
                        );
                    }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::Ident { affix, ident },
                    ..
                } => {
                    add_pars_scope(pars, analyser);
                    // look for it in scope
                    let var = get_var(&ident, analyser);

                    if var.is_some() {
                        // MOVE SEMANTICS
                        match expect.clone() {
                            Expect::Option(_, to_be_ref) | Expect::Required(_, to_be_ref) => {
                                if to_be_ref == DataState::ImutRef || to_be_ref == DataState::MutRef
                                {
                                    // its a reference
                                    match var.clone().unwrap() {
                                        Variable {
                                            state,
                                            ty: _,
                                            value: _,
                                            is_mutable: _,
                                            ident_span,
                                        } => match state {
                                            DataState::Owner => {
                                                // kí ọsán olówó: "Àkànbí" // an owner
                                                // kí &ọsán orúkọ: olówó

                                                if affix.is_some() {
                                                    // kí ọsán olówó: "Àkànbí" // an owner
                                                    // kí &ọsán orúkọ: kó olówó
                                                    let affix = affix.clone().unwrap();
                                                    match affix {
                                                        IdentAffix::Move(_) => {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![
                                                                    format!("{:#?} is assigned to a reference", ident.ident),
                                                                    format!("Remove the move(kó) keyword")
                                                                ],
                                                                hints: vec![],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                        _ => {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "Unexpected affix of {}",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                    }
                                                } // else: does matter weather the type implement Copy or not
                                            }
                                            DataState::Moved(span) => {
                                                // kí ọsán olówó: "Àkànbí" // also an owner but moved
                                                // ...
                                                // kí ọsán orúkọ: olówó
                                                analyser.errs.push(SemanticError {
                                                    messages: vec![format!(
                                                        "{:#?} moved already",
                                                        ident.ident
                                                    )],
                                                    hints: vec![ErrorHint {
                                                        hints: vec![format!(
                                                            "{:#?} moved here",
                                                            ident.ident
                                                        )],
                                                        span,
                                                    }],
                                                    span: ident.span.clone(),
                                                });
                                            }
                                            DataState::ImutRef
                                            | DataState::MutRef
                                            | DataState::GlobalMutRef
                                            | DataState::GlobalImutRef => {
                                                // kí &ọsán olówó: òmíràn // A ref
                                                // kí &ọsán orúkọ: olówó
                                                if affix.is_some() {
                                                    let affix = affix.clone().unwrap();
                                                    match affix {
                                                        IdentAffix::Move(_) => {
                                                            // kí &ọsán olówó: òmíràn // A ref
                                                            // kí ọsán orúkọ: kó olówó
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "A refrence like {:#?} cannot be moved",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![ErrorHint {
                                                                    hints: vec![format!(
                                                                        "{:#?} was defined here",
                                                                        ident.ident
                                                                    )],
                                                                    span: ident_span,
                                                                }],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                        _ => {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "{:#?} has an unexpected affix",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                    }
                                                } // else: Does not matter if Copy is implemented or not
                                            }
                                            DataState::Global => {
                                                if affix.is_some() {
                                                    let affix = affix.clone().unwrap();
                                                    match affix {
                                                        IdentAffix::Move(_) => {
                                                            // kárí ọsán olówó: "Àkànbí" // also an owner
                                                            // {
                                                            //      kí &ọsán orúkọ: kó olówó
                                                            // }
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "A global variable like {:#?} cannot be moved",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![ErrorHint {
                                                                    hints: vec![format!(
                                                                        "{:#?} was defined here",
                                                                        ident.ident
                                                                    )],
                                                                    span: ident_span,
                                                                }],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                        _ => (),
                                                    }
                                                }
                                            }
                                        },
                                    }
                                } else {
                                    // kí ọsán orúkọ: olówó
                                    // Not a ref
                                    // --CHECK MOVE--
                                    match var.clone().unwrap() {
                                        Variable {
                                            state,
                                            ty,
                                            value: _,
                                            is_mutable: _,
                                            ident_span,
                                        } => match state {
                                            DataState::Owner => {
                                                // kí ọsán olówó: "Àkànbí" // also an owner
                                                // kí ọsán orúkọ: olówó
                                                let impl_copy =
                                                    move_checker::check_ty_copy_impl(&ty, analyser);
                                                if !impl_copy {
                                                    if affix.is_some() {
                                                        let affix = affix.clone().unwrap();
                                                        match affix {
                                                            IdentAffix::Move(s) => {
                                                                move_checker::move_var(
                                                                    &ident, s, analyser,
                                                                );
                                                            }
                                                            _ => {
                                                                analyser.errs.push(SemanticError {
                                                                    messages: vec![
                                                                        format!("{:#?} does not implement the Copy trait(àfijọ)", ident.ident),
                                                                        format!("Perhaps you should move(kó) it")
                                                                    ],
                                                                    hints: vec![],
                                                                    span: ident.span.clone(),
                                                                });
                                                            }
                                                        }
                                                    } else {
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![
                                                                format!("{:#?} does not implement the Copy trait(àfijọ)", ident.ident),
                                                                format!("Perhaps you should move(kó) it")
                                                            ],
                                                            hints: vec![],
                                                            span: ident.span.clone(),
                                                        });
                                                    }
                                                } else {
                                                    // if Copy was implimented, and the data state is owned and not moved
                                                    // kí ọsán olówó: "Àkànbí" // also an owner and impl Copy
                                                    // kí ọsán orúkọ: olówó
                                                    if affix.is_some() {
                                                        let affix = affix.clone().unwrap();
                                                        match affix {
                                                            IdentAffix::Move(s) => {
                                                                // kí ọsán olówó: "Àkànbí"
                                                                // kí ọsán orúkọ: kó olówó
                                                                move_checker::move_var(
                                                                    &ident, s, analyser,
                                                                );
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                    // and nothing if it has no affix(move)
                                                }
                                            }
                                            DataState::Moved(span) => {
                                                // kí ọsán olówó: "Àkànbí" // also an owner but moved
                                                // ...
                                                // kí ọsán orúkọ: olówó
                                                analyser.errs.push(SemanticError {
                                                    messages: vec![format!(
                                                        "{:#?} moved already",
                                                        ident.ident
                                                    )],
                                                    hints: vec![ErrorHint {
                                                        hints: vec![format!(
                                                            "{:#?} moved here",
                                                            ident.ident
                                                        )],
                                                        span,
                                                    }],
                                                    span: ident.span.clone(),
                                                });
                                            }
                                            DataState::ImutRef
                                            | DataState::MutRef
                                            | DataState::GlobalMutRef
                                            | DataState::GlobalImutRef => {
                                                // kí &ọsán olówó: òmíràn // A ref
                                                // kí ọsán orúkọ: olówó
                                                let impl_copy =
                                                    move_checker::check_ty_copy_impl(&ty, analyser);

                                                if affix.is_some() {
                                                    let affix = affix.clone().unwrap();
                                                    match affix {
                                                        IdentAffix::Move(_) => {
                                                            // kí &ọsán olówó: òmíràn // A ref
                                                            // kí ọsán orúkọ: kó olówó
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "A refrence like {:#?} cannot be moved",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![ErrorHint {
                                                                    hints: vec![format!(
                                                                        "{:#?} was defined here",
                                                                        ident.ident
                                                                    )],
                                                                    span: ident_span,
                                                                }],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                        _ => {
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "{:#?} has an unexpected affix",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                    }
                                                } else {
                                                    // kí &ọsán olówó: òmíràn // A ref
                                                    // kí ọsán orúkọ: kó olówó // has no affix at all
                                                    if !impl_copy {
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![
                                                                format!("{:#?} does not implement the Copy trait(àfijọ)", ident.ident),
                                                                format!("Perhaps you should borrow it as well, as it cannot be moved")
                                                            ],
                                                            hints: vec![],
                                                            span: ident.span.clone(),
                                                        });
                                                    }
                                                }
                                            }
                                            DataState::Global => {
                                                if affix.is_some() {
                                                    let affix = affix.clone().unwrap();
                                                    match affix {
                                                        IdentAffix::Move(_) => {
                                                            // kárí ọsán olówó: "Àkànbí" // also an owner
                                                            // {
                                                            //      kí ọsán orúkọ: kó olówó
                                                            // }
                                                            analyser.errs.push(SemanticError {
                                                                messages: vec![format!(
                                                                    "A global variable like {:#?} cannot be moved",
                                                                    ident.ident
                                                                )],
                                                                hints: vec![ErrorHint {
                                                                    hints: vec![format!(
                                                                        "{:#?} was defined here",
                                                                        ident.ident
                                                                    )],
                                                                    span: ident_span,
                                                                }],
                                                                span: ident.span.clone(),
                                                            });
                                                        }
                                                        _ => (),
                                                    }
                                                } else {
                                                    // kárí ọsán olówó: "Àkànbí"
                                                    // {
                                                    //      kí ọsán orúkọ: olówó // no affixes
                                                    // }
                                                    let impl_copy =
                                                        move_checker::check_ty_copy_impl(
                                                            &ty, analyser,
                                                        );

                                                    if !impl_copy {
                                                        // kárí ọsán olówó: "Àkànbí" // does not impl Copy
                                                        // {
                                                        //      kí ọsán orúkọ: olówó
                                                        // }
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![
                                                                format!("{:#?} does not implement the Copy trait(àfijọ)", ident.ident),
                                                                format!("Perhaps you should borrow it as it cannot be moved")
                                                            ],
                                                            hints: vec![],
                                                            span: ident.span.clone(),
                                                        });
                                                    }
                                                    // no problem if Copy is impl
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                            Expect::Void => (),
                        }

                        // TYPE CHECKING
                        let ident_type = type_checker::get_var_type(var.unwrap());

                        if ident_type.is_some() {
                            type_checker::is_type_compatible(
                                &ty,
                                &ident_type.clone().unwrap(),
                                genrs,
                                analyser,
                            );
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Unable to the data type of variable '{:?}'",
                                    expr
                                )],
                                hints: vec![],
                                span: Span {
                                    st: Pos { column: 0, line: 0 },
                                    en: Pos { column: 0, line: 0 },
                                    file: analyser.modl.clone(),
                                }, //self.get_span_of_expr(ty),
                            });
                        }
                    } else {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "Variable '{}' not found in this scope",
                                ident.ident.clone()
                            )],
                            hints: vec![],
                            span: ident.span.clone(), //self.get_span_of_expr(ty),
                        });
                    }
                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind:
                        ExprKind::Binary {
                            lhs: _,
                            op: _,
                            rhs: _,
                        },
                    span: _,
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
                }
                Expr {
                    kind: ExprKind::StructVal { .. },
                    span,
                } => {
                    add_pars_scope(pars, analyser);

                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "This Struct is not owned by any variable which is what a reference requires",
                            )],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        // type_checker::is_type_compatible(
                        //     &ty,
                        //     &TypeSpec {
                        //         hint: TypeHint::StructLiteral { ident, args: vals },
                        //         span,
                        //     },
                        //     genrs,
                        //     analyser,
                        // );
                        match &ty.hint {
                            TypeHint::Custom {
                                import,
                                ident: par_ident,
                                generics: _,
                            } => {
                                let type_def = analyser.get_custom_type(import, par_ident, genrs);
                                if type_def.is_some() {
                                    validate_custom_type_value(
                                        &type_def.unwrap(),
                                        ty,
                                        &expr,
                                        analyser,
                                    );
                                } else {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Type definition for '{}' not found in this scope",
                                            par_ident.ident
                                        )],
                                        hints: vec![],
                                        span: par_ident.span.clone(), //self.get_span_of_expr(ty),
                                    });
                                }
                            }
                            _ => (),
                        }
                    }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind:
                        ExprKind::ElemNamespaceAccess {
                            ident,
                            access: kind,
                        },
                    span,
                } => {
                    add_pars_scope(pars, analyser);

                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "The value from the namespace is not owned by any variable which is what a reference requires",
                            )],
                            hints: vec![],
                            span, //self.get_span_of_expr(ty),
                        });
                    } else {
                        let mut val_ident = Ident {
                            ident: String::from("<Unknown>"),
                            span: Span {
                                st: Pos { column: 0, line: 0 },
                                en: Pos { column: 0, line: 0 },
                                file: analyser.modl.clone(),
                            },
                        };
                        let elem = if ident.is_some() {
                            val_ident = ident.clone().unwrap();
                            analyser.get_element(&ident.clone().unwrap())
                        } else {
                            // Infer Namespace/Type
                            // kí Àwọ̀ ire: .Funfun
                            match &ty.hint {
                                TypeHint::Custom {
                                    import,
                                    ident,
                                    generics: _,
                                } => {
                                    val_ident = ident.clone();
                                    if import.is_none() {
                                        analyser.get_element(ident)
                                    } else {
                                        let import = import.clone().unwrap();
                                        // kí irúfẹ́\Àwọ̀ ire: .Funfun

                                        let elem = analyser.get_custom_from_take(
                                            (
                                                FullTok {
                                                    kind: Token::Identifier(ident.ident.clone()),
                                                    span: ident.span.clone(),
                                                },
                                                None,
                                            ),
                                            import.1,
                                            import.0,
                                        );

                                        if elem.is_some() {
                                            match &elem {
                                                Some(TopLevel::StructDecl { .. })
                                                | Some(TopLevel::EnumDecl { .. }) => {
                                                    Some(Element::Global(elem.unwrap()))
                                                }
                                                _ => None,
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                }
                                _ => None,
                            }
                        };

                        if elem.is_some() {
                            let elem = elem.unwrap();
                            match elem {
                                Element::Global(top) => {
                                    match top {
                                        TopLevel::EnumDecl {
                                            ident: _,
                                            generic_types: _,
                                            vals: _,
                                            traits: _,
                                            public: _,
                                            associateds: _,
                                            bridges: _,
                                            info: _,
                                        } => {
                                            validate_custom_type_value(&top, ty, &expr, analyser);
                                            // if variant not found, the function calls validate_ass_fn_call() function
                                        }
                                        TopLevel::StructDecl {
                                            ident: _,
                                            generic_types: _,
                                            vals: _,
                                            traits: _,
                                            public: _,
                                            associateds,
                                            bridges: _,
                                            info: _,
                                        } => {
                                            // most be an associated function
                                            validate_assoc_fn_call(*kind, associateds, analyser);
                                        }
                                        TopLevel::StaticDef {
                                            ident,
                                            public: _,
                                            block: static_block,
                                            info: _,
                                        } => {
                                            // The block is like a seperate module with it's own namespace
                                            validate_static_access(
                                                expect,
                                                pars,
                                                &ident,
                                                ty,
                                                static_block,
                                                &kind,
                                                expect_state,
                                                genrs,
                                                analyser,
                                            );
                                        }
                                        TopLevel::TakeStmt {
                                            take,
                                            from,
                                            lib,
                                            info: _,
                                        } => {
                                            let source = analyser
                                                .get_namespace_from_take(&take, &from, &lib);

                                            if source.is_some() {
                                                let top = source.unwrap();
                                                match &top {
                                                    TopLevel::EnumDecl {
                                                        ident: _,
                                                        generic_types: _,
                                                        vals: _,
                                                        traits: _,
                                                        public: _,
                                                        associateds: _,
                                                        bridges: _,
                                                        info: _,
                                                    } => {
                                                        validate_custom_type_value(
                                                            &top, ty, &expr, analyser,
                                                        );
                                                        // if variant not found, the function calls validate_ass_fn_call() function
                                                    }
                                                    TopLevel::StructDecl {
                                                        ident: _,
                                                        generic_types: _,
                                                        vals: _,
                                                        traits: _,
                                                        public: _,
                                                        associateds,
                                                        bridges: _,
                                                        info: _,
                                                    } => {
                                                        // most be an associated function
                                                        validate_assoc_fn_call(
                                                            *kind,
                                                            associateds.clone(),
                                                            analyser,
                                                        );
                                                    }
                                                    TopLevel::StaticDef {
                                                        ident,
                                                        public: _,
                                                        block: static_block,
                                                        info: _,
                                                    } => {
                                                        // The block is like a seperate module with it's own namespace
                                                        validate_static_access(
                                                            expect,
                                                            pars,
                                                            &ident,
                                                            ty,
                                                            static_block.clone(),
                                                            &kind,
                                                            expect_state,
                                                            genrs,
                                                            analyser,
                                                        );
                                                    }
                                                    _ => (),
                                                }
                                            } else {
                                                analyser.errs.push(SemanticError {
                                                    messages: vec![format!(
                                                        "{} is not found in {}",
                                                        take.0.kind, from
                                                    )],
                                                    hints: vec![],
                                                    span: take.0.span,
                                                });
                                            }
                                        }
                                        _ => {
                                            // Err
                                            todo!("Not yet implemented");
                                        }
                                    }
                                }
                                Element::Local(_) => {
                                    todo!("todo");
                                }
                            }
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "{} is not found in scope",
                                    val_ident.ident
                                )],
                                hints: vec![],
                                span: val_ident.span,
                            });
                        }
                    }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::ObjAssess { obj, fields },
                    ..
                } => {
                    add_pars_scope(pars, analyser);
                    // let name;

                    validate_assess_expr(&obj, fields, expect, genrs, expr, pars, analyser);

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::Block(block),
                    ..
                } => {
                    add_pars_scope(pars, analyser);

                    // if *its_ref {
                    //     analyser.errs.push(SemanticError {
                    //         file: analyser.modl.clone(),
                    //         messages: vec![format!(
                    //             "This Block (on line {}) is not owned by any variable which is what a reference requires",
                    //             span.st.line
                    //         )],
                    //         hints: vec![],
                    //         span,
                    //     });
                    // } else {
                    analyse_block(expect, block, genrs, pars, analyser); // always some for fun
                                                                         // }

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind:
                        ExprKind::ArgBuffer {
                            asy: _,
                            generics: buffer_genrs,
                            args,
                            name,
                        },
                    ..
                } => {
                    add_pars_scope(pars, analyser);

                    validate_fn_call_expr(name, expect, genrs, buffer_genrs, args, false, analyser);

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind: ExprKind::AsType { expr, ty: as_ty },
                    ..
                } => {
                    add_pars_scope(pars, analyser);

                    type_checker::validate_ty_conversion(expect, *expr, *as_ty.clone(), analyser);

                    type_checker::is_type_compatible(ty, &*as_ty, &vec![], analyser);

                    if pars.len() > 0 {
                        analyser.symbol_table.pop(); // the parameter scope
                    }
                }
                Expr {
                    kind:
                        ExprKind::If {
                            bool_expr,
                            exe,
                            else_ifs,
                            else_exe,
                        },
                    span,
                } => {
                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "If Expr from line {} to {} is not owned by any variable which is what a reference requires",
                                span.st.line, span.en.line
                            )],
                            hints: vec![],
                            span: Span {
                                st: Pos { column: 0, line: 0 },
                                en: Pos { column: 0, line: 0 },
                                file: analyser.modl.clone()
                            }, //self.get_span_of_expr(ty),
                        });
                    } else {
                        // tí bool_expr...
                        validate_assignment(
                            &TypeSpec {
                                hint: TypeHint::Bool,
                                span: Span {
                                    st: Pos { column: 0, line: 0 },
                                    en: Pos { column: 0, line: 0 },
                                    file: analyser.modl.clone(),
                                },
                            },
                            DataState::Owner,
                            &bool_expr,
                            &vec![],
                            &vec![],
                            analyser,
                        );

                        check_for_val_compatibility(
                            *exe.clone(),
                            &expect,
                            genrs,
                            &vec![],
                            analyser,
                        );

                        // àìjébè tí bool_expr: {...}
                        for else_if in &else_ifs {
                            match else_if {
                                ElseIf {
                                    bool_expr,
                                    exe,
                                    span: _,
                                } => {
                                    validate_assignment(
                                        &TypeSpec {
                                            hint: TypeHint::Bool,
                                            span: Span {
                                                st: Pos { column: 0, line: 0 },
                                                en: Pos { column: 0, line: 0 },
                                                file: analyser.modl.clone(),
                                            },
                                        },
                                        DataState::Owner,
                                        bool_expr,
                                        &vec![],
                                        &vec![],
                                        analyser,
                                    );

                                    check_for_val_compatibility(
                                        exe.clone(),
                                        &expect,
                                        genrs,
                                        &vec![],
                                        analyser,
                                    );
                                }
                            }
                        }
                        // àìjébè {...}
                        if *else_exe != None {
                            // no else arm
                            check_for_val_compatibility(
                                else_exe.unwrap(),
                                &expect,
                                genrs,
                                &vec![],
                                analyser,
                            );
                        } else {
                            let st = bool_expr.span.st;
                            let en = *&exe.span.en;

                            analyser.errs.push(SemanticError {
                                messages: vec![format!("If expression is missing the else block")],
                                hints: vec![],
                                span: Span {
                                    st,
                                    en,
                                    file: bool_expr.span.file.clone(),
                                },
                            });
                        }
                    }
                }
                Expr {
                    kind: ExprKind::Match { var, arms },
                    span,
                } => {
                    if *expect_state == DataState::ImutRef || *expect_state == DataState::MutRef {
                        analyser.errs.push(SemanticError {
                            messages: vec![
                                format!(
                                    "Match Expr from line {} to {} is not owned by any variable which is what a reference requires",
                                    span.st.line, span.en.line
                                )
                            ],
                            hints: vec![],
                            span: Span {
                                st: Pos { column: 0, line: 0 },
                                en: Pos { column: 0, line: 0 },
                                file: analyser.modl.clone()
                            }, //self.get_span_of_expr(ty),
                        });
                    } else {
                        let variable = get_var(&var, analyser);
                        if variable.is_some() {
                            let var_type = type_checker::get_var_type(variable.clone().unwrap());

                            if var_type.is_some() {
                                let var_type = var_type.unwrap();

                                for arm in &arms {
                                    match arm {
                                        Case { expr, exe } => {
                                            validate_match_arm(
                                                &variable.clone().unwrap().state,
                                                &var_type,
                                                genrs,
                                                expr,
                                                exe,
                                                expect.clone(),
                                                analyser,
                                            );
                                        }
                                    }
                                }
                            } else {
                                analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Unable to determine the type of variable {}",
                                        var.ident.clone()
                                    )],
                                    hints: vec![],
                                    span: var.span.clone(),
                                });
                            }
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Variable {} not found in this scope",
                                    var.ident.clone()
                                )],
                                hints: vec![],
                                span: var.span.clone(),
                            });
                        }
                    }
                }
                Expr {
                    kind: ExprKind::Address(addr_status, expr),
                    span: addr_span,
                } => {
                    // @expr
                    match expr.as_ref() {
                        Expr {
                            kind: ExprKind::Ident { affix: _, ident },
                            span: _ident_span,
                        } => {
                            match ty {
                                TypeSpec { hint, span } => {
                                    match hint {
                                        TypeHint::RawPtr { tysp, mutb } => {
                                            // now they are both raw pointers
                                            let mut satisfied = true;
                                            let mut is_imut_addr = true;
                                            match addr_status {
                                                MutStatus::Mut(_) => {
                                                    if !mutb {
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![format!(
                                                                "Expected an immutable raw pointer but found a mutable one"
                                                            )],
                                                            hints: vec![],
                                                            span: addr_span.clone()
                                                        });
                                                        satisfied = false;
                                                    }
                                                    is_imut_addr = false;
                                                }
                                                MutStatus::Const => {
                                                    if *mutb {
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![format!(
                                                                "Expected a mutable raw pointer but found an immutable one"
                                                            )],
                                                            hints: vec![],
                                                            span: addr_span.clone()
                                                        });
                                                        satisfied = false;
                                                    }
                                                }
                                            }

                                            if satisfied {
                                                //check the base type
                                                let var = analyser.get_var(ident);

                                                if var.is_some() {
                                                    match var.unwrap().0 {
                                                        Variable {
                                                            state,
                                                            ty,
                                                            value: _,
                                                            is_mutable,
                                                            ident_span,
                                                        } => {
                                                            if ty.hint == tysp.hint {
                                                                if !is_imut_addr {
                                                                    if !is_mutable.unwrap_or(false)
                                                                    {
                                                                        // if the address is mutable and(but) the variable is not mutable
                                                                        // then we need to check if perhaps the variable is a mut reference
                                                                        match state {
                                                                            DataState::MutRef => (),
                                                                            _ => {
                                                                                //Error:
                                                                                analyser.errs.push(SemanticError {
                                                                                    messages: vec![format!(
                                                                                        "Cannot get {} as mutable pointer since it wasn't declared mutable",
                                                                                        ident.ident
                                                                                    )],
                                                                                    hints: vec![
                                                                                        ErrorHint {
                                                                                            hints: vec![format!(
                                                                                                "The variable was defined here"
                                                                                            )],
                                                                                            span: ident_span
                                                                                        }
                                                                                    ],
                                                                                    span: ident.span.clone(),
                                                                                });
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                analyser.errs.push(SemanticError {
                                                                    messages: vec![format!(
                                                                        "Raw poiter type of {} does not match the required {}",
                                                                        ident.ident,
                                                                        hint
                                                                    )],
                                                                    hints: vec![
                                                                        ErrorHint {
                                                                            hints: vec![format!(
                                                                                "The variable was defined here"
                                                                            )],
                                                                            span: ident_span
                                                                        }
                                                                    ],
                                                                    span: ident.span.clone(),
                                                                });
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    analyser.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Variable {} not found in scope",
                                                            ident.ident
                                                        )],
                                                        hints: vec![],
                                                        span: ident.span.clone(),
                                                    });
                                                }
                                            }
                                        }
                                        _ => {
                                            // Error:
                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "Expected {} but found a raw pointer instead",
                                                    hint
                                                )],
                                                hints: vec![],
                                                span: span.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => (), // always an ident
                    }
                }
                Expr {
                    kind: ExprKind::Foo,
                    ..
                } => (), // NEEDS TO PASS
                oth => {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!("The expression {} is not clear", oth.kind)],
                        hints: vec![],
                        span: oth.span,
                    });
                }
            }
        }
        Expect::Void => {
            match expr {
                Expr {
                    kind: ExprKind::Block(block),
                    ..
                } => {
                    analyse_block(expect, block, genrs, pars, analyser); // always some for fun
                }
                _ => {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!("Expected Void but found '{:?}'", expr)],
                        hints: vec![],
                        span: Span {
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: analyser.modl.clone(),
                        }, //self.get_span_of_expr(ty),
                    });
                }
            }
        }
    }
    // Try not to use this function directly
    // use validate_assignment() instead
    // match expr.clone() {}
}

fn add_to_global_scope(
    ident: Ident,
    glo_elem: TopLevel,
    // span: &Span,
    analyser: &mut Analyser,
) {
    if analyser.symbol_table[0].contains_key(&ident.ident) {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is a clash of name; Another global element already as the name '{:?}'",
                ident
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
    } else {
        analyser.symbol_table[0].insert(ident.ident, Element::Global(glo_elem));
    }
}

fn init_global_scope(
    modl: Option<&Mod>,
    static_block: Option<&CustomBlock>, // can only have either hir or static_block not both
    analyser: &mut Analyser,
) {
    if modl.is_some() {
        let modl = modl.unwrap();
        // For Global variables
        for each in &modl.globals {
            match each {
                TopLevel::GlobalDecl {
                    interpret: _,
                    state: _,
                    ty: _,
                    name, // should come in handy with span, for precise error handling
                    value: _,
                    public: _,
                    info: _,
                } => {
                    add_to_global_scope(name.clone(), each.clone(), analyser);
                }
                _ => (),
            }
        }

        // For Global functions
        for each in &modl.fns {
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

        // For Platforms
        // for each in self.hirs[0].platform_n_s {
        //     match each {
        //         TopLevel::PlatformNS { arms, span } => {
        //             self.add_to_global_scope(ident, Token::Function, span);
        //         }
        //         _ => (),
        //     }
        // }

        // For takes
        for each in &modl.takes {
            match each {
                TopLevel::TakeStmt {
                    take,
                    from: _,
                    lib: _,
                    info: _,
                } => {
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
                    add_to_global_scope(ident, each.clone(), analyser);
                }
                _ => (),
            }
        }

        // For traits
        for each in &modl.traits {
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
        for each in &modl.workers {
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
        for each in &modl.statics {
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

        // For custom types
        for each in &modl.custom_tys {
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
                } => {
                    add_to_global_scope(ident.clone(), each.clone(), analyser);
                }
                TopLevel::StructDecl {
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
    } else {
        // then it's static_block
        // For Global variables
        for each in static_block.unwrap().globals.clone() {
            match each {
                CustomLevel::GlobalDecl {
                    interpret,
                    state,
                    ty,
                    name, // should come with span, for precise error handling
                    value,
                    public,
                    info,
                } => {
                    if interpret {
                        match ty {
                            TypeSpec {
                                hint: TypeHint::Void,
                                span: _,
                            } => {
                                analyser.errs.push(SemanticError {
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

                    validate_assignment(&ty, state.clone(), &value, &vec![], &vec![], analyser);
                    add_to_global_scope(
                        name.clone(),
                        TopLevel::GlobalDecl {
                            interpret,
                            state,
                            ty,
                            name,
                            value,
                            public,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }

        // For Global functions
        for each in static_block.unwrap().funcs.clone() {
            match each {
                CustomLevel::FuncDecl {
                    is_async,
                    expect,
                    ret_state,
                    ident,
                    generics,
                    is_public,
                    is_visible: _,
                    is_extern,
                    par,
                    worker_usage,
                    block,
                    info,
                } => {
                    add_to_global_scope(
                        ident.clone(),
                        TopLevel::FuncDecl {
                            is_async,
                            expect,
                            ret_state,
                            ident,
                            generics,
                            is_visible: false,
                            is_public,
                            is_extern,
                            par,
                            worker_usage,
                            block,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }

        // For takes
        for each in static_block.unwrap().takes.clone() {
            match each {
                CustomLevel::TakeStmt {
                    take,
                    from,
                    lib,
                    info,
                } => {
                    let ident;
                    if take.1.is_some() {
                        ident = take.1.clone().unwrap();
                    } else {
                        match take.0.clone() {
                            FullTok {
                                kind: Token::Identifier(s),
                                span,
                            } => {
                                ident = Ident { ident: s, span };
                            }
                            _ => {
                                ident = Ident {
                                    ident: String::from(""),
                                    span: info.span.clone(),
                                };
                            }
                        }
                    }
                    add_to_global_scope(
                        ident,
                        TopLevel::TakeStmt {
                            take,
                            from,
                            lib,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }

        // For traits
        for each in static_block.unwrap().traits.clone() {
            match each {
                CustomLevel::TraitDef {
                    ident,
                    generic_types,
                    interprets,
                    methods,
                    is_public,
                    info,
                } => {
                    add_to_global_scope(
                        ident.clone(),
                        TopLevel::TraitDef {
                            ident,
                            generic_types,
                            interprets,
                            methods,
                            is_public,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }

        // For workers
        for each in static_block.unwrap().workers.clone() {
            match each {
                CustomLevel::WorkerDef {
                    ident,
                    func_name,
                    public,
                    block,
                    info,
                } => {
                    add_to_global_scope(
                        ident.clone(),
                        TopLevel::WorkerDef {
                            ident,
                            func_name,
                            public,
                            block,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }

        // For custom types
        for each in static_block.unwrap().custom_tys.clone() {
            match each {
                CustomLevel::EnumDecl {
                    ident,
                    generic_types,
                    vals,
                    traits,
                    public,
                    associateds,
                    bridges,
                    info,
                } => {
                    add_to_global_scope(
                        ident.clone(),
                        TopLevel::EnumDecl {
                            ident,
                            generic_types,
                            vals,
                            traits,
                            public,
                            associateds,
                            bridges,
                            info,
                        },
                        analyser,
                    );
                }
                CustomLevel::StructDecl {
                    ident,
                    generic_types,
                    vals,
                    traits,
                    public,
                    associateds,
                    bridges,
                    info,
                } => {
                    add_to_global_scope(
                        ident.clone(),
                        TopLevel::StructDecl {
                            ident,
                            generic_types,
                            vals,
                            traits,
                            public,
                            associateds,
                            bridges,
                            info,
                        },
                        analyser,
                    );
                }
                _ => (),
            }
        }
    }
}

fn _get_static_def(ident: &Ident, analyser: &mut Analyser) -> Option<TopLevel> {
    let top_level = &analyser.symbol_table[0];
    let element = top_level.get(&ident.ident);
    if element.is_some() {
        match element.unwrap().clone() {
            Element::Global(elem) => match &elem {
                TopLevel::StaticDef {
                    ident: _,
                    public: _,
                    block: _,
                    info: _,
                } => {
                    return Some(elem);
                }
                TopLevel::TakeStmt {
                    take,
                    from,
                    lib,
                    info: _,
                } => {
                    match from {
                        ModPath::Ident(mod_ident) => {
                            let unit = analyser.unit.get(&mod_ident.ident);
                            if unit.is_some() {
                                match unit.unwrap() {
                                    NameSpace::Singular { name: _, file } => {
                                        if *lib == None {
                                            for st in &file.statics {
                                                match st {
                                                    TopLevel::StaticDef {
                                                        ident,
                                                        public,
                                                        block: _,
                                                        info: _,
                                                    } => {
                                                        match take.0.clone().kind {
                                                            Token::Identifier(s) => {
                                                                if s == ident.ident {
                                                                    // found it
                                                                    if public.clone() {
                                                                        return Some(st.clone());
                                                                    } else {
                                                                        analyser.errs.push(SemanticError {
                                                                                messages: vec![format!(
                                                                                    "Static '{:?}' is not public",
                                                                                    ident
                                                                                )],
                                                                                hints: vec![],
                                                                                span: ident.span.clone(),
                                                                            });
                                                                    }
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        } else {
                                            // Error: Wrong module path
                                        }
                                    }
                                    NameSpace::Plural { name: _, files: _ } => {}
                                    NameSpace::Lib { .. } => {}
                                }
                            } else {
                                analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Can't find module '{}'",
                                        mod_ident.ident
                                    )],
                                    hints: vec![],
                                    span: mod_ident.span.clone(),
                                }); // for now
                            }
                        }
                        ModPath::Str(_) => {
                            todo!("todo")
                        }
                        ModPath::UnExpected => {
                            todo!("Hmm");
                        }
                        ModPath::FromLibFolder(_, _) => {
                            todo!("Take from library folder")
                        }
                    }
                }
                _ => (),
            },
            Element::Local(_) => (), // it's a toplevel only element
        }
    }
    return None; // by default
}

pub fn get_func_def(ident: &Ident, analyser: &mut Analyser) -> Option<TopLevel> {
    let element = analyser.get_element(ident);
    if element.is_some() {
        match element.unwrap() {
            Element::Global(elem) => match &elem {
                TopLevel::FuncDecl {
                    is_async: _,
                    expect: _,
                    ret_state: _,
                    ident: _,
                    generics: _,
                    is_visible: _,
                    is_public: _,
                    is_extern: _,
                    par: _,
                    worker_usage: _,
                    block: _,
                    info: _,
                } => {
                    return Some(elem);
                }
                TopLevel::TakeStmt {
                    take,
                    from,
                    lib,
                    info: _,
                } => {
                    match from {
                        ModPath::Ident(mod_ident) => {
                            let modl = analyser.unit.get(&mod_ident.ident);
                            if modl.is_some() {
                                match modl.unwrap() {
                                    NameSpace::Singular { name: _, file } => {
                                        if *lib == None {
                                            for f in &file.fns {
                                                match f {
                                                    TopLevel::FuncDecl {
                                                        is_async: _,
                                                        expect: _,
                                                        ret_state: _,
                                                        ident,
                                                        generics: _,
                                                        is_visible: _,
                                                        is_public,
                                                        is_extern: _,
                                                        par: _,
                                                        worker_usage: _,
                                                        block: _,
                                                        info: _,
                                                    } => {
                                                        match take.0.clone().kind {
                                                            Token::Identifier(s) => {
                                                                if s == ident.ident {
                                                                    // found it
                                                                    if is_public.clone() {
                                                                        return Some(f.clone());
                                                                    } else {
                                                                        analyser.errs.push(SemanticError {
                                                                            messages: vec![format!(
                                                                                "Function '{}' is not public",
                                                                                ident.ident
                                                                            )],
                                                                            hints: vec![],
                                                                            span: ident.span.clone(),
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        } else {
                                            //
                                        }
                                    }
                                    NameSpace::Plural { name: _, files: _ } => {}
                                    NameSpace::Lib {
                                        name,
                                        manifest: _,
                                        entry,
                                        src: _,
                                    } => {
                                        if lib.is_none() {
                                            // from entry
                                            for each in &entry.funcs {
                                                if *each.0 == ident.ident {
                                                    // found it
                                                    match each.1.clone() {
                                                        FuncDeclInfo {
                                                            ident_span,
                                                            is_async,
                                                            expect,
                                                            ret_state,
                                                            generics,
                                                            par,
                                                            is_extern,
                                                            is_public,
                                                            is_visible,
                                                            worker_usage,
                                                            block: _,
                                                            stm_span,
                                                        } => {
                                                            // reconstruct TopLevel::FuncDecl
                                                            let top = TopLevel::FuncDecl {
                                                                is_async,
                                                                expect,
                                                                ret_state,
                                                                ident: Ident {
                                                                    ident: each.0.clone(),
                                                                    span: ident_span,
                                                                },
                                                                generics,
                                                                is_visible,
                                                                is_public,
                                                                is_extern,
                                                                par,
                                                                worker_usage,
                                                                block: Expr {
                                                                    kind: ExprKind::Foo,
                                                                    span: Span {
                                                                        st: Pos {
                                                                            column: 0,
                                                                            line: 0,
                                                                        },
                                                                        en: Pos {
                                                                            column: 0,
                                                                            line: 0,
                                                                        },
                                                                        file: format!(
                                                                            "Lib: {}",
                                                                            name
                                                                        ),
                                                                    },
                                                                },
                                                                info: StmtInfo {
                                                                    span: stm_span,
                                                                    docs: vec![],
                                                                },
                                                            };

                                                            return Some(top);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                analyser.errs.push(SemanticError {
                                    messages: vec![format!("Can't find module '{:?}'", from)],
                                    hints: vec![],
                                    span: mod_ident.span.clone(),
                                }); // for now
                            }
                        }
                        ModPath::Str(_) => {
                            todo!("todo")
                        }
                        ModPath::UnExpected => {
                            todo!("Haha");
                        }
                        ModPath::FromLibFolder(_, _) => {
                            todo!("Take from library folder")
                        }
                    }
                }
                _ => (),
            },
            Element::Local(elem) => match elem {
                BlockLevel::FuncDecl {
                    expect,
                    ident,
                    generics,
                    is_extern,
                    par,
                    worker_usage,
                    block,
                    info,
                } => {
                    return Some(TopLevel::FuncDecl {
                        is_async: false,
                        expect,
                        ret_state: DataState::Owner,
                        ident,
                        generics,
                        is_visible: false,
                        is_public: false,
                        is_extern,
                        par,
                        worker_usage,
                        block,
                        info,
                    });
                }
                BlockLevel::TakeStmt {
                    take,
                    from,
                    lib,
                    info: _,
                } => {
                    match from {
                        ModPath::Ident(mod_ident) => {
                            let modl = analyser.unit.get(&mod_ident.ident);
                            if modl.is_some() {
                                match modl.unwrap() {
                                    NameSpace::Singular { name: _, file } => {
                                        if lib == None {
                                            for f in &file.fns {
                                                match f {
                                                    TopLevel::FuncDecl {
                                                        is_async: _,
                                                        expect: _,
                                                        ret_state: _,
                                                        ident,
                                                        generics: _,
                                                        is_visible: _,
                                                        is_public,
                                                        is_extern: _,
                                                        par: _,
                                                        worker_usage: _,
                                                        block: _,
                                                        info: _,
                                                    } => {
                                                        match take.0.clone().kind {
                                                            Token::Identifier(s) => {
                                                                if s == ident.ident {
                                                                    // found it
                                                                    if is_public.clone() {
                                                                        return Some(f.clone());
                                                                    } else {
                                                                        analyser.errs.push(SemanticError {
                                                                            messages: vec![format!(
                                                                                "Function '{}' is not public",
                                                                                ident.ident
                                                                            )],
                                                                            hints: vec![],
                                                                            span: ident.span.clone(),
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        } else {
                                            //
                                        }
                                    }
                                    NameSpace::Plural { name: _, files: _ } => {}
                                    NameSpace::Lib { .. } => {}
                                }
                            } else {
                                analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Can't find module '{:?}'",
                                        mod_ident.ident
                                    )],
                                    hints: vec![],
                                    span: mod_ident.span.clone(),
                                }); // for now
                            }
                        }
                        ModPath::Str(_string) => {
                            todo!("todo")
                        }
                        ModPath::UnExpected => {
                            todo!("Nice");
                        }
                        ModPath::FromLibFolder(_, _) => {
                            todo!("Take from lib folder")
                        }
                    }
                }
                _ => (),
            },
        }
    }
    return None; // by default
}

fn get_var(ident: &Ident, analyser: &mut Analyser) -> Option<Variable> {
    // let mut i = 0;
    for scope in analyser.symbol_table.iter().rev() {
        let element = scope.get(&ident.ident);
        if element.is_some() {
            match element.unwrap().clone() {
                Element::Global(elem) => {
                    match elem {
                        TopLevel::GlobalDecl {
                            interpret: _,
                            state,
                            ty,
                            name,
                            value,
                            public: _,
                            info: _,
                        } => {
                            return Some(Variable {
                                state,
                                ty,
                                value,
                                is_mutable: None,
                                ident_span: name.span,
                            });
                        }
                        TopLevel::TakeStmt {
                            take,
                            from,
                            lib,
                            info: _,
                        } => {
                            match from {
                                ModPath::Ident(mod_ident) => {
                                    let modl = analyser.unit.get(&mod_ident.ident);
                                    if modl.is_some() {
                                        match modl.unwrap() {
                                            NameSpace::Singular { name: _, file } => {
                                                if lib == None {
                                                    for g in &file.globals {
                                                        match g {
                                                            TopLevel::GlobalDecl {
                                                                interpret: _,
                                                                state,
                                                                ty,
                                                                name,
                                                                value,
                                                                public,
                                                                info: _,
                                                            } => {
                                                                match take.0.clone().kind {
                                                                    Token::Identifier(s) => {
                                                                        if s == name.ident {
                                                                            // found it
                                                                            if public.clone() {
                                                                                return Some(Variable {
                                                                                state: state.clone(),
                                                                                ty: ty.clone(),
                                                                                value: value.clone(),
                                                                                is_mutable: None,
                                                                                ident_span: name
                                                                                    .span
                                                                                    .clone(),
                                                                            });
                                                                            } else {
                                                                                analyser.errs.push(
                                                                                    SemanticError {
                                                                                        messages: vec![
                                                                                            format!(
                                                                                        "'{:?}', is not public",
                                                                                        name
                                                                                    ),
                                                                                        ],
                                                                                        hints: vec![],
                                                                                        span: name.span.clone(),
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                    }
                                                                    _ => (),
                                                                }
                                                            }
                                                            _ => (),
                                                        }
                                                    }
                                                } else {
                                                    // Error:
                                                }
                                            }
                                            NameSpace::Plural { name: _, files: _ } => {}
                                            NameSpace::Lib { .. } => {}
                                        }
                                    } else {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Can't find module '{:?}'",
                                                mod_ident.ident
                                            )],
                                            hints: vec![],
                                            span: mod_ident.span.clone(),
                                        }); // for now
                                    }
                                }
                                ModPath::Str(_string) => {
                                    todo!("Module path from folder")
                                }
                                ModPath::UnExpected => {
                                    todo!("Unexpected module path");
                                }
                                ModPath::FromLibFolder(_, _) => {
                                    todo!("Take from library folder")
                                }
                            }
                        }
                        _ => (),
                    }
                }
                Element::Local(elem) => match elem {
                    BlockLevel::VarDecl {
                        interpret: _,
                        state,
                        ty,
                        name,
                        value,
                        mutable,
                        info: _,
                    } => {
                        return Some(Variable {
                            state,
                            ty,
                            value,
                            is_mutable: Some(mutable.clone()),
                            ident_span: name.span,
                        });
                    }
                    BlockLevel::UnInitVarDecl {
                        state,
                        ty,
                        name,
                        mutable,
                        info,
                    } => {
                        return Some(Variable {
                            state,
                            ty,
                            value: Expr {
                                kind: ExprKind::Undefined,
                                span: info.span,
                            },
                            is_mutable: Some(mutable.clone()),
                            ident_span: name.span,
                        });
                    }
                    BlockLevel::Param {
                        state,
                        ty,
                        name,
                        mutable,
                        info: _,
                    } => {
                        return Some(Variable {
                            state,
                            ty,
                            value: Expr {
                                kind: ExprKind::Foo,
                                span: name.span.clone(),
                            },
                            is_mutable: Some(mutable),
                            ident_span: name.span,
                        });
                    }
                    _ => (),
                },
            }
        }

        // i += 1;
    }
    return None; // by default
}

fn get_var_expr(ident: &Ident, analyser: &mut Analyser) -> Option<Expr> {
    let var = get_var(ident, analyser);

    if var.is_some() {
        match var.unwrap() {
            Variable {
                state: _,
                ty: _,
                value,
                is_mutable: _,
                ident_span: _,
            } => Some(value),
        }
    } else {
        None
    }
}

fn var_not_initialized(
    ident: &Ident,
    // span: &StmtSpan,
    analyser: &mut Analyser,
) -> bool {
    let val = get_var_expr(ident, analyser);

    if val.is_some() {
        match val.unwrap() {
            Expr {
                kind: ExprKind::Undefined,
                ..
            } => true,
            _ => false,
        }
    } else {
        analyser.errs.push(SemanticError {
            messages: vec![format!(
                "There is no variable with the name '{:?}' in this scope",
                ident
            )],
            hints: vec![],
            span: ident.span.clone(),
        });
        false
    }
}

fn validate_match_arm(
    state: &DataState,
    ty: &TypeSpec,
    genrs: &Vec<Generic>,
    expr: &Expr,
    exe: &Expr,
    expect: Expect,
    analyser: &mut Analyser,
) {
    let mut arm_params = vec![];
    match ty {
        TypeSpec { hint, span: _ } => match hint {
            TypeHint::Custom {
                import,
                ident,
                generics,
            } => {
                let cust = analyser.get_custom_type(import, ident, &vec![]);

                if cust.is_some() {
                    let cust = cust.unwrap();

                    match cust {
                        TopLevel::EnumDecl {
                            ident,
                            generic_types,
                            vals,
                            traits: _,
                            public: _,
                            associateds: _,
                            bridges: _,
                            info: _,
                        } => {
                            match expr {
                                Expr {
                                    kind:
                                        ExprKind::ElemNamespaceAccess {
                                            ident: val_ident,
                                            access: kind,
                                        },
                                    span: val_span,
                                } => {
                                    let mut check = true;
                                    if val_ident.is_some() {
                                        let val_ident = val_ident.clone().unwrap();

                                        if ident.ident != val_ident.ident {
                                            check = false; // there is no need

                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "Expected {} variant pattern but found {}",
                                                    ident.ident.clone(),
                                                    val_ident.ident
                                                )],
                                                hints: vec![],
                                                span: ident.span.clone(),
                                            });
                                        }
                                    }

                                    if check {
                                        validate_num_of_par_generics(
                                            &generic_types,
                                            generics,
                                            ident.clone(),
                                            analyser,
                                        );

                                        let mut found = false;
                                        match *kind.clone() {
                                            Expr {
                                                kind:
                                                    ExprKind::ArgBuffer {
                                                        args,
                                                        name: val_name,
                                                        ..
                                                    },
                                                span: _,
                                            } => {
                                                for val in vals {
                                                    match val {
                                                        EnumVariants::Tuple { name, fields } => {
                                                            if name.ident == val_name.ident {
                                                                found = true;
                                                                if args.len() != fields.len() {
                                                                    analyser.errs.push(SemanticError {
                                                                        messages: vec![format!(
                                                                            "Expected {} but found {} entries",
                                                                            fields.len(),
                                                                            args.len()
                                                                        )],
                                                                        hints: vec![],
                                                                        span: name.span,
                                                                    });
                                                                } else {
                                                                    // When found, we need add each arg as parameters
                                                                    let mut i = 0;
                                                                    for arg in args {
                                                                        let span = arg.span;

                                                                        match arg {
                                                                            Expr {
                                                                                kind:
                                                                                    ExprKind::Ident {
                                                                                        affix,
                                                                                        ident,
                                                                                    },
                                                                                ..
                                                                            } => {
                                                                                if affix.is_some() {
                                                                                    analyser.errs.push(SemanticError {
                                                                                        messages: vec![format!(
                                                                                            "Affixes are unexpected for this identifier"
                                                                                        )],
                                                                                        hints: vec![],
                                                                                        span: span.clone(),
                                                                                    });
                                                                                }
                                                                                let ty = fields[i]
                                                                                    .clone();

                                                                                arm_params.push(
                                                                                    InterfaceParam { ident: ident.clone(), state: state.clone(), ty: ty.1, is_mut: true, span: ident.span.clone() }
                                                                                );
                                                                            }
                                                                            _ => {
                                                                                analyser.errs.push(SemanticError {
                                                                                    messages: vec![format!(
                                                                                        "Expected an Identifier"
                                                                                    )],
                                                                                    hints: vec![],
                                                                                    span,
                                                                                });
                                                                            }
                                                                        }

                                                                        i += 1;
                                                                    }
                                                                }
                                                                break;
                                                            }
                                                        }
                                                        _ => (),
                                                    }
                                                }
                                            }
                                            Expr {
                                                kind: ExprKind::Ident { affix: _, ident },
                                                span: _,
                                            } => {
                                                for val in vals {
                                                    match val {
                                                        EnumVariants::Simple(def_variant_ident) => {
                                                            if def_variant_ident.ident
                                                                == ident.ident
                                                            {
                                                                found = true;
                                                                break;
                                                            }
                                                        }
                                                        _ => (),
                                                    }
                                                }
                                            }
                                            Expr {
                                                kind: ExprKind::StructVal { ident, vals: strts },
                                                span: _,
                                            } => {
                                                for val in &vals {
                                                    match val {
                                                        EnumVariants::Record { name, fields } => {
                                                            if name.ident
                                                                == ident.clone().unwrap().ident
                                                            // ident is always some
                                                            {
                                                                found = true;

                                                                if strts.len() == fields.len() {
                                                                    let mut assigned_fields =
                                                                        Vec::new();

                                                                    for strt in &strts {
                                                                        match strt {
                                                                            FieldAssignm {
                                                                                ident,
                                                                                expr: _,
                                                                            } => {
                                                                                if assigned_fields
                                                                                    .contains(
                                                                                        &ident
                                                                                            .ident,
                                                                                    )
                                                                                {
                                                                                    analyser.errs.push(SemanticError {
                                                                                        messages: vec![format!(
                                                                                            "Field {} had already been represented",
                                                                                            ident.ident.clone()
                                                                                        )],
                                                                                        hints: vec![],
                                                                                        span: ident.span.clone()
                                                                                    });
                                                                                } else {
                                                                                    assigned_fields.push(ident.ident.clone());
                                                                                    let mut found =
                                                                                        false;

                                                                                    for field in
                                                                                        fields
                                                                                    {
                                                                                        match field {
                                                                                            Parameter { ident: par_ident, state: _, ty: _ } => {
                                                                                                if par_ident.ident == ident.ident {
                                                                                                    found = true;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }

                                                                                    if !found {
                                                                                        analyser.errs.push(SemanticError {
                                                                                            messages: vec![format!(
                                                                                                "Field {} can't be found",
                                                                                                ident.ident.clone()
                                                                                            )],
                                                                                            hints: vec![],
                                                                                            span: ident.span.clone()
                                                                                        });
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    analyser.errs.push(SemanticError {
                                                                        messages: vec![format!(
                                                                            "Expected {} but found {} fields",
                                                                            fields.len(), strts.len()
                                                                        )],
                                                                        hints: vec![],
                                                                        span: ident.clone().unwrap().span,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                        _ => (),
                                                    }
                                                }
                                            }
                                            _ => (),
                                        }

                                        if !found {
                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!("Unmatched pattern",)],
                                                hints: vec![],
                                                span: val_span.clone(),
                                            });
                                        }
                                    }
                                }
                                Expr {
                                    kind: ExprKind::Default,
                                    ..
                                } => (),
                                _ => {
                                    let span = expr.span.clone();
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!("Unmatched pattern",)],
                                        hints: vec![],
                                        span,
                                    });
                                }
                            }
                        }
                        // TopLevel::StructDecl {
                        //     ident,
                        //     generic_types,
                        //     vals,
                        //     worker_usage,
                        //     public,
                        //     methods,
                        //     span,
                        // } => {}
                        _ => (),
                    }
                } else {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!("{} not found in scope", ident.ident.clone())],
                        hints: vec![],
                        span: ident.span.clone(),
                    });
                }
            }
            _ => (),
        },
    }
    add_pars_scope(&arm_params, analyser); // parameter scope

    match exe {
        Expr {
            kind: ExprKind::Block(_),
            ..
        } => {
            check_for_val_compatibility(exe.clone(), &expect, genrs, &arm_params, analyser);
        }
        _ => {
            let span = exe.span.clone();
            analyser.errs.push(SemanticError {
                messages: vec![format!("You can only use value binding statement i.e ':expr' to return value when directly under another binded block expression"),
                    format!("In this case try binding with a block")],
                hints: vec![],
                span,
            });
        }
    }

    analyser.symbol_table.pop();
}

pub fn analyse_block(
    expect: &Expect,
    block: Vec<BlockLevel>,
    genrs: &Vec<Generic>,
    pars: &Vec<InterfaceParam>,
    analyser: &mut Analyser,
) -> Option<TypeSpec> {
    let mut ret: Option<TypeSpec> = None;
    // self.init_local_scope(block);
    analyser.symbol_table.push(HashMap::new()); // initializing this block
    let mut returned_val = false;
    let last_scope_index = analyser.symbol_table.len() - 1;
    for par in pars {
        match par {
            InterfaceParam {
                ident,
                state,
                ty,
                is_mut,
                span,
            } => {
                analyser.symbol_table[last_scope_index].insert(
                    ident.ident.clone(),
                    Element::Local(BlockLevel::Param {
                        state: state.clone(),
                        ty: ty.clone(),
                        name: ident.clone(),
                        mutable: is_mut.clone(),
                        info: StmtInfo {
                            span: span.clone(),
                            docs: vec![],
                        },
                    }),
                );
            }
        }
    }
    // let mut i = 1;
    for item in block.clone() {
        match item.clone() {
            BlockLevel::TakeStmt {
                take,
                from: _,
                lib: _,
                info: _,
            } => {
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

                name_resolver::add_to_local_scope(ident, item, analyser);
            }
            BlockLevel::VarDecl {
                interpret,
                state,
                ty,
                name,
                value,
                mutable: _,
                info: _,
            } => {
                if interpret {
                    match ty {
                        TypeSpec {
                            hint: TypeHint::Void,
                            span: _,
                        } => {
                            analyser.errs.push(SemanticError {
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

                validate_assignment(&ty, state, &value, &vec![], &vec![], analyser);
                name_resolver::add_to_local_scope(name, item, analyser);
            }
            BlockLevel::UnInitVarDecl {
                state: _,
                ty: _,
                name,
                mutable: _,
                info: _,
            } => {
                name_resolver::add_to_local_scope(name, item, analyser);
            }
            BlockLevel::VarInit {
                ident,
                expr,
                info: _,
            } => {
                // self.check_for_val_compatibility(
                //     expr.clone(),
                //     expect.clone(),
                //     &vec![],
                //     &vec![],
                //     symbol_table,
                // );

                let var = get_var(&ident, analyser);

                if var.is_some() {
                    let par_ty = type_checker::get_var_type(var.clone().unwrap());
                    if var_not_initialized(&ident, analyser) {
                        if par_ty.is_some() {
                            validate_assignment(
                                &par_ty.unwrap(),
                                var.unwrap().state,
                                &expr,
                                &vec![],
                                &vec![],
                                analyser,
                            );
                        }
                    }
                } else {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!(
                            "Unable to determine the type of variable '{:?}'",
                            ident
                        )],
                        hints: vec![],
                        span: ident.span.clone(),
                    });
                }
            }
            BlockLevel::RetVal { expr, info: _ } => {
                let span = expr.span.clone();
                //
                match &expect {
                    Expect::Required(_, _) => {
                        returned_val = true;
                        // Value was returned but as optional i.e jọ̀wọ́ expr, not binded i.e :expr
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Expected value type was returned but not binded to the statement"),
                                format!(
                                    "Note: You can only use value binding statement i.e ':expr' to return value when directly under another binded block expression"
                                )
                            ],
                            hints: vec![],
                            span: span.clone(),
                        });
                    }
                    Expect::Option(ty, refr) => {
                        if *refr == DataState::ImutRef || *refr == DataState::MutRef {
                            // if the block is to return a referencr,
                            // then the return value most be an identifier
                            // and it most have the same owner as any of the parameters that are references
                            match &expr {
                                Expr {
                                    kind: ExprKind::Ident { affix, ident },
                                    ..
                                } => {
                                    let var = analyser.get_var(ident);

                                    if var.is_some() {
                                        if affix.is_some() {
                                            match affix.clone().unwrap() {
                                                IdentAffix::Move(_) => {
                                                    analyser.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Reference variable {} cannot be moved out of scope",
                                                            ident.ident.clone()
                                                        )],
                                                        hints: vec![],
                                                        span: ident.span.clone(),
                                                    });
                                                }
                                                _ => (),
                                            }
                                        }
                                    } else {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Variable {} not found in scope",
                                                ident.ident.clone()
                                            )],
                                            hints: vec![],
                                            span: ident.span.clone(),
                                        });
                                    }
                                }
                                _ => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!("A reference should received an identifier instead of a raw value",)],
                                        hints: vec![],
                                        span: span.clone(),
                                    });
                                }
                            }
                        }

                        let expr_ty = type_checker::check_for_val_type(
                            expr.clone(),
                            expect,
                            genrs,
                            pars,
                            analyser,
                        );
                        if expr_ty.is_some() {
                            type_checker::is_type_compatible(
                                ty,
                                &expr_ty.clone().unwrap(),
                                genrs,
                                analyser,
                            );
                        } else {
                            match expr {
                                Expr {
                                    kind: ExprKind::Ident { affix: _, ident },
                                    ..
                                } => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Variable {} not found in scope",
                                            ident.ident.clone()
                                        )],
                                        hints: vec![],
                                        span: ident.span.clone(),
                                    });
                                }
                                Expr {
                                    kind: ExprKind::SelfVal,
                                    span,
                                } => match ty {
                                    TypeSpec {
                                        hint: TypeHint::SelfKw,
                                        span: _,
                                    } => {
                                        // for now
                                    }
                                    oths => {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Expected self but {}",
                                                oths.hint
                                            )],
                                            hints: vec![],
                                            span,
                                        });
                                    }
                                },
                                _ => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Unable to resolve expression type"
                                        )],
                                        hints: vec![],
                                        span,
                                    });
                                }
                            }
                        }

                        let _ = returned_val;
                        returned_val = true;
                        ret = expr_ty;
                    }
                    Expect::Void => {
                        returned_val = true;
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("A variable of void type can only receive a block of code with no value binded to it")],
                            hints: vec![],
                            span,
                        });
                    }
                }
            }
            BlockLevel::BindedVal { expr, info: _ } => {
                let span = expr.span.clone();
                match &expect {
                    Expect::Required(ty, refr) => {
                        if *refr == DataState::ImutRef || *refr == DataState::MutRef {
                            // if the block is to return a referencr,
                            // then the return value most be an identifier
                            // and it most have the same owner as any of the parameters that are references
                            // or from outer scope
                            match &expr {
                                Expr {
                                    kind: ExprKind::Ident { affix, ident },
                                    ..
                                } => {
                                    let var = analyser.get_var(ident);

                                    if var.is_some() {
                                        if var.unwrap().1 {
                                            analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "Variable {} does no live long enough",
                                                    ident.ident.clone()
                                                )],
                                                hints: vec![],
                                                span: ident.span.clone(),
                                            });
                                        } else {
                                            if affix.is_some() {
                                                match affix.clone().unwrap() {
                                                    IdentAffix::Move(_) => {
                                                        analyser.errs.push(SemanticError {
                                                            messages: vec![format!(
                                                                "Reference variable {} cannot be moved out of scope",
                                                                ident.ident.clone()
                                                            )],
                                                            hints: vec![],
                                                            span: ident.span.clone(),
                                                        });
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                    } else {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Variable {} not found in scope",
                                                ident.ident.clone()
                                            )],
                                            hints: vec![],
                                            span: ident.span.clone(),
                                        });
                                    }
                                }
                                Expr {
                                    kind: ExprKind::SelfVal,
                                    span,
                                } => match ty {
                                    TypeSpec {
                                        hint: TypeHint::SelfKw,
                                        span: _,
                                    } => {
                                        // for now
                                    }
                                    oths => {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Expected self but {}",
                                                oths.hint
                                            )],
                                            hints: vec![],
                                            span: span.clone(),
                                        });
                                    }
                                },
                                _ => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!("A reference should received an identifier instead of a raw value",)],
                                        hints: vec![],
                                        span: span.clone(),
                                    });
                                }
                            }
                        }

                        let expr_ty = type_checker::check_for_val_type(
                            expr.clone(),
                            expect,
                            genrs,
                            pars,
                            analyser,
                        );
                        if expr_ty.is_some() {
                            type_checker::is_type_compatible(
                                ty,
                                &expr_ty.clone().unwrap(),
                                genrs,
                                analyser,
                            );
                        } else {
                            match &expr {
                                Expr {
                                    kind: ExprKind::Ident { affix: _, ident },
                                    ..
                                } => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Variable {} not found in scope",
                                            ident.ident.clone()
                                        )],
                                        hints: vec![],
                                        span: ident.span.clone(),
                                    });
                                }
                                Expr {
                                    kind: ExprKind::SelfVal,
                                    ..
                                } => (), // for now
                                Expr {
                                    kind: ExprKind::ArgBuffer { name, .. },
                                    span: _,
                                } => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Function {} not found in scope",
                                            name.ident
                                        )],
                                        hints: vec![],
                                        span: name.span.clone(),
                                    });
                                }
                                _ => {
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!(
                                            "Unable to resolve expression type"
                                        )],
                                        hints: vec![],
                                        span,
                                    });
                                }
                            }
                        }

                        let _ = returned_val;
                        returned_val = true;
                        ret = expr_ty;
                    }
                    Expect::Option(_, _) => {
                        returned_val = true;
                        let span = expr.span.clone();
                        // Value was returned but as binded i.e :expr, not optional i.e jọ̀wọ́ expr
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("Expected value type was returned but was trying bind itself to the statement"),
                                format!("Note: You can only use value binding statement i.e ':expr' to return value when directly under binded block expression like this"),
                                format!("In this case try using throw keyword i.e with 'ju' instead of ':' in front of it")],
                            hints: vec![],
                            span,
                        });
                    } // Should return its own error
                    Expect::Void => {
                        returned_val = true;
                        analyser.errs.push(SemanticError {
                        messages: vec![format!("Void type can only receive a block of code with no returned value what so ever")],
                        hints: vec![],
                        span,
                    });
                    }
                }
            }
            BlockLevel::FuncCall { func, info: _ } => {
                match func.clone() {
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
                        if name.ident != String::from("tẹ") {
                            // for now
                            validate_fn_call_expr(
                                name,
                                expect,
                                genrs,
                                buffer_genrs,
                                args,
                                true,
                                analyser,
                            );
                        }
                    }
                    Expr {
                        kind: ExprKind::ObjAssess { obj, fields },
                        span: _,
                    } => {
                        validate_assess_expr(
                            obj.as_ref(),
                            fields,
                            expect,
                            genrs,
                            func,
                            pars,
                            analyser,
                        );
                    }
                    Expr {
                        kind:
                            ExprKind::ElemNamespaceAccess {
                                ident: _,
                                access: _,
                            },
                        span: _,
                    } => {
                        // static
                        todo!("not yet done");
                    }
                    _ => (), // unexpected
                }
            }
            BlockLevel::ReAssign {
                to_mut,
                assign_op: _,
                val,
                info: _,
            } => {
                match to_mut {
                    Expr {
                        kind:
                            ExprKind::MutableIdent {
                                var: var_ident,
                                fields,
                            },
                        ..
                    } => {
                        let var = get_var(&var_ident, analyser);

                        if var.is_some() {
                            if !var_not_initialized(&var_ident, analyser) {
                                if fields.len() == 0 {
                                    let par_ty = type_checker::get_var_type(var.clone().unwrap());
                                    if par_ty.is_some() {
                                        validate_assignment(
                                            &par_ty.unwrap(),
                                            var.unwrap().state,
                                            &val,
                                            &vec![],
                                            &vec![],
                                            analyser,
                                        );
                                    } else {
                                        analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "Unable to determine the type of variable '{}'",
                                                var_ident.ident.clone()
                                            )],
                                            hints: vec![],
                                            span: var_ident.span.clone(),
                                        });
                                    }
                                } else {
                                }
                            } else {
                                analyser.errs.push(SemanticError {
                                    messages: vec![
                                        format!(
                                            "Variable '{}' hasn't been initialized yet",
                                            var_ident.ident.clone()
                                        ),
                                        format!("Try initializing it, by removing the yí keyword"),
                                    ],
                                    hints: vec![],
                                    span: var_ident.span.clone(),
                                });
                            }
                        } else {
                            analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "Can't find variable '{}' in scope",
                                    var_ident.ident.clone()
                                )],
                                hints: vec![],
                                span: var_ident.span.clone(),
                            });
                        }
                    }
                    _ => (), // should be unreached
                }
            }
            BlockLevel::IfStmt {
                bool_expr,
                exe,
                else_ifs,
                else_exe,
                info: _,
            } => {
                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                // tí bool_expr...
                validate_assignment(
                    &TypeSpec {
                        hint: TypeHint::Bool,
                        span: Span {
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: analyser.modl.clone(),
                        },
                    },
                    DataState::Owner,
                    &bool_expr,
                    &vec![],
                    &vec![],
                    analyser,
                );

                match exe {
                    Expr {
                        kind: ExprKind::Block(_),
                        ..
                    } => {
                        check_for_val_compatibility(exe, &cust_expected, genrs, &vec![], analyser);
                    }
                    _ => {
                        let span = exe.span;
                        analyser.errs.push(SemanticError {
                            messages: vec![format!("You can only use value binding statement i.e ':expr' to return value when directly under another binded block expression"),
                                format!("In this case try binding with a block")],
                            hints: vec![],
                            span,
                        });
                    }
                }
                // àìjébè tí bool_expr: {...}
                for else_if in &else_ifs {
                    match else_if {
                        ElseIf {
                            bool_expr,
                            exe,
                            span: _,
                        } => {
                            validate_assignment(
                                &TypeSpec {
                                    hint: TypeHint::Bool,
                                    span: Span {
                                        st: Pos { column: 0, line: 0 },
                                        en: Pos { column: 0, line: 0 },
                                        file: analyser.modl.clone(),
                                    },
                                },
                                DataState::Owner,
                                bool_expr,
                                &vec![],
                                &vec![],
                                analyser,
                            );

                            match exe {
                                Expr {
                                    kind: ExprKind::Block(_),
                                    ..
                                } => {
                                    check_for_val_compatibility(
                                        exe.clone(),
                                        &cust_expected,
                                        genrs,
                                        &vec![],
                                        analyser,
                                    );
                                }
                                _ => {
                                    let span = exe.span.clone();
                                    analyser.errs.push(SemanticError {
                                        messages: vec![format!("You can only use value binding statement i.e ':expr' to return value when directly under another binded block expression"),
                                            format!("In this case try binding with a block")],
                                        hints: vec![],
                                        span,
                                    });
                                }
                            }
                        }
                    }
                }
                // àìjébè {...}
                if else_exe != None {
                    // no else arm
                    match else_exe.clone().unwrap() {
                        Expr {
                            kind: ExprKind::Block(_),
                            ..
                        } => {
                            check_for_val_compatibility(
                                else_exe.unwrap(),
                                &cust_expected,
                                genrs,
                                &vec![],
                                analyser,
                            );
                        }
                        _ => {
                            let span = else_exe.unwrap().span;
                            analyser.errs.push(SemanticError {
                                messages: vec![format!("You can only use value binding statement i.e ':expr' to return value when directly under another binded block expression"),
                                    format!("In this case try binding with a block")],
                                hints: vec![],
                                span,
                            });
                        }
                    }
                }
            }
            BlockLevel::ForStmt {
                assign,
                cond: _,
                incr: _,
                exe,
                info: _,
            } => {
                let made_par;
                match assign {
                    ForAssign {
                        ty,
                        name,
                        value: _,
                        span,
                    } => {
                        // self.validate_assignment(
                        //     &ty,
                        //     &value,
                        //     &vec![],
                        //     &vec![],
                        //     symbol_table,
                        // );

                        made_par = InterfaceParam {
                            ident: name,
                            state: DataState::Owner,
                            ty,
                            is_mut: true,
                            span,
                        }
                    }
                }
                // confirm the cond and incr operations as well
                // ... to_do()

                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                analyse_block(&cust_expected, exe, genrs, &vec![made_par], analyser);
            }
            BlockLevel::LoopStmt { exe, info: _ } => {
                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                analyse_block(&cust_expected, exe, genrs, &vec![], analyser);
            }
            BlockLevel::WhileStmt { expr, exe, info: _ } => {
                validate_assignment(
                    &TypeSpec {
                        hint: TypeHint::Bool,
                        span: Span {
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: analyser.modl.clone(),
                        },
                    },
                    DataState::Owner,
                    &expr,
                    genrs,
                    &vec![],
                    analyser,
                );

                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                analyse_block(&cust_expected, exe, genrs, &vec![], analyser);
            }
            BlockLevel::DoWhileStmt { exe, expr, info: _ } => {
                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                analyse_block(&cust_expected, exe, genrs, &vec![], analyser);

                validate_assignment(
                    &TypeSpec {
                        hint: TypeHint::Bool,
                        span: Span {
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: analyser.modl.clone(),
                        },
                    },
                    DataState::Owner,
                    &expr,
                    genrs,
                    &vec![],
                    analyser,
                );
            }
            BlockLevel::PeekingStmt {
                param,
                expr: _,
                exe,
                info: _,
            } => {
                let made_par;
                match param {
                    Parameter { ident, state, ty } => {
                        let span = ident.span.clone();
                        made_par = InterfaceParam {
                            ident,
                            state,
                            ty,
                            is_mut: true,
                            span,
                        };
                    }
                }
                // now check if expr is an iterator or as iteration triat
                //

                // match exe {
                //     Expr::Block(_, _) => {
                let cust_expected;
                match expect.clone() {
                    Expect::Required(ty, r) | Expect::Option(ty, r) => {
                        cust_expected = Expect::Option(ty, r);
                    }
                    Expect::Void => {
                        cust_expected = Expect::Void;
                    }
                }

                analyse_block(&cust_expected, exe, genrs, &vec![made_par], analyser);
            }
            BlockLevel::MatchStmt { var, arms, info: _ } => {
                let variable = get_var(&var, analyser);
                if variable.is_some() {
                    let var_type = type_checker::get_var_type(variable.clone().unwrap());

                    if var_type.is_some() {
                        let var_type = var_type.unwrap();

                        for arm in arms {
                            match arm {
                                Case { expr, exe } => {
                                    let cust_expected;
                                    match expect.clone() {
                                        Expect::Required(ty, r) | Expect::Option(ty, r) => {
                                            cust_expected = Expect::Option(ty, r);
                                        }
                                        Expect::Void => {
                                            cust_expected = Expect::Void;
                                        }
                                    }

                                    validate_match_arm(
                                        &variable.clone().unwrap().state,
                                        &var_type,
                                        genrs,
                                        &expr,
                                        &exe,
                                        cust_expected.clone(),
                                        analyser,
                                    );
                                }
                            }
                        }
                    } else {
                        analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "Unable to determine the type of variable {}",
                                var.ident.clone()
                            )],
                            hints: vec![],
                            span: var.span.clone(),
                        });
                    }
                } else {
                    analyser.errs.push(SemanticError {
                        messages: vec![format!(
                            "Variable {} not found in this scope",
                            var.ident.clone()
                        )],
                        hints: vec![],
                        span: var.span.clone(),
                    });
                }
            }
            _ => {
                analyser.errs.push(SemanticError {
                    messages: vec![format!("Found an unknown statement")],
                    hints: vec![],
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: analyser.modl.clone(),
                    },
                });
            }
        }
        // i += 1;
    }

    match expect {
        Expect::Required(ty, _) => {
            if !returned_val {
                analyser.errs.push(SemanticError {
                    messages: vec![format!("A return of type {} is expected", ty.hint)],
                    hints: vec![],
                    span: ty.span.clone(),
                });
            }
        }
        _ => (),
    }
    // closing block
    analyser.symbol_table.pop();

    ret
}
