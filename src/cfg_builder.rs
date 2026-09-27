use crate::cfg_resourses::{
    ArgOwnership, BlockId, BorrowKind, Inst, InstKind, Loan, LoanId, ProgramPoint, Terminator, CFG,
};

use crate::parser::{BlockLevel, DataState, Expr, ExprKind, Ident, IdentAffix, TypeHint, TypeSpec};
use crate::scanner::{Pos, Span};
use crate::{type_checker};

/// Builds a CFG from a function's AST
pub struct CFGBuilder {
    modl: String,
    pub cfg: CFG,
    pub current: BlockId, // currently active block
    temp_count: usize,    // for generating temp names
    inst_count: usize,
}

impl CFGBuilder {
    pub fn new(func_name: &str, modl: String) -> Self {
        let mut cfg = CFG::new(func_name.to_string());
        let entry = cfg.new_block("entry");
        cfg.entry = entry;
        Self {
            modl,
            cfg,
            current: entry,
            temp_count: 0,
            inst_count: 0,
        }
    }

    /// generate a unique temp variable name
    fn fresh(&mut self, prefix: &str) -> String {
        let name = format!("{}.{}", prefix, self.temp_count);
        self.temp_count += 1;
        let _ = self.temp_count;
        name
    }

    /// allocate a new block
    fn new_block(&mut self, label: &str) -> BlockId {
        self.cfg.new_block(label)
    }

    /// emit an instruction into the current block
    fn emit(&mut self, kind: InstKind) {
        if !self.cfg.block(self.current).is_terminated() {
            let point = ProgramPoint {
                block: self.current,
                inst: self.inst_count,
            };
            self.inst_count += 1;
            self.cfg.push_inst(self.current, Inst { point, kind });
        }
    }

    /// set terminator on current block
    pub fn terminate(&mut self, term: Terminator) {
        if !self.cfg.block(self.current).is_terminated() {
            let cur = self.current;
            self.cfg.terminate(cur, term);
        }
    }

    /// switch builder to a different block
    fn switch_to(&mut self, block: BlockId) {
        self.current = block;
        self.inst_count = 0;
    }

    // -- public entry point --

    // pub fn build_function(&self, func_name: &str) -> CFG {
    //     let builder = CFGBuilder::new(func_name);

    //     // println!("Expr: {:#?} -> {:#?}", body, builder.build_expr(ty, body));

    //     // if entry block has no terminator, add implicit ret void
    //     // if !builder.cfg.block(builder.current).is_terminated() {
    //     //     builder.terminate(Terminator::Return(None));
    //     // }

    //     builder.cfg
    // }

    // -- expression lowering --

    pub fn build_expr(&mut self, state: DataState, ty: TypeSpec, expr: &Expr) -> Option<Ident> {
        match expr {
            Expr {
                kind: ExprKind::Block(stmts),
                ..
            } => {
                self.build_stmts(state, ty, stmts);
            }
            Expr {
                kind:
                    ExprKind::If {
                        bool_expr,
                        exe,
                        else_ifs,
                        else_exe,
                    },
                ..
            } => {
                self.build_if(state, ty, bool_expr, exe, else_ifs, else_exe);
            }
            _ => {
                // expression used as a statement — emit as assign to temp
                let tmp = self.fresh("tmp");

                let ident = Ident {
                    ident: tmp,
                    span: expr.span.clone(),
                };

                self.emit(InstKind::Assign {
                    ty,
                    name: ident.clone(),
                    value: expr.clone(),
                    is_move: false,
                });

                return Some(ident);
            }
        }

        None
    }

    // -- statement lowering --

