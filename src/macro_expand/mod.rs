// One important thing to known about macro in Kui is that, the expands to block statement,
// which means you can use them only where you can use a block statement e.g on blocklevel and expressions

use std::collections::HashMap;
use std::vec;

use crate::compiler::{ErrorHint, Mod, NameSpace, SemanticError};
use crate::parser::{
    AssFunc, BlockLevel, BridgeDecl, Case, CustTyAss, CustomBlock, CustomLevel, Delimiter, ElseIf,
    Expr, ExprKind, ForAssign, ForCond, ForIncr, FragKind, Ident, MacroCase, MacroGroup, MacroPatt,
    Method, ModPath, Parser, PlatformArm, RepOperator, StmtInfo, TopLevel,
};
use crate::scanner::{FullTok, Pos, Span, Token};
use crate::analyser::Analyser;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MacroFree {
    // It most marge all modules into 1, and make them a single namespace
    pub name: String,
    pub mod_pub: Option<TopLevel>,
    pub main_fn: Option<TopLevel>,
    pub fns: Vec<TopLevel>,
    pub platform_n_s: Vec<TopLevel>,
    pub takes: Vec<TopLevel>,
    pub globals: Vec<TopLevel>,
    pub traits: Vec<TopLevel>,
    pub workers: Vec<TopLevel>,
    pub statics: Vec<TopLevel>,
    pub custom_tys: Vec<TopLevel>,
    pub type_converters: Vec<TopLevel>,
    pub lines: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AstMacroExpandsion<'a> {
    file: String, // the name of the file/module currently being analysed
    // hirs: &'a [Hir<'a>],
    has_macro: bool,
    analyser: &'a mut Analyser, //
}

impl<'a> AstMacroExpandsion<'a> {
    pub fn new(analyser: &'a mut Analyser) -> Self {
        // AstMacroExpandsion uses the unit and errs fields of Analyser
        Self {
            file: String::new(),
            // hirs: Vec::new(),
            has_macro: false,
            analyser,
        }
    }