    fn lower_expr(&mut self, ty: &TypeSpec, expr: &Expr) -> Expr {
        match &expr.kind {
            ExprKind::AsType { expr: _as_expr, ty: _as_ty } => {
                if type_checker::is_num_type(&ty) {
                    expr.clone()
                } else {
                    todo!("call to conversion trait for {} to {}", expr.kind, ty.hint)
                }
            }
            ExprKind::Number(n) => {
                match &ty.hint {
                    TypeHint::U8(_) | TypeHint::U16(_) |
                    TypeHint::U32(_) | TypeHint::U64(_) |
                    TypeHint::U128(_) |

                    TypeHint::I8(_) | TypeHint::I16(_) |
                    TypeHint::I32(_) | TypeHint::I64(_) |
                    TypeHint::I128(_) => Expr {
                        kind: ExprKind::AsType { expr: Box::new(expr.clone()), ty: Box::new(ty.clone()) },
                        span: expr.span.clone()
                    },
                    _ => todo!("call to conversion function for {} to {}", n, ty.hint)
                }
            }
            _ => {
                todo!("This complex expression cannot yet be lowered to cfg: {}", expr.kind);
            }
        }
    }

    fn build_stmts(&mut self, state: DataState, ty: TypeSpec, stmts: &[BlockLevel]) {
        for stmt in stmts {
            // stop emitting if block is already terminated
            if self.cfg.block(self.current).is_terminated() {
                break;
            }
            self.build_stmt(state.clone(), ty.clone(), stmt);
        }
    }

    fn build_stmt(&mut self, state: DataState, ty: TypeSpec, stmt: &BlockLevel) {
        match stmt {
            BlockLevel::VarDecl {
                name,
                value,
                ty,
                state,
                ..
            } => {
                match state {
                    DataState::ImutRef => {
                        // &x — shared borrow
                        if let ExprKind::Ident { ident, .. } = &value.kind {
                            self.emit_borrow(
                                InstKind::BorrowShared {
                                    result: name.clone(),
                                    lender: ident.clone(),
                                    loan_id: LoanId(self.cfg.loans.len()),
                                },
                                ident.clone(), // lender
                                name.clone(),  // borrower
                                BorrowKind::Shared,
                            );
                            return; // skip the normal Assign emit below
                        }
                    }
                    DataState::MutRef => {
                        // &mut x — mutable borrow
                        if let ExprKind::Ident { ident, .. } = &value.kind {
                            self.emit_borrow(
                                InstKind::BorrowMut {
                                    result: name.clone(),
                                    lender: ident.clone(),
                                    loan_id: LoanId(self.cfg.loans.len()),
                                },
                                ident.clone(),
                                name.clone(),
                                BorrowKind::Mutable,
                            );
                            return;
                        }
                    }
                    DataState::Owner => {
                        // check if value is an ident being moved
                        if let ExprKind::Ident { affix, ident } = &value.kind {
                            if affix.is_some() {
                                let affix = affix.clone().unwrap();
                                match affix {
                                    IdentAffix::Move(_) => {
                                        self.emit(InstKind::Move {
                                            from: ident.clone(),
                                            to: name.clone(),
                                        });
                                        return;
                                    }
                                    _ => (),
                                }
                            } else {
                                self.emit(InstKind::Use {
                                    used: ident.clone(),
                                    by: name.clone(),
                                });
                            }
                        }
                    }
                    _ => {} // fall through to normal Assign
                }

                let new_value = self.lower_expr(ty, value);

                self.emit(InstKind::Assign {
                    ty: ty.clone(),
                    name: name.clone(),
                    value: new_value,
                    is_move: false,
                });
            }
            BlockLevel::UnInitVarDecl { name, ty, .. } => {
                // emit as assign with a sentinel — marks variable as uninitialized
                self.emit(InstKind::Assign {
                    ty: ty.clone(),
                    name: name.clone(),
                    value: Expr {
                        // sentinel empty expr
                        kind: ExprKind::Undefined,
                        span: name.span.clone(),
                    },
                    is_move: false,
                });
                // immediately mark as moved so any Use before VarInit is caught
                // check_use_after_move will flag "UseAfterMove" which you can
                // rename to UseOfUninitialized at the error reporting layer
            }
            BlockLevel::VarInit { ident, expr, .. } => {
                self.emit(InstKind::Store {
                    name: ident.clone(),
                    value: expr.clone(),
                    is_move: false,
                    is_init: true,
                });
            }
            BlockLevel::ReAssign { to_mut, val, .. } => {
                self.emit(InstKind::ReStore {
                    name: self.extract_name(to_mut),
                    to_mut: to_mut.clone(),
                    val: val.clone(),
                    is_move: false,
                    is_init: false,
                });
            }
            BlockLevel::RetVal { expr, .. } => {
                self.terminate(Terminator::Return(Some(expr.clone())));
            }
            BlockLevel::Return(_) => {
                self.terminate(Terminator::Return(None));
            }
            BlockLevel::BindedVal { expr, .. } => {
                self.terminate(Terminator::Return(Some(expr.clone())));
            }
            BlockLevel::FuncCall { func, .. } => {
                if let Expr {
                    kind: ExprKind::ArgBuffer { name, args, .. },
                    ..
                } = func
                {
                    let owned_args: Vec<(Expr, ArgOwnership)> = args
                        .iter()
                        .map(|arg| {
                            let ownership = match arg {
                                // &x — shared borrow
                                Expr {
                                    kind: ExprKind::Ident { affix, ident: _ },
                                    ..
                                } => match affix {
                                    Some(IdentAffix::MutBorrow) => ArgOwnership::MutRef,
                                    Some(IdentAffix::Shared) => ArgOwnership::SharedRef,
                                    _ => ArgOwnership::Owned,
                                },
                                _ => ArgOwnership::Owned,
                            };
                            (arg.clone(), ownership)
                        })
                        .collect();

                    self.emit(InstKind::Call {
                        func: name.clone(),
                        args: owned_args,
                        result: None,
                    });

                    // self.emit(InstKind::Call {
                    //     func:   name.ident.clone(),
                    //     args:   args.clone(),
                    //     result: None, // result discarded
                    // });
                }
            }
            BlockLevel::IfStmt {
                bool_expr,
                exe,
                else_ifs,
                else_exe,
                ..
            } => {
                self.build_if(state, ty, bool_expr, exe, else_ifs, else_exe);
            }
            BlockLevel::WhileStmt { expr, exe, .. } => {
                self.build_while(state, ty, expr, exe);
            }
            BlockLevel::ForStmt {
                assign: _,
                cond: _,
                incr: _,
                exe,
                ..
            } => {
                self.build_for(state, ty, exe); // simplified — extend as needed
            }
            BlockLevel::LoopStmt { exe, .. } => {
                self.build_loop(state, ty, exe);
            }
            BlockLevel::Block { stms, .. } => {
                self.build_stmts(state, ty, stms);
            }
            BlockLevel::Break(_) => {
                // placeholder — loop builder sets the actual break target
                self.terminate(Terminator::Unreachable);
            }
            BlockLevel::Continue(_) => {
                // placeholder — loop builder sets the actual continue target
                self.terminate(Terminator::Unreachable);
            }
            _ => {}
        }
    }

    // -- control flow --

    fn build_if(
        &mut self,
        state: DataState,
        ty: TypeSpec,
        cond: &Expr,
        then_: &Expr,
        else_ifs: &[crate::parser::ElseIf],
        else_: &Option<Expr>,
    ) {
        let then_bb = self.new_block("if.then");
        let merge_bb = self.new_block("if.merge");
        let else_bb = if else_.is_some() || !else_ifs.is_empty() {
            self.new_block("if.else")
        } else {
            merge_bb
        };

        // terminate current block with branch
        self.terminate(Terminator::Branch {
            cond: cond.clone(),
            true_bb: then_bb,
            false_bb: else_bb,
        });

        // then block
        self.switch_to(then_bb);
        self.build_expr(state.clone(), ty.clone(), then_);
        if !self.cfg.block(self.current).is_terminated() {
            self.terminate(Terminator::Jump(merge_bb));
        }

        // else-if chain
        if !else_ifs.is_empty() || else_.is_some() {
            self.switch_to(else_bb);

            for (i, elif) in else_ifs.iter().enumerate() {
                let elif_then = self.new_block("elif.then");
                let elif_else = if i + 1 < else_ifs.len() || else_.is_some() {
                    self.new_block("elif.else")
                } else {
                    merge_bb
                };

                self.terminate(Terminator::Branch {
                    cond: elif.bool_expr.clone(),
                    true_bb: elif_then,
                    false_bb: elif_else,
                });

                self.switch_to(elif_then);
                self.build_expr(state.clone(), ty.clone(), &elif.exe);
                if !self.cfg.block(self.current).is_terminated() {
                    self.terminate(Terminator::Jump(merge_bb));
                }

                self.switch_to(elif_else);
            }

            // final else
            if let Some(else_expr) = else_ {
                self.build_expr(state.clone(), ty, else_expr);
                if !self.cfg.block(self.current).is_terminated() {
                    self.terminate(Terminator::Jump(merge_bb));
                }
            }
        }

        self.switch_to(merge_bb);
    }