    fn is_compatible(&mut self, kind: FragKind, tok: Token) -> bool {
        if kind == FragKind::Expr {
            match tok {
                Token::StringLiteral(_) => true,
                Token::Number(_) => true,
                _ => false,
            }
        } else if kind == FragKind::Ident {
            match tok {
                Token::Identifier(_) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn match_pattern(
        &mut self,
        patterns: &Vec<MacroPatt>,
        args: &Vec<FullTok>,
    ) -> Option<HashMap<String, FullTok>> {
        let mut bindings = HashMap::new();
        if args.len() == 0 && patterns.len() == 0 {
            // if they are both empty; we still want to return Some(), so that it will not be mistaken for an error
            return Some(bindings);
        }

        let mut ret = false;

        let mut expect_seperator = false;

        let mut i = 0; // record the index of current argument
        for patt in patterns.clone() {
            if expect_seperator {
                // if there is a seperator operator to be settled
                let for_sep = self.match_pattern(&vec![patt.clone()], &vec![args[i].clone()]);

                match for_sep {
                    Some(_) => {
                        // if the seperator is there
                        i += 1;
                        continue;
                    }
                    None => (), // if not there, then it was the last pattern, and the loop would have ended
                }
            }
            // patterns are the like parameters for macro_use arguments
            // so each pattern i.e patt can either be a literal - exactly what is written
            // it can also be fragment - a true parameter that tells which type of token is expected
            // a group that can contain or group other patterns into one - the delimeter need to match the one in the argument
            // or a repetition -
            let for_tok = args[i].clone(); // This is the Full-token/argument at the current position
            match patt {
                MacroPatt::Fragment { name, kind } => {
                    // check if the kind of the fragment matched the argument/full-token
                    let is_comp = self.is_compatible(kind, for_tok.kind.clone());

                    let string = match name {
                        Ident { ident, span: _ } => ident,
                    };
                    if is_comp {
                        bindings.insert(string, for_tok);
                        ret = true;
                    } else {
                        break;
                    }
                }
                MacroPatt::Group { delim, items } => {
                    if delim == Delimiter::Brace {
                        if for_tok.kind == Token::LBrace {
                            let mut toks = Vec::new();
                            i += 1; // advance
                            if i > patterns.len() - 1 {
                                // if out of bound
                                i -= 1; // To cause mis-match
                            }
                            while args[i].kind != Token::RBrace {
                                toks.push(args[i].clone());
                                i += 1;
                            }
                            // i += 1; // advance from RBrace

                            let group = self.match_pattern(&items, &toks);

                            match group {
                                Some(hash_map) => {
                                    for each in hash_map {
                                        bindings.insert(each.0, each.1);
                                    }
                                    ret = true;
                                }
                                None => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else if delim == Delimiter::Paren {
                        if for_tok.kind == Token::LParen {
                            let mut toks = Vec::new();
                            i += 1; // advance
                            if i > patterns.len() - 1 {
                                // if out of bound
                                i -= 1; // To cause mis-match
                            }
                            while args[i].kind != Token::RParen {
                                toks.push(args[i].clone());
                                i += 1; // advance
                                if i > patterns.len() - 1 {
                                    // if out of bound
                                    i -= 1; // To cause mis-match
                                }
                            }
                            // i += 1; // advance from RBrace

                            let group = self.match_pattern(&items, &toks);

                            match group {
                                Some(hash_map) => {
                                    for each in hash_map {
                                        bindings.insert(each.0, each.1);
                                    }
                                    ret = true;
                                }
                                None => {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else if delim == Delimiter::Bracket {
                        if for_tok.kind == Token::LBracket {
                            let mut toks = Vec::new();
                            i += 1; // advance
                            if i > patterns.len() - 1 {
                                // if out of bound
                                i -= 1; // To cause mis-match
                            }
                            while args[i].kind != Token::RBracket {
                                toks.push(args[i].clone());
                                i += 1; // advance
                                if i > patterns.len() - 1 {
                                    // if out of bound
                                    i -= 1; // To cause mis-match
                                }
                            }
                            // i += 1; // advance from RBrace

                            let group = self.match_pattern(&items, &toks);

                            match group {
                                Some(hash_map) => {
                                    for each in hash_map {
                                        bindings.insert(each.0, each.1);
                                    }
                                    ret = true;
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                    }
                }
                MacroPatt::Literal(l) => {
                    if l.kind != for_tok.kind {
                        break;
                    }
                    ret = true;
                }
                MacroPatt::Repetition {
                    pattern,
                    seperator,
                    operator,
                } => {
                    // a repetition pattern can contain only one other patterns
                    // except a nexted repetion
                    match *pattern.clone() {
                        MacroPatt::Repetition {
                            pattern: _,
                            seperator: _,
                            operator: _,
                        } => {
                            self.analyser.errs.push(SemanticError {
                                messages: vec![format!("")],
                                hints: vec![],
                                span: Span {
                                    st: Pos { column: 0, line: 0 },
                                    en: Pos { column: 0, line: 0 },
                                    file: self.file.clone()
                                },
                            });
                        }
                        _ => {
                            // *pattern = Group || Fragment || Literal
                            i += 1; // advance
                            if i > patterns.len() - 1 {
                                // if out of bound
                                i -= 1; // To cause mis-match
                            }
                            match operator {
                                RepOperator::OnceOrMore => {
                                    let patt = *pattern;
                                    let optn = self
                                        .match_pattern(&vec![patt.clone()], &vec![args[i].clone()]); // most match at least this one time

                                    match optn {
                                        Some(hash_map) => {
                                            for each in hash_map {
                                                bindings.insert(each.0, each.1);
                                            }
                                            // May be more than once
                                            loop {
                                                // thus the loop
                                                i += 1; // advance for each next argument
                                                let optn = self.match_pattern(
                                                    &vec![patt.clone()],
                                                    &vec![args[i].clone()],
                                                );

                                                match optn {
                                                    Some(hash_map) => {
                                                        for each in hash_map {
                                                            bindings.insert(each.0, each.1);
                                                        }

                                                        // check for the seperator
                                                        match seperator {
                                                            Some(_) => {
                                                                // since patterns run one after the other (for patt in patterns.clone()... at d top)
                                                                expect_seperator = true;
                                                                // it'll be handled at the start of each iteration
                                                            }
                                                            None => (),
                                                        }
                                                    }
                                                    None => {
                                                        // if the literal doesn't match, then it is obmitted thus the reverse
                                                        i -= 1;
                                                        break;
                                                    }
                                                }
                                            }
                                            ret = true;
                                            // check for the seperator
                                            match seperator {
                                                Some(_) => {
                                                    // since patterns run one after the other (for patt in patterns.clone()... at d top)
                                                    expect_seperator = true; // it'll be handled at the start of each iteration
                                                }
                                                None => (),
                                            }
                                        }
                                        None => {
                                            // if the literal doesn't match, then it is obmitted thus the reverse
                                            i -= 1;
                                        }
                                    }
                                }
                                RepOperator::ZeroOrMore => {}
                                RepOperator::Optional => {
                                    // Once or None
                                    let patt = *pattern;
                                    let optn =
                                        self.match_pattern(&vec![patt], &vec![args[i].clone()]);

                                    match optn {
                                        Some(hash_map) => {
                                            for each in hash_map {
                                                bindings.insert(each.0, each.1);
                                            }
                                            ret = true;
                                            // check for the seperator
                                            match seperator {
                                                Some(_) => {
                                                    // since patterns run one after the other (for patt in patterns.clone()... at d top)
                                                    expect_seperator = true; // it'll be handled at the start of each iteration
                                                }
                                                None => (),
                                            }
                                        }
                                        None => {
                                            // if the literal doesn't match, then it is obmitted thus the reverse
                                            i -= 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                MacroPatt::Foo => (),
            }
            i += 1; // increment the index number
        }
        if ret == true {
            Some(bindings)
        } else {
            None
        }
    }

    fn subtitude(
        &mut self,
        block: Vec<FullTok>,
        binding: HashMap<String, FullTok>,
        _from: (String, Option<Ident>),
    ) -> Vec<FullTok> {
        let mut ret = Vec::new();

        let mut i: usize = 0;

        loop {
            if block.len() != i + 1 {
                match &block[i] {
                    FullTok {
                        kind: Token::Percent,
                        span,
                    } => {
                        let next = &block[i + 1];

                        match next {
                            FullTok {
                                kind: Token::Identifier(ident),
                                span,
                            } => {
                                i += 1; // advance
                                let bind_to = binding.get(ident);

                                match bind_to.cloned() {
                                    Some(f) => {
                                        ret.push(f);
                                    }
                                    None => {
                                        self.analyser.errs.push(SemanticError {
                                            messages: vec![format!(
                                                "The macro variable {:?}, does not exist in this scope",
                                                ident,
                                            )],
                                            hints: vec![],
                                            span: span.clone()
                                        });
                                    }
                                }
                            }
                            _ => {
                                self.analyser.errs.push(SemanticError {
                                    messages: vec![format!(
                                        "Use %% to escape % as modulus operator"
                                    )],
                                    hints: vec![],
                                    span: span.clone(),
                                });
                            }
                        }
                    }
                    FullTok {
                        kind: Token::DPercent,
                        span,
                    } => {
                        ret.push(FullTok {
                            kind: Token::Percent,
                            span: span.clone(),
                        });
                    }
                    FullTok {
                        kind: Token::MacroCall(_),
                        span,
                    } => {
                        // let for_lib = if from.1.clone() == None {
                        //     String::from("")
                        // } else {
                        //     String::from("::") + from.1.clone().unwrap().ident.as_str()
                        // };
                        self.analyser.errs.push(SemanticError {
                            messages: vec![format!("Recursive macros are not allowed")],
                            hints: vec![],
                            span: span.clone(),
                        }); // Or expand it here
                    }
                    _ => {
                        ret.push(block[i].clone());
                    }
                }
            } else {
                break;
            }
            i += 1;
        }

        // for f_t in block {
        //     // f_t -> FullTok
        //     match f_t {
        //         FullTok {
        //             kind: Token::Percent,
        //             span: _,
        //         } => {
        // let next = block.next();

        // match next {
        //     Some(FullTok {
        //         kind: Token::Identifier(ident),
        //         span: _,
        //     }) => {
        //         let bind_to = binding.get(&ident);

        //         match bind_to.cloned() {
        //             Some(f) => {
        //                 ret.push(f);
        //             }
        //             None => {
        //                 self.errs.push(SemanticError {
        //                     file: self.file.clone(),
        //                     message: format!(
        //                         "The macro variable {:?}, does not exist in this scope",
        //                         ident,
        //                     ),
        //                     span: Span {
        //                         st: 0,
        //                         en: 0,
        //                         line: 0,
        //                     },
        //                 });
        //             }
        //         }
        //     }
        //     _ => (),
        // }
        //         }
        //         FullTok {
        //             kind: Token::MacroCall(_),
        //             span,
        //         } => {
        //             let for_lib = if from.1.clone() == None {
        //                 String::from("")
        //             } else {
        //                 String::from("::") + from.1.clone().unwrap().as_str()
        //             };
        //             self.errs.push(SemanticError {
        //                 file: from.0.clone() + for_lib.as_str() + String::from(" module").as_str(),
        //                 message: format!("Recursive macros are not allowed"),
        //                 span,
        //             }); // Or expand it here
        //         }
        //         _ => {
        //             ret.push(f_t);
        //         }
        //     }
        // }

        ret
    }

    // fn desolve_parse_result(&mut self, parse_result: ParseResult) -> (Vec<TopLevel>, Vec<Error>) {
    //     let mut ret = Vec::new();

    //     if parse_result.mod_pub.is_some() {
    //         let publ = parse_result.mod_pub.unwrap();

    //         let span = match publ {
    //             TopLevel::ModPub(span) => Span {
    //                 st: span.st,
    //                 en: span.en,
    //                 line: span.en,
    //             },
    //             _ => Span {
    //                 st: 0,
    //                 en: 0,
    //                 line: 0,
    //             },
    //         };
    //         self.errs.push(SemanticError {
    //             file: self.file.clone(),
    //             message: format!("Module publicity is not expected here"),
    //             span,
    //         });
    //     }

    //     for each in parse_result.mods {
    //         ret.push(each);
    //     }

    //     for each in parse_result.takes {
    //         ret.push(each);
    //     }

    //     for each in parse_result.globals {
    //         ret.push(each);
    //     }

    //     for each in parse_result.interprs {
    //         ret.push(each);
    //     }

    //     for each in parse_result.macro_def {
    //         ret.push(each);
    //     }

    //     for each in parse_result.macro_uses {
    //         ret.push(each);
    //     }

    //     for each in parse_result.custom_tys {
    //         ret.push(each);
    //     }

    //     for each in parse_result.traits {
    //         ret.push(each);
    //     }

    //     for each in parse_result.workers {
    //         ret.push(each);
    //     }

    //     for each in parse_result.bundles {
    //         ret.push(each);
    //     }

    //     for each in parse_result.funcs {
    //         ret.push(each);
    //     }

    //     for each in parse_result.platform_n_s {
    //         ret.push(each);
    //     }

    //     if parse_result.main_fn.is_some() {
    //         let m_fn = parse_result.main_fn.unwrap();

    //         let span = match m_fn {
    //             TopLevel::Entry {
    //                 expect: _,
    //                 agent_usage: _,
    //                 block: _,
    //                 span,
    //             } => Span {
    //                 st: span.st,
    //                 en: span.en,
    //                 line: span.en,
    //             },
    //             _ => Span {
    //                 st: 0,
    //                 en: 0,
    //                 line: 0,
    //             },
    //         };
    //         self.errs.push(SemanticError {
    //             file: self.file.clone(),
    //             message: format!("An Entry function 'gbòógì' is not expected here"),
    //             span,
    //         });
    //     }

    //     (ret, parse_result.errs)
    // }

    // fn solve_for_t_l_platform_block(
    //     &mut self,
    //     top_levels: Vec<TopLevel>,
    //     global: &mut HashMap<String, TopLevel>,
    // ) -> Vec<TopLevel> {
    //     let mut ret = Vec::new();

    //     for each in top_levels {
    //         match each.clone() {
    //             TopLevel::FuncDecl {
    //                 is_async,
    //                 expect,
    //                 ident,
    //                 generics,
    //                 is_public,
    //                 is_extern,
    //                 par,
    //                 worker_usage,
    //                 block,
    //                 span,
    //             } => {
    //                 if is_extern.is_some() {
    //                     // is_extern if block is Foo
    //                     ret.push(each);
    //                 } else {
    //                     let new_block = self.solve_for_expr(block.clone(), global);

    //                     ret.push(TopLevel::FuncDecl {
    //                         is_async,
    //                         expect,
    //                         ident,
    //                         generics,
    //                         is_public,
    //                         is_extern,
    //                         par,
    //                         worker_usage,
    //                         block: new_block,
    //                         span,
    //                     });
    //                 }
    //             }
    //             TopLevel::GlobalDecl {
    //                 interpret,
    //                 ty,
    //                 name,
    //                 value,
    //                 public,
    //                 span,
    //             } => {
    //                 let new_value = self.solve_for_expr(value.clone(), global);

    //                 ret.push(TopLevel::GlobalDecl {
    //                     interpret,
    //                     ty,
    //                     name,
    //                     value: new_value,
    //                     public,
    //                     span,
    //                 });
    //             } //     TopLevel::MacroUse {
    //             //         name,
    //             //         delimiter,
    //             //         tokens,
    //             //         expanded: _,
    //             //         span,
    //             //     } => {
    //             //         let expanded0 = self.expand_macro(each, global, mod_asts, platform.clone()); // new

    //             //         if let Ok(f_t_vec) = expanded0 {
    //             //             let mut new_parser = Parser::new(f_t_vec, false);
    //             //             let ast = new_parser.parse_program();

    //             //             let top_levels = self.desolve_parse_result(ast);

    //             //             let expanded;
    //             //             if top_levels.1.is_empty() {
    //             //                 expanded = Some(top_levels.0);
    //             //             } else {
    //             //                 for each in top_levels.1 {
    //             //                     self.errs.push(SemanticError {
    //             //                         file: self.file.clone(),
    //             //                         message: each.message,
    //             //                         span: each.span,
    //             //                     });
    //             //                 }
    //             //                 expanded = None;
    //             //             }

    //             //             ret.push(TopLevel::MacroUse {
    //             //                 name,
    //             //                 delimiter,
    //             //                 tokens,
    //             //                 expanded,
    //             //                 span,
    //             //             }); // already expanded for use
    //             //         }
    //             //     }
    //             //     // Macro definations and imports/takes have been added to global
    //             _ => (),
    //         }
    //     }

    //     ret
    // }

    fn solve_for_block(
        &mut self,
        block: Vec<BlockLevel>,
        global: &mut HashMap<String, TopLevel>,
    ) -> Vec<BlockLevel> {
        // Block Level blocks
        let mut ret = vec![];
        for each in block {
            match each {
                BlockLevel::VarDecl {
                    interpret,
                    state,
                    ty,
                    name,
                    value,
                    mutable,
                    info,
                } => {
                    let new_value = self.solve_for_expr(value.clone(), global);

                    ret.push(BlockLevel::VarDecl {
                        interpret,
                        state,
                        ty,
                        name,
                        value: new_value,
                        mutable,
                        info,
                    });
                }
                BlockLevel::BindedVal { expr, info } => {
                    let new_expr = self.solve_for_expr(expr.clone(), global);

                    ret.push(BlockLevel::BindedVal {
                        expr: new_expr,
                        info,
                    });
                }
                BlockLevel::ForStmt {
                    assign,
                    cond,
                    incr,
                    exe,
                    info,
                } => {
                    let new_assign;
                    match assign {
                        ForAssign {
                            ty,
                            name,
                            value,
                            span,
                        } => {
                            let new_value = self.solve_for_expr(*value.clone(), global);

                            new_assign = ForAssign {
                                ty,
                                name,
                                value: Box::new(new_value),
                                span,
                            };
                        }
                    }

                    let new_cond;
                    match cond {
                        ForCond { op, expr } => {
                            let new_expr = self.solve_for_expr(*expr.clone(), global);

                            new_cond = ForCond {
                                op,
                                expr: Box::new(new_expr),
                            };
                        }
                    }

                    let new_incr;
                    match incr {
                        ForIncr { incr_op, expr } => {
                            let new_expr = self.solve_for_expr(*expr.clone(), global);

                            new_incr = ForIncr {
                                incr_op,
                                expr: Box::new(new_expr),
                            };
                        }
                    }

                    let new_exe = self.solve_for_block(exe, global);

                    ret.push(BlockLevel::ForStmt {
                        assign: new_assign,
                        cond: new_cond,
                        incr: new_incr,
                        exe: new_exe,
                        info,
                    });
                }

                BlockLevel::PeekingStmt {
                    param,
                    expr,
                    exe,
                    info,
                } => {
                    let new_expr = self.solve_for_expr(expr.clone(), global);

                    let new_exe = self.solve_for_block(exe, global);

                    ret.push(BlockLevel::PeekingStmt {
                        param,
                        expr: new_expr,
                        exe: new_exe,
                        info,
                    });
                }
                BlockLevel::LoopStmt { exe, info } => {
                    let new_exe = self.solve_for_block(exe, global);

                    ret.push(BlockLevel::LoopStmt { exe: new_exe, info });
                }
                BlockLevel::WhileStmt { expr, exe, info } => {
                    let new_expr = self.solve_for_expr(expr.clone(), global);

                    let new_exe = self.solve_for_block(exe, global);

                    ret.push(BlockLevel::WhileStmt {
                        expr: new_expr,
                        exe: new_exe,
                        info,
                    });
                }
                BlockLevel::DoWhileStmt { exe, expr, info } => {
                    let new_expr = self.solve_for_expr(expr.clone(), global);

                    let new_exe = self.solve_for_block(exe, global);

                    ret.push(BlockLevel::DoWhileStmt {
                        exe: new_exe,
                        expr: new_expr,
                        info,
                    });
                }
                BlockLevel::IfStmt {
                    bool_expr,
                    exe,
                    else_ifs,
                    else_exe,
                    info,
                } => {
                    let new_expr = self.solve_for_expr(bool_expr.clone(), global);

                    let new_exe = self.solve_for_expr(exe.clone(), global);

                    let mut new_else_ifs = vec![];

                    for each in else_ifs {
                        match each {
                            ElseIf {
                                bool_expr,
                                exe,
                                span,
                            } => {
                                let new_exe = self.solve_for_expr(exe.clone(), global);

                                new_else_ifs.push(ElseIf {
                                    bool_expr,
                                    exe: new_exe,
                                    span,
                                });
                            }
                        }
                    }

                    // let new_block = self.solve_for_block(block.clone(), global)?;
                    let new_else_exe;
                    if else_exe.is_some() {
                        new_else_exe = Some(self.solve_for_expr(else_exe.unwrap().clone(), global));
                    } else {
                        new_else_exe = None;
                    }

                    ret.push(BlockLevel::IfStmt {
                        bool_expr: new_expr,
                        exe: new_exe,
                        else_ifs: new_else_ifs,
                        else_exe: new_else_exe,
                        info,
                    });
                }
                // BlockLevel::PlatformNS { arms, span } => {
                //     let mut new_arms = Vec::new();

                //     for arm in arms {
                //         match arm {
                //             PlatformArm {
                //                 hardware,
                //                 os,
                //                 block,
                //                 span,
                //             } => {
                //                 let new_block = self.solve_for_block(
                //                     block,
                //                     global,
                //                     mod_asts,
                //                     Some(Platform {
                //                         os: os.clone(),
                //                         hardware: hardware.clone(),
                //                     }),
                //                 )?;

                //                 new_arms.push(PlatformArm {
                //                     hardware,
                //                     os,
                //                     block: new_block,
                //                     span,
                //                 });
                //             }
                //         }
                //     }

                //     ret.push(BlockLevel::PlatformNS {
                //         arms: new_arms,
                //         span,
                //     });
                // }
                BlockLevel::MatchStmt { var, arms, info } => {
                    let mut new_arms = vec![];

                    for arm in arms {
                        match arm {
                            Case {
                                expr: arm_expr,
                                exe,
                            } => {
                                let new_arm_expr = self.solve_for_expr(arm_expr.clone(), global);

                                let new_exe = self.solve_for_expr(exe.clone(), global);

                                new_arms.push(Case {
                                    expr: new_arm_expr,
                                    exe: new_exe,
                                });
                            }
                        }
                    }

                    ret.push(BlockLevel::MatchStmt {
                        var,
                        arms: new_arms,
                        info,
                    });
                }
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
                    if is_extern.is_none() {
                        let new_block = self.solve_for_expr(block.clone(), global);

                        ret.push(BlockLevel::FuncDecl {
                            expect,
                            ident,
                            generics,
                            is_extern,
                            par,
                            worker_usage,
                            block: new_block,
                            info,
                        });
                    } else {
                        ret.push(BlockLevel::FuncDecl {
                            expect,
                            ident,
                            generics,
                            is_extern,
                            par,
                            worker_usage,
                            block,
                            info,
                        });
                    }
                }
                BlockLevel::FuncCall { func, info } => match func.clone() {
                    Expr {
                        kind:
                            ExprKind::ElemNamespaceAccess {
                                ident: static_ident,
                                access: _,
                            },
                        span: static_span,
                    } => {
                        let expr = self.solve_for_expr(func.clone(), global);

                        ret.push(BlockLevel::FuncCall {
                            func: Expr {
                                kind: ExprKind::ElemNamespaceAccess {
                                    ident: static_ident,
                                    access: Box::new(expr),
                                },
                                span: static_span,
                            },
                            info,
                        });
                    }
                    _ => {
                        ret.push(BlockLevel::FuncCall { func, info });
                    }
                },
                _ => {
                    ret.push(each);
                }
            }
        }
        ret
    }

    fn expand_macro(
        &mut self,
        macro_call: Expr,
        global: &mut HashMap<String, TopLevel>,
    ) -> Vec<FullTok> {
        let mut expanded = Vec::new();

        let mut matched = false;

        let mut span_of_use = Span {
            st: Pos { column: 0, line: 0 },
            en: Pos { column: 0, line: 0 },
            file: self.file.clone()
        };

        match macro_call {
            Expr {
                kind:
                    ExprKind::Macro {
                        name,
                        delimiter,
                        tokens,
                    },
                span,
            } => {
                let (key, name_span) = match name.clone() {
                    Ident { ident, span } => (ident, span),
                };
                let def = global.get(&key);

                span_of_use = span.clone();

                if def.is_none() {
                    self.analyser.errs.push(SemanticError {
                        messages: vec![format!(
                            "No macro with the identifier '{:?} exists in this scope'",
                            name
                        )],
                        hints: vec![],
                        span: name_span,
                    });
                } else {
                    let rule = def.unwrap();
                    match rule {
                        TopLevel::MacroDef {
                            ident: _,
                            is_public: _,
                            cases,
                            info: _,
                        } => {
                            for case in cases {
                                match case {
                                    MacroCase {
                                        pattern,
                                        block,
                                        info: _,
                                    } => match pattern {
                                        MacroGroup { delim, items } => {
                                            if delimiter == *delim {
                                                if let Some(binding) =
                                                    self.match_pattern(&items, &tokens)
                                                {
                                                    matched = true;
                                                    let mut sub = self.subtitude(
                                                        block.clone(),
                                                        binding,
                                                        (self.file.clone(), None),
                                                    );
                                                    sub.push(FullTok {
                                                        kind: Token::RBrace,
                                                        span: span.clone(),
                                                    }); // for stmt parsing...
                                                    expanded = sub;
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        TopLevel::TakeStmt {
                            take,
                            from,
                            lib,
                            info: _,
                        } => {
                            let take_ident = match take.0.kind.clone() {
                                Token::MacroCall(c) => Some(c),
                                _ => None,
                            };

                            match from {
                                ModPath::Ident(mod_ident) => {
                                    let modl = self.analyser.unit.get(&mod_ident.ident);

                                    match modl {
                                        Some(NameSpace::Singular {
                                            name: _,
                                            file:
                                                Mod {
                                                    mod_docs: _,
                                                    main_fn: _,
                                                    macro_defs,
                                                    fns: _,
                                                    globals: _,
                                                    platform_n_s: _,
                                                    takes: _,
                                                    traits: _,
                                                    workers: _,
                                                    statics: _,
                                                    custom_tys: _,
                                                    lines: _,
                                                    scope: _,
                                                },
                                        }) => {
                                            let mut gotten = None;
                                            for def in macro_defs {
                                                let macro_take_ident = take_ident.clone();
                                                match def.clone() {
                                                    TopLevel::MacroDef {
                                                        ident,
                                                        is_public,
                                                        cases,
                                                        info,
                                                    } => {
                                                        if macro_take_ident != None {
                                                            if ident.ident
                                                                == take_ident.clone().unwrap()
                                                            {
                                                                if is_public {
                                                                    gotten =
                                                                        Some(TopLevel::MacroDef {
                                                                            ident,
                                                                            is_public,
                                                                            cases,
                                                                            info,
                                                                        });
                                                                } else {
                                                                    self.analyser.errs.push(SemanticError {
                                                                                messages: vec![format!(
                                                                                    "Macro {:#?}, is not public",
                                                                                    take_ident.clone().unwrap()
                                                                                )],
                                                                                span: take.0.span.clone(),
                                                                                hints: vec![
                                                                                    ErrorHint {
                                                                                        hints: vec![format!(
                                                                                            "Macro defination {:#?} is private to this module",
                                                                                            ident.ident
                                                                                        )],
                                                                                        span: ident.span
                                                                                    }
                                                                                ]
                                                                            });
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }

                                            match gotten {
                                                Some(TopLevel::MacroDef {
                                                    ident: _,
                                                    is_public: _,
                                                    cases,
                                                    info: _,
                                                }) => {
                                                    for case in cases {
                                                        match case {
                                                            MacroCase {
                                                                pattern,
                                                                block,
                                                                info: _,
                                                            } => {
                                                                match pattern {
                                                                    MacroGroup { delim, items } => {
                                                                        if delimiter == delim {
                                                                            if let Some(binding) =
                                                                                self.match_pattern(
                                                                                    &items, &tokens,
                                                                                )
                                                                            {
                                                                                matched = true;
                                                                                let sub = self
                                                                                    .subtitude(
                                                                                    block,
                                                                                    binding.clone(),
                                                                                    (
                                                                                        mod_ident
                                                                                            .ident
                                                                                            .clone(
                                                                                            ),
                                                                                        lib.clone(),
                                                                                    ),
                                                                                );

                                                                                expanded = sub;
                                                                                // f_t -> FullTok
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    self.analyser.errs.push(SemanticError {
                                                        messages: vec![format!(
                                                            "Macro for {:#?}, does not exist in {:#?}",
                                                            take_ident.unwrap_or(String::from("Unknown")),
                                                            self.file.clone()
                                                        )],
                                                        span: take.0.span.clone(),
                                                        hints: vec![],
                                                    });
                                                }
                                            }
                                        }
                                        Some(NameSpace::Plural { name: _, files: _ }) => {}
                                        Some(NameSpace::Lib { .. }) => {}
                                        None => {
                                            self.analyser.errs.push(SemanticError {
                                                messages: vec![format!(
                                                    "The module {:#?}, does not exist in this project",
                                                    from
                                                )],
                                                span: take.0.span.clone(),
                                                hints: vec![],
                                            });
                                        }
                                    }
                                }
                                ModPath::Str(_string) => {
                                    todo!("todo")
                                }
                                ModPath::UnExpected => {
                                    todo!("Ohh");
                                }
                                ModPath::FromLibFolder(_, _) => {
                                    todo!("Import from lib mod")
                                }
                            }
                        }
                        _ => {
                            self.analyser.errs.push(SemanticError {
                                messages: vec![format!(
                                    "No macro with the identifier '{:?} exists in this scope'",
                                    name
                                )],
                                hints: vec![],
                                span, // of macro call #ident()
                            });
                        }
                    }
                }
            }
            _ => (),
        }
        if matched {
            expanded
        } else {
            self.analyser.errs.push(SemanticError {
                messages: vec![format!("No arm is matched")],
                hints: vec![],
                span: span_of_use,
            });
            vec![]
        }
    }

    fn solve_for_expr(&mut self, value: Expr, global: &mut HashMap<String, TopLevel>) -> Expr {
        // let macro_call;
        match value.clone() {
            Expr {
                kind: ExprKind::ObjAssess { obj, fields },
                span,
            } => {
                let new_obj = self.solve_for_expr(*obj.clone(), global);

                let mut new_fields = Vec::new();
                for field in fields {
                    let new_field = self.solve_for_expr(field.clone(), global);
                    new_fields.push(new_field);
                }

                Expr {
                    kind: ExprKind::ObjAssess {
                        obj: Box::new(new_obj),
                        fields: new_fields,
                    },
                    span,
                }
            }
            Expr {
                kind: ExprKind::Binary { lhs, op, rhs },
                span,
            } => {
                // let mut ret_none = true;
                let new_lhs = self.solve_for_expr(*lhs.clone(), global);

                let new_rhs = self.solve_for_expr(*rhs.clone(), global);

                Expr {
                    kind: ExprKind::Binary {
                        lhs: Box::new(new_lhs),
                        op,
                        rhs: Box::new(new_rhs),
                    },
                    span,
                }
            }
            Expr {
                kind: ExprKind::Unary { op, rhs },
                span,
            } => {
                let new_rhs = self.solve_for_expr(*rhs, global);

                Expr {
                    kind: ExprKind::Unary {
                        op,
                        rhs: Box::new(new_rhs),
                    },
                    span,
                }
            }
            Expr {
                kind:
                    ExprKind::ElemNamespaceAccess {
                        ident: static_ident,
                        access: call,
                    },
                span,
            } => match *call.clone() {
                Expr {
                    kind:
                        ExprKind::Macro {
                            name: _,
                            delimiter: _,
                            tokens: _,
                        },
                    span: _,
                } => {
                    let mut global_for_bundle = HashMap::new();

                    let mut macro_rules = vec![];

                    let ident = static_ident.clone().unwrap().ident + &"--";
                    let bundle = global.get(&ident);

                    if bundle.is_some() {
                        // building it's own macro_rules for it's custom global
                        match bundle.unwrap() {
                            TopLevel::StaticDef {
                                ident: _,
                                public: _,
                                block,
                                info: _,
                            } => {
                                for item in &*block.macro_def {
                                    match item {
                                        CustomLevel::MacroDef {
                                            ident,
                                            is_public,
                                            cases,
                                            info,
                                        } => {
                                            macro_rules.push(TopLevel::MacroDef {
                                                ident: ident.clone(),
                                                is_public: is_public.clone(),
                                                cases: cases.clone(),
                                                info: info.clone(),
                                            });
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            _ => (),
                        }
                    } else {
                        self.analyser.errs.push(SemanticError {
                            messages: vec![format!("Can't find Bundle '{:#?}' in scope", ident)],
                            hints: vec![],
                            span: span.clone(),
                        });
                    }

                    self.populate_global(vec![], macro_rules, vec![], &mut global_for_bundle);

                    // solving the macro expression
                    let new_call = self.solve_for_expr(*call.clone(), &mut global_for_bundle);

                    Expr {
                        kind: ExprKind::ElemNamespaceAccess {
                            ident: static_ident,
                            access: Box::new(new_call),
                        },
                        span,
                    }
                }
                _ => value,
            },
            Expr {
                kind: ExprKind::Block(block),
                span,
            } => {
                let new_block = self.solve_for_block(block, global);

                Expr {
                    kind: ExprKind::Block(new_block),
                    span,
                }
            }
            Expr {
                kind:
                    ExprKind::Macro {
                        name: _,
                        delimiter: _,
                        tokens: _,
                    },
                span,
            } => {
                let expanded = self.expand_macro(value, global);
                // let start_line = match &expanded[0] {
                //     FullTok { kind: _, span } => match span {
                //         Span { st: _, en: _, line } => line.clone(),
                //     },
                // };
                let mut new_parser = Parser::new(expanded.clone(), self.file.clone(), false);

                let stms = new_parser.parse_stmts();

                let block = stms.0;

                let expr = Expr {
                    kind: ExprKind::Block(block),
                    span,
                }; // every macro returns a block

                if new_parser.errs.is_empty() {
                    expr
                } else {
                    for err in &new_parser.errs {
                        self.analyser.errs.push(SemanticError {
                            messages: vec![err.message.clone()],
                            hints: vec![],
                            span: err.span.clone(),
                        });
                    }
                    expr
                }
            }
            oth => oth,
        }
    }

    fn add_macro_take_to_global(
        &mut self,
        take: (FullTok, Option<Ident>),
        from: ModPath,
        lib: Option<Ident>,
        info: StmtInfo,
        global: &mut HashMap<String, TopLevel>,
    ) {
        let tok = take.0.kind.clone();
        let _as = take.1;

        let mut key = String::new();

        match _as {
            // if some(), use it as the key
            Some(s) => {
                match tok.clone() {
                    // Token::Identifier(i) => {
                    //     // we must know if the identifier points to a bundle declaration
                    //     // because a bundle as the potencial of calling a macro
                    //     let mod_ast = self.analyser.unit.get(&from.ident);

                    //     if mod_ast.is_some() {
                    //         let bundles = &mod_ast.unwrap().statics;

                    //         for bundle in bundles {
                    //             match bundle {
                    //                 TopLevel::StaticDef {
                    //                     ident,
                    //                     public: _,
                    //                     block: _,
                    //                     span: _,
                    //                 } => {
                    //                     if ident.ident == i {
                    //                         // found it!
                    //                         key = s.ident.clone() + &"--";
                    //                     }
                    //                 }
                    //                 _ => (),
                    //             }
                    //         }
                    //     }
                    // }
                    Token::MacroCall(_) => {
                        key = s.ident;
                    }
                    _ => (), // it can only be macro call identifier for now
                }
            }
            None => {
                // if its none, then use the identifier
                match tok.clone() {
                    // Token::Identifier(i) => {
                    //     // we must know if the identifier points to a bundle declaration
                    //     // because a bundle as the potencial of calling a macro
                    //     let mod_ast = self.analyser.unit.get(&from.ident);

                    //     if mod_ast.is_some() {
                    //         let bundles = &mod_ast.unwrap().statics;

                    //         for bundle in bundles {
                    //             match bundle {
                    //                 TopLevel::StaticDef {
                    //                     ident,
                    //                     public: _,
                    //                     block: _,
                    //                     span: _,
                    //                 } => {
                    //                     if ident.ident == i {
                    //                         // found it!
                    //                         key = ident.ident.clone() + &"--"; // added to prevent name clashes; with macros
                    //                     }
                    //                 }
                    //                 _ => (),
                    //             }
                    //         }
                    //     }
                    // }
                    Token::MacroCall(ident) => {
                        key = ident;
                    }
                    _ => (), // it can only be macro call identifier for now
                }
            }
        }
        if key != String::new() {
            // only if key String is not empty
            if global.contains_key(&key) {
                // let alrealdy_there = global.get(&key);
                self.analyser.errs.push(SemanticError {
                    messages: vec![format!(
                        "Import with the same identifier '{:#?}', already exist'",
                        key
                    )],
                    hints: vec![],
                    span: take.0.span,
                });
            } else {
                global.insert(
                    key.clone(),
                    TopLevel::TakeStmt {
                        take: (take.0, None),
                        from: from.clone(),
                        lib: lib.clone(),
                        info: info.clone(),
                    },
                );
            }
        }
    }

    // for each toplevel statements

    // fn for_macro_use(
    //     &mut self,
    //     unit: &mut CompilationUnit,
    //     global: &mut HashMap<String, (TopLevel, Option<Platform>)>,
    //     mod_asts: &HashMap<String, ModAST>,
    // ) {
    //     for each in unit.macro_uses.clone() {
    //         // recursive micros are not allowed
    //         let expanded = self.expand_macro(each, global, mod_asts, None);
    //         // let mut unit = CompilationUnit::new(self.file.clone());

    //         if let Ok(f_t_vec) = expanded {
    //             let mut new_parser = Parser::new(f_t_vec, false);
    //             let ast = new_parser.parse_program();

    //             // For Module publicity declaration
    //             match ast.mod_pub {
    //                 Some(_) => {
    //                     self.errs.push(
    //                         SemanticError {
    //                             file: self.file.clone(),
    //                             message: format!("Declare module publicity at the on the file it self; not in a macro"),
    //                             span: Span { st: 0, en: 0, line: 0 }
    //                         }
    //                     );
    //                 }
    //                 None => (),
    //             }
    //             // For use statements - not expected in macros
    //             for each in ast.mods.clone() {
    //                 match each {
    //                     TopLevel::UseStmt {
    //                         module: _,
    //                         as_: _,
    //                         span,
    //                     } => {
    //                         self.errs.push(
    //                             SemanticError {
    //                                 file: self.file.clone(),
    //                                 message: format!("Module declaration should be on the file it self; not in a macro"),
    //                                 span: Span {
    //                                     st: span.st,
    //                                     en: span.en,
    //                                     line: span.en
    //                                 }
    //                             }
    //                         );
    //                     }
    //                     _ => (),
    //                 }
    //             }

    //             // For functions statements
    //             unit.assign_fns(ast.funcs);

    //             // For platform_n_s statements
    //             unit.assign_platform_n_s(ast.platform_n_s);

    //             // For import/take statements
    //             unit.assign_takes(ast.takes);

    //             // For import/take statements
    //             unit.assign_globals(ast.globals);

    //             // For macro_rules statements
    //             unit.assign_macro_rules(ast.macro_def);

    //             // For traits statements
    //             unit.assign_traits(ast.traits);

    //             // For agents statements
    //             unit.assign_workers(ast.workers);

    //             // For custom_tys statements
    //             unit.assign_cust_tys(ast.custom_tys);

    //             // For macro_uses statements
    //             unit.assign_macro_uses(ast.macro_uses.clone());
    //         }
    //     }
    //     unit.macro_rules.clear();
    // }

    fn for_global_var(
        &mut self,
        var: TopLevel,
        global: &mut HashMap<String, TopLevel>,
    ) -> TopLevel {
        match var {
            TopLevel::GlobalDecl {
                interpret,
                state,
                ty,
                name,
                value,
                public,
                info,
            } => {
                let new_value = self.solve_for_expr(value.clone(), global);

                TopLevel::GlobalDecl {
                    interpret,
                    state,
                    ty,
                    name,
                    value: new_value,
                    public,
                    info,
                }
            }
            _ => var,
        }
    }

    // fn for_platforms(
    //     &mut self,
    //     mut platform_n_s: Vec<TopLevel>,
    //     global: &mut HashMap<String, (TopLevel, Option<Platform>)>,
    //     mod_asts: &HashMap<String, ModAST>,
    // ) -> Vec<TopLevel> {
    //     let mut i = 0;
    //     for each in platform_n_s.clone() {
    //         match each {
    //             TopLevel::PlatformNS { arms, span } => {
    //                 let mut new_arms = Vec::new();

    //                 for arm in arms {
    //                     match arm {
    //                         PlatformTLArm {
    //                             hardware,
    //                             os,
    //                             block,
    //                             span,
    //                         } => {
    //                             let from_block = self.solve_for_t_l_platform_block(
    //                                 block,
    //                                 global,
    //                                 mod_asts,
    //                                 Some(Platform {
    //                                     os: os.clone(),
    //                                     hardware: hardware.clone(),
    //                                 }),
    //                             );

    //                             if let Ok(new_block) = from_block {
    //                                 new_arms.push(PlatformTLArm {
    //                                     hardware,
    //                                     os,
    //                                     block: new_block,
    //                                     span,
    //                                 });
    //                             }
    //                         }
    //                     }
    //                 }

    //                 platform_n_s[i] = TopLevel::PlatformNS {
    //                     arms: new_arms,
    //                     span,
    //                 };
    //             }
    //             _ => (),
    //         }
    //         i += 1;
    //     }

    //     platform_n_s
    // }

    fn for_function(&mut self, func: TopLevel, global: &mut HashMap<String, TopLevel>) -> TopLevel {
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
                    let new_block = self.solve_for_expr(block.clone(), global);

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
                        block: new_block,
                        info,
                    }
                } else {
                    func
                }
            }
            _ => func,
        }
    }

    fn for_static(
        &mut self,
        this_static: TopLevel,
        _global: &mut HashMap<String, TopLevel>,
    ) -> TopLevel {
        match &this_static {
            TopLevel::StaticDef {
                ident,
                public,
                block,
                info,
            } => {
                match block.clone() {
                    CustomBlock {
                        takes,
                        globals,
                        macro_def,
                        custom_tys,
                        traits,
                        workers,
                        funcs,
                        platform_n_s,
                        ..
                    } => {
                        // Statics are treated just like a module
                        let mut static_global = HashMap::new();

                        let mut top_takes = Vec::new();
                        let mut top_globals = Vec::new();
                        let mut top_macro_def = Vec::new();
                        let mut top_custom_tys = Vec::new();
                        let mut top_traits = Vec::new();
                        let mut top_workers = Vec::new();
                        let mut top_funcs = Vec::new();
                        let mut top_platform_n_s = Vec::new();

                        //  FROM STATIC_LEVEL TO TOP_LEVEL
                        {
                            for each in takes.clone() {
                                match each {
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

                            for each in globals {
                                match each {
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

                            for each in macro_def {
                                match each {
                                    CustomLevel::MacroDef {
                                        ident,
                                        is_public,
                                        cases,
                                        info,
                                    } => {
                                        top_macro_def.push(TopLevel::MacroDef {
                                            ident,
                                            is_public,
                                            cases,
                                            info,
                                        });
                                    }
                                    _ => (),
                                }
                            }

                            for each in custom_tys {
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
                                        top_custom_tys.push(TopLevel::EnumDecl {
                                            ident,
                                            generic_types,
                                            vals,
                                            traits,
                                            public,
                                            associateds,
                                            bridges,
                                            info,
                                        });
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
                                        top_custom_tys.push(TopLevel::StructDecl {
                                            ident,
                                            generic_types,
                                            vals,
                                            traits,
                                            public,
                                            associateds,
                                            bridges,
                                            info,
                                        });
                                    }
                                    _ => (),
                                }
                            }

                            for each in traits {
                                match each {
                                    CustomLevel::TraitDef {
                                        ident,
                                        generic_types,
                                        interprets,
                                        methods,
                                        is_public,
                                        info,
                                    } => {
                                        top_traits.push(TopLevel::TraitDef {
                                            ident,
                                            generic_types,
                                            interprets,
                                            methods,
                                            is_public,
                                            info,
                                        });
                                    }
                                    _ => (),
                                }
                            }

                            for each in workers {
                                match each {
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

                            for each in funcs {
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
                                        top_funcs.push(TopLevel::FuncDecl {
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
                                    _ => (),
                                }
                            }

                            for each in platform_n_s {
                                match each {
                                    CustomLevel::PlatformNS { arms, info } => {
                                        top_platform_n_s.push(TopLevel::PlatformNS {
                                            arms,
                                            info,
                                        });
                                    }
                                    _ => (),
                                }
                            }
                        }

                        self.populate_global(
                            top_takes.clone(),
                            top_macro_def.clone(),
                            vec![], // no recursive static
                            &mut static_global,
                        );

                        // expand all macros
                        // for each in self.unit.macro_uses.clone() {

                        let mut static_block = CustomBlock {
                            takes: takes.clone(),
                            globals: vec![],
                            macro_def: vec![],
                            custom_tys: vec![],
                            traits: vec![],
                            workers: vec![],
                            funcs: vec![],
                            platform_n_s: vec![],
                            static_defs: vec![]
                        };

                        // Next: Check for macro in each global variable
                        for glob in &top_globals {
                            let top = self.for_global_var(glob.clone(), &mut static_global);

                            match top {
                                TopLevel::GlobalDecl {
                                    interpret,
                                    state,
                                    ty,
                                    name,
                                    value,
                                    public,
                                    info,
                                } => {
                                    static_block.globals.push(CustomLevel::GlobalDecl {
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

                        // Functions
                        for func in &top_funcs {
                            let top = self.for_function(func.clone(), &mut static_global);

                            match top {
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
                                    static_block.globals.push(CustomLevel::FuncDecl {
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
                                    });
                                }
                                _ => (),
                            }
                        }

                        // Traits
                        for this_trait in &top_traits {
                            let top = self.for_trait(this_trait.clone(), &mut static_global);

                            match top {
                                TopLevel::TraitDef {
                                    ident,
                                    generic_types,
                                    interprets,
                                    methods,
                                    is_public,
                                    info,
                                } => {
                                    static_block.globals.push(CustomLevel::TraitDef {
                                        ident,
                                        generic_types,
                                        interprets,
                                        methods,
                                        is_public,
                                        info,
                                    });
                                }
                                _ => (),
                            }
                        }

                        // Types
                        for custm in &top_custom_tys {
                            let top = self.for_cust_type(custm.clone(), &mut static_global);

                            match top {
                                TopLevel::EnumDecl {
                                    ident,
                                    generic_types,
                                    vals,
                                    traits,
                                    public,
                                    associateds,
                                    bridges,
                                    info,
                                } => {
                                    static_block.custom_tys.push(CustomLevel::EnumDecl {
                                        ident,
                                        generic_types,
                                        vals,
                                        traits,
                                        public,
                                        associateds,
                                        bridges,
                                        info,
                                    });
                                }
                                TopLevel::StructDecl {
                                    ident,
                                    generic_types,
                                    vals,
                                    traits,
                                    public,
                                    associateds,
                                    bridges,
                                    info,
                                } => {
                                    static_block.custom_tys.push(CustomLevel::StructDecl {
                                        ident,
                                        generic_types,
                                        vals,
                                        traits,
                                        public,
                                        associateds,
                                        bridges,
                                        info,
                                    });
                                }
                                _ => (),
                            }
                        }

                        // Workers
                        for worker in &top_workers {
                            let top = self.for_worker(worker.clone(), &mut static_global);

                            match top {
                                TopLevel::WorkerDef {
                                    ident,
                                    func_name,
                                    public,
                                    block,
                                    info,
                                } => {
                                    static_block.globals.push(CustomLevel::WorkerDef {
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

                        TopLevel::StaticDef {
                            ident: ident.clone(),
                            public: public.clone(),
                            block: static_block,
                            info: info.clone(),
                        }
                    }
                }
            }
            _ => this_static,
        }
    }

    fn for_platform(
        &mut self,
        platform: TopLevel,
        _global: &mut HashMap<String, TopLevel>,
    ) -> TopLevel {
        match &platform {
            TopLevel::PlatformNS { arms, info } => {
                let mut new_arms = vec![];

                for arm in arms {
                    match arm {
                        PlatformArm {
                            target,
                            block,
                            span,
                        } => {
                            match block.clone() {
                                CustomBlock {
                                    takes,
                                    globals,
                                    macro_def,
                                    custom_tys,
                                    traits,
                                    workers,
                                    funcs,
                                    static_defs,
                                    platform_n_s: _,
                                } => {
                                    // Statics are treated just like a module
                                    let mut arm_global = HashMap::new();

                                    let mut top_takes = Vec::new();
                                    let mut top_globals = Vec::new();
                                    let mut top_macro_def = Vec::new();
                                    let mut top_custom_tys = Vec::new();
                                    let mut top_traits = Vec::new();
                                    let mut top_workers = Vec::new();
                                    let mut top_funcs = Vec::new();
                                    let mut top_static = Vec::new();

                                    //  FROM STATIC_LEVEL TO TOP_LEVEL
                                    {
                                        for each in takes.clone() {
                                            match each {
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

                                        for each in globals {
                                            match each {
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

                                        for each in macro_def {
                                            match each {
                                                CustomLevel::MacroDef {
                                                    ident,
                                                    is_public,
                                                    cases,
                                                    info,
                                                } => {
                                                    top_macro_def.push(TopLevel::MacroDef {
                                                        ident,
                                                        is_public,
                                                        cases,
                                                        info,
                                                    });
                                                }
                                                _ => (),
                                            }
                                        }

                                        for each in custom_tys {
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
                                                    top_custom_tys.push(TopLevel::EnumDecl {
                                                        ident,
                                                        generic_types,
                                                        vals,
                                                        traits,
                                                        public,
                                                        associateds,
                                                        bridges,
                                                        info,
                                                    });
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
                                                    top_custom_tys.push(TopLevel::StructDecl {
                                                        ident,
                                                        generic_types,
                                                        vals,
                                                        traits,
                                                        public,
                                                        associateds,
                                                        bridges,
                                                        info,
                                                    });
                                                }
                                                _ => (),
                                            }
                                        }

                                        for each in traits {
                                            match each {
                                                CustomLevel::TraitDef {
                                                    ident,
                                                    generic_types,
                                                    interprets,
                                                    methods,
                                                    is_public,
                                                    info,
                                                } => {
                                                    top_traits.push(TopLevel::TraitDef {
                                                        ident,
                                                        generic_types,
                                                        interprets,
                                                        methods,
                                                        is_public,
                                                        info,
                                                    });
                                                }
                                                _ => (),
                                            }
                                        }

                                        for each in workers {
                                            match each {
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

                                        for each in funcs {
                                            match each {
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
                                                    top_funcs.push(TopLevel::FuncDecl {
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

                                        for each in static_defs {
                                            match each {
                                                CustomLevel::StaticDef { ident, public, block, info } => {
                                                    top_static.push(TopLevel::StaticDef { ident, public, block, info });
                                                }
                                                _ => (),
                                            }
                                        }
                                    }

                                    self.populate_global(
                                        top_takes.clone(),
                                        top_macro_def.clone(),
                                        vec![], // no recursive static
                                        &mut arm_global,
                                    );

                                    // expand all macros
                                    // for each in self.unit.macro_uses.clone() {

                                    let mut arm_block = CustomBlock {
                                        takes: takes.clone(),
                                        globals: vec![],
                                        macro_def: vec![],
                                        custom_tys: vec![],
                                        traits: vec![],
                                        workers: vec![],
                                        funcs: vec![],
                                        platform_n_s: vec![],
                                        static_defs: vec![]
                                    };

                                    // Next: Check for macro in each global variable
                                    for glob in &top_globals {
                                        let top =
                                            self.for_global_var(glob.clone(), &mut arm_global);

                                        match top {
                                            TopLevel::GlobalDecl {
                                                interpret,
                                                state,
                                                ty,
                                                name,
                                                value,
                                                public,
                                                info,
                                            } => {
                                                arm_block.globals.push(CustomLevel::GlobalDecl {
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

                                    // Functions
                                    for func in &top_funcs {
                                        let top = self.for_function(func.clone(), &mut arm_global);

                                        match top {
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
                                                arm_block.funcs.push(CustomLevel::FuncDecl {
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
                                                });
                                            }
                                            _ => (),
                                        }
                                    }

                                    // Traits
                                    for this_trait in &top_traits {
                                        let top =
                                            self.for_trait(this_trait.clone(), &mut arm_global);

                                        match top {
                                            TopLevel::TraitDef {
                                                ident,
                                                generic_types,
                                                interprets,
                                                methods,
                                                is_public,
                                                info,
                                            } => {
                                                arm_block.traits.push(CustomLevel::TraitDef {
                                                    ident,
                                                    generic_types,
                                                    interprets,
                                                    methods,
                                                    is_public,
                                                    info,
                                                });
                                            }
                                            _ => (),
                                        }
                                    }

                                    // Types
                                    for custm in &top_custom_tys {
                                        let top =
                                            self.for_cust_type(custm.clone(), &mut arm_global);

                                        match top {
                                            TopLevel::EnumDecl {
                                                ident,
                                                generic_types,
                                                vals,
                                                traits,
                                                public,
                                                associateds,
                                                bridges,
                                                info,
                                            } => {
                                                arm_block.custom_tys.push(CustomLevel::EnumDecl {
                                                    ident,
                                                    generic_types,
                                                    vals,
                                                    traits,
                                                    public,
                                                    associateds,
                                                    bridges,
                                                    info,
                                                });
                                            }
                                            TopLevel::StructDecl {
                                                ident,
                                                generic_types,
                                                vals,
                                                traits,
                                                public,
                                                associateds,
                                                bridges,
                                                info,
                                            } => {
                                                arm_block.custom_tys.push(
                                                    CustomLevel::StructDecl {
                                                        ident,
                                                        generic_types,
                                                        vals,
                                                        traits,
                                                        public,
                                                        associateds,
                                                        bridges,
                                                        info,
                                                    },
                                                );
                                            }
                                            _ => (),
                                        }
                                    }

                                    for sta in &top_static {
                                        let top =
                                            self.for_static(sta.clone(), &mut arm_global);

                                        match top {
                                            TopLevel::StaticDef { ident, public, block, info } => {
                                                arm_block.static_defs.push(CustomLevel::StaticDef { ident, public, block, info });
                                            }
                                            _ => (),
                                        }
                                    }

                                    // Workers
                                    for worker in &top_workers {
                                        let top = self.for_worker(worker.clone(), &mut arm_global);

                                        match top {
                                            TopLevel::WorkerDef {
                                                ident,
                                                func_name,
                                                public,
                                                block,
                                                info,
                                            } => {
                                                arm_block.workers.push(CustomLevel::WorkerDef {
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

                                    new_arms.push(PlatformArm {
                                        target: target.clone(),
                                        block: arm_block,
                                        span: span.clone(),
                                    });
                                    // TopLevel::StaticDef {
                                    //     ident: ident.clone(),
                                    //     public: public.clone(),
                                    //     block: static_block,
                                    //     info: info.clone(),
                                    // }
                                }
                            }
                        }
                    }
                }

                TopLevel::PlatformNS {
                    arms: new_arms,
                    info: info.clone(),
                }
            }
            _ => platform,
        }
    }

    fn for_trait(
        &mut self,
        this_trait: TopLevel,
        global: &mut HashMap<String, TopLevel>,
    ) -> TopLevel {
        match this_trait.clone() {
            TopLevel::TraitDef {
                ident,
                generic_types,
                interprets,
                methods,
                is_public,
                info,
            } => {
                let new_interp = self.solve_for_block(interprets.clone(), global);

                let mut new_methods = vec![];
                for each in methods.clone() {
                    match each {
                        Method {
                            worker_usage,
                            meth_self,
                            // inher_val,
                            // is_mut,
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
                            if is_extern.is_none() {
                                if block.is_some() {
                                    let new_block =
                                        self.solve_for_expr(block.clone().unwrap(), global);

                                    new_methods.push(Method {
                                        worker_usage,
                                        meth_self,
                                        // inher_val,
                                        // is_mut,
                                        expect,
                                        ret_state,
                                        ident,
                                        generics,
                                        is_public,
                                        is_extern,
                                        par,
                                        block: Some(new_block),
                                        info,
                                    });
                                } else {
                                    new_methods.push(Method {
                                        worker_usage,
                                        meth_self,
                                        // inher_val,
                                        // is_mut,
                                        expect,
                                        ret_state,
                                        ident,
                                        generics,
                                        is_public,
                                        is_extern,
                                        par,
                                        block,
                                        info,
                                    });
                                }
                            }
                        }
                    }
                }
                TopLevel::TraitDef {
                    ident,
                    generic_types,
                    interprets: new_interp,
                    methods: new_methods,
                    is_public,
                    info,
                }
            }
            _ => this_trait,
        }
    }

    fn for_cust_type(
        &mut self,
        custom_ty: TopLevel,
        global: &mut HashMap<String, TopLevel>,
    ) -> TopLevel {
        match custom_ty.clone() {
            TopLevel::EnumDecl {
                ident,
                generic_types,
                vals,
                traits,
                public,
                associateds,
                bridges,
                info,
            } => {
                let mut new_methods = Vec::new();
                for each in associateds.clone() {
                    match each {
                        CustTyAss::Method(meth) => {
                            match meth {
                                Method {
                                    worker_usage,
                                    meth_self,
                                    // inher_val,
                                    // is_mut,
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
                                    if is_extern.is_none() {
                                        if block.is_some() {
                                            let new_block =
                                                self.solve_for_expr(block.clone().unwrap(), global);

                                            new_methods.push(CustTyAss::Method(Method {
                                                worker_usage,
                                                meth_self,
                                                // inher_val,
                                                // is_mut,
                                                expect,
                                                ret_state,
                                                ident,
                                                generics,
                                                is_public,
                                                is_extern,
                                                par,
                                                block: Some(new_block),
                                                info,
                                            }));
                                        } else {
                                            new_methods.push(CustTyAss::Method(Method {
                                                worker_usage,
                                                meth_self,
                                                // inher_val,
                                                // is_mut,
                                                expect,
                                                ret_state,
                                                ident,
                                                generics,
                                                is_public,
                                                is_extern,
                                                par,
                                                block,
                                                info,
                                            }));
                                        }
                                    } // else: leave it
                                }
                            }
                        }
                        CustTyAss::AssFunction(func) => match func {
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
                                if is_extern.is_none() {
                                    if block.is_some() {
                                        let new_block =
                                            self.solve_for_expr(block.clone().unwrap(), global);

                                        new_methods.push(CustTyAss::AssFunction(AssFunc {
                                            worker_usage,
                                            expect,
                                            ret_state,
                                            ident,
                                            generics,
                                            is_public,
                                            is_extern,
                                            par,
                                            block: Some(new_block),
                                            info,
                                        }));
                                    } else {
                                        new_methods.push(CustTyAss::AssFunction(AssFunc {
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
                                        }));
                                    }
                                }
                            }
                        },
                    }
                }

                let mut new_bridges = vec![];

                for bridge in bridges {
                    match bridge {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let new_block = self.solve_for_expr(block, global);

                            new_bridges.push(BridgeDecl {
                                ass_state,
                                ass_type,
                                ass_ident,
                                new_ty,
                                public,
                                block: new_block,
                                // methods: new_methods,
                                info,
                            });
                        }
                    }
                }

                TopLevel::EnumDecl {
                    ident,
                    generic_types,
                    vals,
                    traits,
                    public,
                    associateds: new_methods,
                    bridges: new_bridges,
                    info,
                }
            }
            TopLevel::StructDecl {
                ident,
                generic_types,
                vals,
                traits,
                public,
                associateds,
                bridges,
                info,
            } => {
                let mut new_methods = Vec::new();

                for each in associateds.clone() {
                    match each {
                        CustTyAss::Method(meth) => {
                            match meth {
                                Method {
                                    worker_usage,
                                    meth_self,
                                    // inher_val,
                                    // is_mut,
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
                                    if is_extern.is_none() {
                                        if block.is_some() {
                                            let new_block =
                                                self.solve_for_expr(block.clone().unwrap(), global);

                                            new_methods.push(CustTyAss::Method(Method {
                                                worker_usage,
                                                meth_self,
                                                // inher_val,
                                                // is_mut,
                                                expect,
                                                ret_state,
                                                ident,
                                                generics,
                                                is_public,
                                                is_extern,
                                                par,
                                                block: Some(new_block),
                                                info,
                                            }));
                                        } else {
                                            new_methods.push(CustTyAss::Method(Method {
                                                worker_usage,
                                                meth_self,
                                                // inher_val,
                                                // is_mut,
                                                expect,
                                                ret_state,
                                                ident,
                                                generics,
                                                is_public,
                                                is_extern,
                                                par,
                                                block,
                                                info,
                                            }));
                                        }
                                    } // else: leave it
                                }
                            }
                        }
                        CustTyAss::AssFunction(_func) => {
                            todo!("todo")
                        }
                    }
                }

                let mut new_bridges = vec![];

                for bridge in bridges {
                    match bridge {
                        BridgeDecl {
                            ass_state,
                            ass_type,
                            ass_ident,
                            new_ty,
                            public,
                            block,
                            info,
                        } => {
                            let new_block = self.solve_for_expr(block, global);

                            new_bridges.push(BridgeDecl {
                                ass_state,
                                ass_type,
                                ass_ident,
                                new_ty,
                                public,
                                block: new_block,
                                // methods: new_methods,
                                info,
                            });
                        }
                    }
                }

                TopLevel::StructDecl {
                    ident,
                    generic_types,
                    vals,
                    traits,
                    public,
                    associateds: new_methods,
                    bridges: new_bridges,
                    info,
                }
            }
            _ => custom_ty,
        }
    }

    fn for_worker(&mut self, worker: TopLevel, global: &mut HashMap<String, TopLevel>) -> TopLevel {
        match worker.clone() {
            TopLevel::WorkerDef {
                ident,
                func_name,
                public,
                block,
                info,
            } => {
                let new_block = self.solve_for_block(block.clone(), global);

                TopLevel::WorkerDef {
                    ident,
                    func_name,
                    public,
                    block: new_block,
                    info,
                }
            }
            _ => worker,
        }
    }

    fn populate_global(
        &mut self,
        takes: Vec<TopLevel>,
        macro_rules: Vec<TopLevel>,
        bundles: Vec<TopLevel>,
        global: &mut HashMap<String, TopLevel>,
    ) {
        for import in takes {
            match import {
                TopLevel::TakeStmt {
                    take,
                    from,
                    lib,
                    info,
                } => {
                    self.add_macro_take_to_global(take, from, lib, info, global);
                }
                _ => (),
            }
        }

        for bundle in bundles {
            match bundle {
                TopLevel::StaticDef {
                    ident,
                    public,
                    block,
                    info,
                } => {
                    if global.contains_key(&(ident.ident.clone() + &"--")) {
                        // bundles unlike direct macro definations have '--' after their idents
                        self.analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "Bundle with the same identifier '{:#?}', already exist'",
                                ident
                            )],
                            hints: vec![],
                            span: ident.span.clone(),
                        });
                    } else {
                        global.insert(
                            ident.ident.clone() + &"--",
                            TopLevel::StaticDef {
                                ident,
                                public,
                                block,
                                info,
                            },
                        );
                    }
                }
                _ => (),
            }
        }

        for m_rule in macro_rules {
            match m_rule {
                TopLevel::MacroDef {
                    ident,
                    is_public,
                    cases,
                    info,
                } => {
                    if global.contains_key(&ident.ident) {
                        self.analyser.errs.push(SemanticError {
                            messages: vec![format!(
                                "Macro with the same identifier '{:#?}', already exist'",
                                ident
                            )],
                            hints: vec![],
                            span: ident.span.clone(),
                        });
                    } else {
                        global.insert(
                            ident.ident.clone(),
                            TopLevel::MacroDef {
                                ident,
                                is_public,
                                cases,
                                info,
                            },
                        );
                    }
                }
                _ => (),
            }
        }

        // for each in platform_n_s {
        //     match each {
        //         TopLevel::PlatformNS { arms, span: _ } => {
        //             for arm in arms {
        //                 match arm {
        //                     PlatformTLArm {
        //                         hardware,
        //                         os,
        //                         block,
        //                         span: _,
        //                     } => {
        //                         for stm in block {
        //                             match stm {
        //                                 TopLevel::TakeStmt {
        //                                     take,
        //                                     from,
        //                                     lib,
        //                                     span,
        //                                 } => {
        //                                     self.add_macro_take_to_global(
        //                                         take,
        //                                         from,
        //                                         lib,
        //                                         span,
        //                                         global,
        //                                         Some(Platform {
        //                                             os: os.clone(),
        //                                             hardware: hardware.clone(),
        //                                         }),
        //                                     );
        //                                 }
        //                                 TopLevel::MacroDef {
        //                                     ident,
        //                                     is_public,
        //                                     cases,
        //                                     span,
        //                                 } => {
        //                                     if global.contains_key(&ident.ident) {
        //                                         self.errs.push(SemanticError {
        //                                             file: self.file.clone(),
        //                                             messages: vec![format!(
        //                                                 "Macro with the same identifier '{:#?}', already exist'",
        //                                                 ident
        //                                             )],
        //                                             hints: vec![],
        //                                             span: ident.span.clone(),
        //                                         });
        //                                     } else {
        //                                         global.insert(
        //                                             ident.ident.clone(),
        //                                             (
        //                                                 TopLevel::MacroDef {
        //                                                     ident,
        //                                                     is_public,
        //                                                     cases,
        //                                                     span,
        //                                                 },
        //                                                 Some(Platform {
        //                                                     os: os.clone(),
        //                                                     hardware: hardware.clone(),
        //                                                 }),
        //                                             ),
        //                                         );
        //                                     }
        //                                 }
        //                                 _ => (), // Pass
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         _ => (),
        //     }
        // }
    }

    fn go_through_mod(&mut self, this_mod: &Mod, global: &mut HashMap<String, TopLevel>) -> Mod {
        let mut ret_mod = Mod {
            mod_docs: None,
            main_fn: None,
            macro_defs: vec![],
            fns: vec![],
            platform_n_s: vec![],
            takes: this_mod.takes.clone(),
            globals: vec![],
            traits: vec![],
            workers: vec![],
            statics: vec![],
            custom_tys: vec![],
            lines: vec![],
            scope: 0,
        };
        // self.analyser.unit.get(each_mod.0).unwrap();
        self.populate_global(
            this_mod.takes.clone(),
            this_mod.macro_defs.clone(),
            this_mod.statics.clone(),
            global,
        );
        // expand all macros
        ret_mod.mod_docs = this_mod.mod_docs.clone();

        // Next: Check for macro in each global variable
        for glob in &this_mod.globals {
            ret_mod
                .globals
                .push(self.for_global_var(glob.clone(), global));
        }

        // Now for each statements with a block of BlockLevel statements
        // Next: Check for macro in main function
        if this_mod.main_fn.is_some() {
            ret_mod.main_fn = Some(self.for_function(this_mod.main_fn.clone().unwrap(), global));
        } else {
            ret_mod.main_fn = None;
        }

        // Functions
        for func in &this_mod.fns {
            ret_mod.fns.push(self.for_function(func.clone(), global));
        }

        // Statics
        for this_static in &this_mod.statics {
            ret_mod
                .statics
                .push(self.for_static(this_static.clone(), global));
        }

        for this_platform in &this_mod.platform_n_s {
            ret_mod
                .platform_n_s
                .push(self.for_platform(this_platform.clone(), global));
        }

        // Traits
        for this_trait in &this_mod.traits {
            ret_mod
                .traits
                .push(self.for_trait(this_trait.clone(), global));
        }

        // Custom Types
        for cust in &this_mod.custom_tys {
            ret_mod
                .custom_tys
                .push(self.for_cust_type(cust.clone(), global));
        }

        // Workers
        for worker in &this_mod.workers {
            ret_mod
                .workers
                .push(self.for_worker(worker.clone(), global));
        }

        ret_mod
    }

    pub fn solve_ast(&'a mut self) -> HashMap<String, NameSpace> {
        // firstly, i have to get imports and macro_rules into the symbol-table
        let mut global = HashMap::new(); // has/uses its own symbol_table
        let mut ret = HashMap::new();

        let unit = self.analyser.unit.clone();
        for each_mod in &unit {
            //
            let this_mod = each_mod.1;
            match this_mod {
                NameSpace::Singular { name, file } => {
                    let ret_mod = self.go_through_mod(file, &mut global);
                    ret.insert(
                        name.clone(),
                        NameSpace::Singular {
                            name: name.clone(),
                            file: ret_mod,
                        },
                    );
                }
                NameSpace::Plural { name: _, files: _ } => {}
                NameSpace::Lib { name, .. } => {
                    // return the same
                    ret.insert(name.clone(), this_mod.clone());
                }
            }
        }
        ret
    }
}