    fn build_while(&mut self, state: DataState, ty: TypeSpec, cond: &Expr, body: &[BlockLevel]) {
        let cond_bb = self.new_block("while.cond");
        let body_bb = self.new_block("while.body");
        let after_bb = self.new_block("while.after");

        // jump into condition check
        self.terminate(Terminator::Jump(cond_bb));

        // condition block
        self.switch_to(cond_bb);
        self.terminate(Terminator::Branch {
            cond: cond.clone(),
            true_bb: body_bb,
            false_bb: after_bb,
        });

        // body block — loops back to condition
        self.switch_to(body_bb);
        self.build_stmts(state, ty, body);
        if !self.cfg.block(self.current).is_terminated() {
            self.terminate(Terminator::Jump(cond_bb));
        }

        self.switch_to(after_bb);
    }

    fn build_loop(&mut self, state: DataState, ty: TypeSpec, body: &[BlockLevel]) {
        let body_bb = self.new_block("loop.body");
        let after_bb = self.new_block("loop.after");

        self.terminate(Terminator::Jump(body_bb));

        self.switch_to(body_bb);
        self.build_stmts(state, ty, body);
        if !self.cfg.block(self.current).is_terminated() {
            // infinite loop — jumps back to itself
            self.terminate(Terminator::Jump(body_bb));
        }

        self.switch_to(after_bb);
    }

    fn build_for(&mut self, state: DataState, ty: TypeSpec, body: &[BlockLevel]) {
        // simplified — treat like a while loop for now
        // extend with proper ForAssign/ForCond/ForIncr later
        let body_bb = self.new_block("for.body");
        let after_bb = self.new_block("for.after");

        self.terminate(Terminator::Jump(body_bb));
        self.switch_to(body_bb);
        self.build_stmts(state, ty, body);
        if !self.cfg.block(self.current).is_terminated() {
            self.terminate(Terminator::Jump(body_bb));
        }

        self.switch_to(after_bb);
    }

    // -- helpers --

    fn extract_name(&self, expr: &Expr) -> Ident {
        match expr {
            Expr {
                kind: ExprKind::MutableIdent { var, .. },
                ..
            } => var.clone(),
            Expr {
                kind: ExprKind::Ident { ident, .. },
                ..
            } => ident.clone(),
            _ => Ident {
                ident: "unknown".to_string(),
                span: Span {
                    st: Pos { column: 0, line: 0 },
                    en: Pos { column: 0, line: 0 },
                    file: self.modl.clone(),
                },
            },
        }
    }

    // in cfg_builder.rs — when emitting BorrowShared/BorrowMut
    // you need to also push a Loan into cfg.loans:

    fn emit_borrow(
        &mut self,
        kind: InstKind,
        lender: Ident,
        borrower: Ident,
        borrow_kind: BorrowKind,
    ) {
        let loan_id = LoanId(self.cfg.loans.len());
        let created = ProgramPoint {
            block: self.current,
            inst: self.inst_count,
        };

        // expires = end of current block for now
        // will be refined when liveness is computed
        let expires = created; // placeholder — updated after liveness

        self.cfg.loans.push(Loan {
            id: loan_id,
            lender: lender.clone(),
            borrower: borrower.clone(),
            kind: borrow_kind,
            created,
            expires, // refined after compute_liveness
        });

        self.emit(kind);
    }
}
