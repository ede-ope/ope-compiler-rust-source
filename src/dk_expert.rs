use std::collections::HashMap;

use crate::{
    borrow_checker::{FuncDeclInfo, FuncExpr, ModCFG}, cfg_resourses::{
        ArgOwnership, BasicBlock, BlockId, CFG, Inst, InstKind, ProgramPoint, Terminator,
    }, parser::{
        DataState, Expr, ExprKind, Generic, Ident, IdentAffix, IntRange, InterfaceParam, MutStatus, TypeHint, TypeSpec,
    }, scanner::{Pos, Span},
};

#[allow(dead_code)]
#[repr(u8)]
pub enum KkFormatTag {
    CFG = 0,
    Docs = 1,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum IntrKindTag {
    Function = 0,
    Struct = 1,
    Enum = 2,
    Const = 3,
    Global = 4,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum TopLevelTag {
    Idents = 0,
    Func = 1,
    ExternFunc = 2,
    GlobalDecl = 3,
    CustomType = 4,
    Trait = 5,
    Worker = 6,
    Static = 7,
    Method = 8,
    StaticMeth = 9,
    Bridge = 10,
    Take = 11,
    PlatformNS = 12,
    MainFn = 13,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum TypeHintTag {
    I8 = 0,
    I16 = 1,
    I32 = 2,
    I64 = 3,
    I128 = 4,
    ISize = 5,
    U8 = 6,
    U16 = 7,
    U32 = 8,
    U64 = 9,
    U128 = 10,
    USize = 11,
    F32 = 12,
    F64 = 13,
    Number = 14,
    CharLiteral = 15,
    StrLiteral = 16,
    ArityLiteral = 17, // ArityLiteral -> true, false, None, Both
    Str = 18,
    Char = 19,
    Bool = 20,
    Trinary = 21,
    Quaternary = 22,
    End = 23,
    Custom = 24,
    FuncGenericRet = 25,
    EnumLiteral = 26,
    StructLiteral = 27,
    Void = 28,
    SelfKw = 29,
    RawPtr = 30,
    MVSTy = 34,
    MVMTy = 35,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum ExprKindTag {
    Number = 0,
    Float = 1,
    Ident = 2,
    Import = 3,
    Str = 4,
    Char = 5,
    TruthVal = 6,
    MLStr = 7,
    Address = 8,
    List = 9,
    StructVal = 10,
    Block = 11,
    Unary = 12,
    Binary = 13,
    ObjAssess = 14,
    MutableIdent = 15,
    ArgBuffer = 16,
    ElemNamespaceAccess = 17,
    FuncGenericRet = 18,
    Macro = 19,
    AsType = 20,
    // control flow
    If = 21,
    Match = 22,
    // For {
    //     assign: ForAssign,
    //     cond: ForCond,
    //     incr: ForIncr,
    //     exe: Box<Expr>,
    //     span: Span
    // },
    // Loop {
    //     exe: Box<Expr>,
    //     span: Span
    // },
    // While {
    //     expr: Box<Expr>,
    //     exe: Box<Expr>,
    //     span: Span
    // },
    // DoWhile {
    //     exe: Box<Expr>,
    //     expr: Box<Expr>,
    //     span: Span
    // },
    // Peeking {
    //     param: Parameter,
    //     expr: Box<Expr>,
    //     exe: Box<Expr>,
    //     span: Span
    // },
    Undefined = 23,
    SelfVal = 24,
    Default = 25, // Used in Switch Statements
    Foo = 26,
    Nexted = 27,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum StateTag {
    Owner = 0,
    Moved = 1,
    ImutRef = 2,
    MutRef = 3,

    Global = 4,
    GlobalImutRef = 5,
    GlobalMutRef = 6,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum TerminatorTag {
    Jump = 0,
    Branch = 1,
    Return = 2,
    Unreachable = 3,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum OptionTag {
    None = 0,
    Some = 1,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum IdentAffixTag {
    ReAss = 0,
    Move = 1,
    Shared = 2,
    MutBorrow = 3,
    Generics = 4,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum ArgOwnershipTag {
    Owned = 0,
    SharedRef = 1,
    MutRef = 2,
    Copy = 3,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum MutStatusTag {
    Const = 0,
    Mut = 1,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum InstKindTag {
    Assign = 0,
    Store = 1,
    ReStore = 2,
    BorrowShared = 3,
    BorrowMut = 4,
    Use = 5,
    Move = 6,
    Drop = 7,
    Call = 8,
}

pub struct KkWriter<'a> {
    pub buf: Vec<u8>,
    idents_buf: HashMap<String, (u32, Vec<u8>)>,
    //       key      id    chars
    id_count: u32,
    cfg: &'a ModCFG,
}

impl<'a> KkWriter<'a> {
    pub fn new(cfg: &'a ModCFG) -> Self {
        Self {
            buf: vec![],
            idents_buf: HashMap::new(),
            id_count: 0,
            cfg,
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_str(&mut self, s: &str) {
        // length-prefixed string: u32 len + bytes
        let bytes = s.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.buf.extend_from_slice(bytes);
    }

    fn _write_bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    // write Elems
    fn write_bool(&mut self, b: bool) {
        self.write_u8(b as u8);
    }

    fn write_typehint(&mut self, hint: &TypeHint) {
        match hint {
            TypeHint::I8(range) => {
                self.write_u8(TypeHintTag::I8 as u8);
                if range.is_some() {
                    self.write_u8(OptionTag::Some as u8);

                    match range.as_ref().unwrap() {
                        IntRange { st, en, span } => {
                            self.write_str(st);
                            self.write_str(en);
                            self.write_span(span);
                        }
                    }
                } else {
                    self.write_u8(OptionTag::None as u8);
                }
            }
            TypeHint::I32(range) => {
                self.write_u8(TypeHintTag::I32 as u8);
                if range.is_some() {
                    self.write_u8(OptionTag::Some as u8);

                    match range.as_ref().unwrap() {
                        IntRange { st, en, span } => {
                            self.write_str(st);
                            self.write_str(en);
                            self.write_span(span);
                        }
                    }
                } else {
                    self.write_u8(OptionTag::None as u8);
                }
            }
            TypeHint::U8(range) => {
                self.write_u8(TypeHintTag::U8 as u8);
                if range.is_some() {
                    self.write_u8(OptionTag::Some as u8);

                    match range.as_ref().unwrap() {
                        IntRange { st, en, span } => {
                            self.write_str(st);
                            self.write_str(en);
                            self.write_span(span);
                        }
                    }
                } else {
                    self.write_u8(OptionTag::None as u8);
                }
            }
            TypeHint::U32(range) => {
                self.write_u8(TypeHintTag::U32 as u8);
                if range.is_some() {
                    self.write_u8(OptionTag::Some as u8);

                    match range.as_ref().unwrap() {
                        IntRange { st, en, span } => {
                            self.write_str(st);
                            self.write_str(en);
                            self.write_span(span);
                        }
                    }
                } else {
                    self.write_u8(OptionTag::None as u8);
                }
            }
            TypeHint::Str => {
                self.write_u8(TypeHintTag::Str as u8);
            }
            TypeHint::Bool => {
                self.write_u8(TypeHintTag::Bool as u8);
            }
            TypeHint::Void => {
                self.write_u8(TypeHintTag::Void as u8);
            }
            TypeHint::RawPtr { tysp, mutb } => {
                self.write_u8(TypeHintTag::RawPtr as u8);
                self.write_typespec(&*tysp);
                self.write_bool(*mutb);
            }
            oth => todo!("Type hint {:#?} not yet binary encoded", oth),
        }
    }

    fn write_state(&mut self, state: &DataState) {
        match state {
            DataState::Owner => self.write_u8(StateTag::Owner as u8),
            DataState::ImutRef => self.write_u8(StateTag::ImutRef as u8),
            DataState::MutRef => self.write_u8(StateTag::MutRef as u8),
            DataState::Global => self.write_u8(StateTag::Global as u8),
            DataState::GlobalImutRef => self.write_u8(StateTag::GlobalImutRef as u8),
            DataState::GlobalMutRef => self.write_u8(StateTag::GlobalMutRef as u8),
            DataState::Moved(span) => {
                self.write_u8(StateTag::Moved as u8);
                self.write_span(span);
            }
        }
    }

    fn write_span(&mut self, span: &Span) {
        match span {
            Span {
                st:
                    Pos {
                        column: st_col,
                        line: st_lin,
                    },
                en: Pos { column, line },
                file,
            } => {
                // start column
                self.write_u32(*st_col as u32);
                // start line
                self.write_u32(*st_lin as u32);
                // end column
                self.write_u32(*column as u32);
                // end line
                self.write_u32(*line as u32);

                // and the the file location/module of span
                self.write_str(file); // with length
            }
        }
    }

    fn write_typespec(&mut self, spec: &TypeSpec) {
        match spec {
            TypeSpec { hint, span } => {
                // the hint; tag =
                self.write_typehint(hint);

                // span
                self.write_span(span);
            }
        }
    }

    fn write_exprkind(&mut self, kind: &ExprKind) {
        match kind {
            ExprKind::Ident { affix, ident } => {
                // kind tag
                self.write_u8(ExprKindTag::Ident as u8);
                match affix {
                    Option::Some(some) => {
                        self.write_u8(OptionTag::Some as u8);
                        match some {
                            IdentAffix::Move(s) => {
                                self.write_u8(IdentAffixTag::Move as u8);
                                self.write_span(s);
                            }
                            IdentAffix::Shared => {
                                self.write_u8(IdentAffixTag::Shared as u8);
                            }
                            IdentAffix::MutBorrow => {
                                self.write_u8(IdentAffixTag::MutBorrow as u8);
                            }
                            IdentAffix::Generics(tys) => {
                                self.write_u8(IdentAffixTag::Generics as u8);

                                self.write_u32(tys.len() as u32);

                                for ty in tys {
                                    self.write_typespec(ty);
                                }
                            }
                            IdentAffix::ReAss(s) => {
                                self.write_u8(IdentAffixTag::ReAss as u8);
                                self.write_span(s);
                            }
                        }
                    }
                    None => {
                        self.write_u8(OptionTag::None as u8);
                    }
                }

                self.write_ident(ident);
            }
            ExprKind::Address(status, expr) => {
                // kind tag
                self.write_u8(ExprKindTag::Address as u8);
                match status {
                    MutStatus::Const => {
                        self.write_u8(MutStatusTag::Const as u8);
                    }
                    MutStatus::Mut(span) => {
                        self.write_u8(MutStatusTag::Mut as u8);
                        self.write_span(span);
                    }
                }
                self.write_expr(expr.as_ref());
            }
            ExprKind::AsType { expr, ty } => {
                // kind tag
                self.write_u8(ExprKindTag::AsType as u8);

                self.write_expr(expr);
                self.write_typespec(ty);
            }
            oth => {
                todo!("Couldn't write binary for {} yet", oth)
            }
        }
    }

    fn write_expr(&mut self, expr: &Expr) {
        match expr {
            Expr { kind, span } => {
                self.write_exprkind(kind);
                self.write_span(span);
            }
        }
    }

    fn write_ident(&mut self, ident: &Ident) {
        match ident {
            Ident { ident, span } => {
                if let Option::Some((id, _)) = self.idents_buf.get(ident) {
                    // just write the ident id
                    self.write_u32(*id);
                } else {
                    // have to register the ident first
                    self.idents_buf
                        .insert(ident.clone(), (self.id_count, ident.as_bytes().to_vec()));

                    // now write id
                    self.write_u32(self.id_count);
                    // increase count
                    self.id_count += 1;
                }

                self.write_span(span);
            }
        }
    }

    fn write_basic_block(&mut self, block: &BasicBlock) {
        match block {
            BasicBlock {
                id,
                label,
                insts,
                terminator,
                ..
            } => {
                // can't use usize because it's different from machine to machine
                self.write_u64(id.0 as u64);

                // label
                self.write_str(label);

                // instructions
                self.write_u16(insts.len() as u16);
                for inst in insts {
                    match inst {
                        Inst { point, kind } => {
                            match point {
                                ProgramPoint { block, inst } => {
                                    //Id
                                    self.write_u64(block.0 as u64);

                                    // inst int
                                    self.write_u64(*inst as u64);
                                }
                            }

                            // instruction kind
                            match kind {
                                InstKind::Store {
                                    name: _,
                                    value: _,
                                    is_move: _,
                                    is_init: _,
                                } => {
                                    self.write_u8(InstKindTag::Store as u8);
                                    todo!("writing InstKind::Store")
                                }
                                InstKind::Assign {
                                    ty: _,
                                    name: _,
                                    value: _,
                                    is_move: _,
                                    // group: _
                                } => {
                                    todo!("writing InstKind::Assign")
                                }
                                InstKind::Call { func, args, result } => {
                                    self.write_u8(InstKindTag::Call as u8);
                                    // register function name
                                    self.write_ident(func);

                                    // now write the arguments
                                    self.write_u16(args.len() as u16); // or maybe u8 is enough
                                    for arg in args {
                                        self.write_expr(&arg.0);
                                        match arg.1 {
                                            ArgOwnership::Copy => {
                                                self.write_u8(ArgOwnershipTag::Copy as u8);
                                            }
                                            ArgOwnership::MutRef => {
                                                self.write_u8(ArgOwnershipTag::MutRef as u8);
                                            }
                                            ArgOwnership::Owned => {
                                                self.write_u8(ArgOwnershipTag::Owned as u8);
                                            }
                                            ArgOwnership::SharedRef => {
                                                self.write_u8(ArgOwnershipTag::SharedRef as u8);
                                            }
                                        }
                                    }

                                    // result field
                                    if result.is_none() {
                                        self.write_u8(OptionTag::None as u8);
                                    } else {
                                        self.write_u8(OptionTag::Some as u8);
                                        self.write_ident(result.as_ref().unwrap());
                                    }
                                }
                                _ => todo!("Binary of Instruction"),
                            }
                        }
                    }
                }

                match terminator {
                    Option::Some(tmtr) => {
                        self.write_u8(OptionTag::Some as u8);
                        match tmtr {
                            Terminator::Jump(id) => {
                                self.write_u8(TerminatorTag::Jump as u8);

                                self.write_u64(id.0 as u64);
                            }
                            Terminator::Branch {
                                cond,
                                true_bb,
                                false_bb,
                            } => {
                                self.write_u8(TerminatorTag::Branch as u8);

                                self.write_expr(cond);
                                self.write_u64(true_bb.0 as u64);
                                self.write_u64(false_bb.0 as u64)
                            }
                            Terminator::Return(expr) => {
                                self.write_u8(TerminatorTag::Return as u8);
                                if let Option::Some(expr) = expr {
                                    self.write_u8(OptionTag::Some as u8);
                                    self.write_expr(expr);
                                } else {
                                    self.write_u8(OptionTag::None as u8);
                                }
                            }
                            Terminator::Unreachable => {
                                self.write_u8(TerminatorTag::Unreachable as u8);
                            }
                        }
                    }
                    None => {
                        self.write_u8(OptionTag::None as u8);
                    }
                }
            }
        }
    }

    fn write_cfg(&mut self, cfg: &CFG) {
        match cfg {
            CFG {
                name,
                blocks,
                entry,
                ..
            } => {
                // exits, loans, and live_ranges are not added to binary since they are no longer needed
                self.write_str(name);

                // basic blocks
                self.write_u16(blocks.len() as u16);
                for block in blocks {
                    self.write_basic_block(block);
                }

                self.write_u64(entry.0 as u64);
            }
        }
    }

    pub fn gen_from_mod(&mut self) -> Vec<u8> {
        match &self.cfg {
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
                //functions
                // First tag it as functions
                self.write_u8(TopLevelTag::Func as u8);
                self.write_u64(funcs.len() as u64);
                for func in funcs {
                    // the length of identifier is u8, which should be enough
                    self.write_str(func.0);
                    // [0,   1      6,   112, 114, 105, 110, 116, 102]
                    // tag funlen  len            ident
                    match func.1 {
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
                            block,
                            stm_span,
                        } => {
                            // ident span
                            self.write_span(ident_span);

                            // is_async
                            self.write_bool(*is_async);

                            // expected type
                            self.write_typespec(&expect);

                            // tag = u8
                            self.write_state(ret_state);

                            match generics {
                                Option::Some(genrs) => {
                                    self.write_u8(OptionTag::Some as u8);

                                    // the length of the generics
                                    self.write_u16(genrs.len() as u16);

                                    for genr in genrs {
                                        // each generics
                                        match genr {
                                            Generic { name, traits } => {
                                                self.write_ident(name);

                                                self.write_u16(traits.len() as u16);

                                                for tra in traits {
                                                    self.write_ident(tra);
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {
                                    self.write_u8(OptionTag::None as u8);
                                }
                            }

                            // write the length of parameter slice
                            self.write_u32(par.len() as u32);

                            for param in par {
                                // each parameters
                                match param {
                                    InterfaceParam {
                                        ident,
                                        state,
                                        ty,
                                        is_mut,
                                        span,
                                    } => {
                                        self.write_ident(ident);
                                        self.write_state(state);
                                        self.write_typespec(ty);
                                        self.write_bool(*is_mut);
                                        self.write_span(span);
                                    }
                                }
                            }

                            if is_extern.is_some() {
                                self.write_u8(OptionTag::Some as u8);

                                match &is_extern.clone().unwrap() {
                                    (ident, optn_span) => {
                                        self.write_str(&ident.ident);
                                        // ident span
                                        self.write_span(&ident.span);
                                        // span
                                        if optn_span.is_some() {
                                            self.write_u8(OptionTag::Some as u8);

                                            self.write_span(&optn_span.clone().unwrap());
                                        } else {
                                            self.write_u8(OptionTag::None as u8);
                                        }
                                    }
                                }
                            } else {
                                self.write_u8(OptionTag::None as u8);
                            }

                            // is_public
                            self.write_bool(*is_public);

                            // is_visible
                            self.write_bool(*is_visible);

                            // worker usage
                            // length of vec = u16
                            self.write_u16(worker_usage.len() as u16);
                            for ident in worker_usage {
                                self.write_ident(ident);
                            }

                            // block
                            match block {
                                FuncExpr::Cfg(cfg) => {
                                    // write Option::None type
                                    self.write_u8(OptionTag::Some as u8);

                                    self.write_cfg(cfg);
                                }
                                FuncExpr::None => {
                                    self.write_u8(OptionTag::None as u8);
                                }
                            }

                            self.write_span(stm_span);
                        }
                    }
                }

                let mut ident_section = vec![];
                // add the format tag as cfg
                ident_section.extend_from_slice(&(KkFormatTag::CFG as u8).to_le_bytes());
                ident_section.extend_from_slice(&(TopLevelTag::Idents as u8).to_le_bytes());

                ident_section.extend_from_slice(&(self.idents_buf.len() as u32).to_le_bytes());
                for ident in &self.idents_buf {
                    // write id
                    ident_section.extend_from_slice(&ident.1 .0.to_le_bytes());
                    // length — u32 explicitly
                    let len = ident.1 .1.len() as u32;
                    ident_section.extend_from_slice(&len.to_le_bytes());
                    // and the ident it self
                    ident_section.extend_from_slice(&ident.1 .1);
                }
                // make it first to make it easy for the reader
                self.buf.splice(0..0, ident_section);

                self.buf.clone()
            }
        }
    }
}

pub struct KkReader<'a> {
    buf: &'a [u8],
    ident_buf: HashMap<u32, String>,
    //       key      id    chars
    cursor: usize,
}

impl<'a> KkReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            ident_buf: HashMap::new(),
            cursor: 0,
        }
    }

    fn read_u8(&mut self) -> u8 {
        if self.cursor >= self.buf.len() {
            panic!("unexpected end of .kk file")
        }
        let v = self.buf[self.cursor];
        self.cursor += 1; // advance
        v
    }

    fn read_u16(&mut self) -> u16 {
        let b = self.read_bytes(2);
        u16::from_le_bytes(b.try_into().unwrap())
    }

    fn read_u32(&mut self) -> u32 {
        let r = self.read_bytes(4);
        u32::from_le_bytes(r.try_into().unwrap())
    }

    fn read_u64(&mut self) -> u64 {
        let b = self.read_bytes(8);

        u64::from_le_bytes(b.try_into().unwrap())
    }

    fn read_str(&mut self) -> String {
        let len = self.read_u32() as usize;
        let bytes = self.read_bytes(len);
        let from_u8 = String::from_utf8(bytes.to_vec());

        if let Ok(ret) = from_u8 {
            ret
        } else {
            panic!("Unable to read string from binary")
        }
    }

    fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    fn read_bytes(&mut self, n: usize) -> &'a [u8] {
        let up_till = self.cursor + n;

        if up_till > self.buf.len() {
            panic!("unexpected end of .kk file while reading bytes");
        }
        let slice = &self.buf[self.cursor..up_till];

        self.cursor = up_till;
        slice
    }

    fn read_typehint(&mut self) -> TypeHint {
        let byte = self.read_u8();

        if byte == TypeHintTag::I8 as u8 {
            let mut range = None;
            if self.read_u8() == OptionTag::Some as u8 {
                range = Some(IntRange {
                    st: self.read_str(),
                    en: self.read_str(),
                    span: self.read_span(),
                })
            }

            TypeHint::I8(range)
        } else if byte == TypeHintTag::I32 as u8 {
            let mut range = None;
            if self.read_u8() == OptionTag::Some as u8 {
                range = Some(IntRange {
                    st: self.read_str(),
                    en: self.read_str(),
                    span: self.read_span(),
                })
            }

            TypeHint::I32(range)
        } else if byte == TypeHintTag::U8 as u8 {
            let mut range = None;
            if self.read_u8() == OptionTag::Some as u8 {
                range = Some(IntRange {
                    st: self.read_str(),
                    en: self.read_str(),
                    span: self.read_span(),
                })
            }

            TypeHint::U8(range)
        } else if byte == TypeHintTag::U32 as u8 {
            let mut range = None;
            if self.read_u8() == OptionTag::Some as u8 {
                range = Some(IntRange {
                    st: self.read_str(),
                    en: self.read_str(),
                    span: self.read_span(),
                })
            }

            TypeHint::U32(range)
        } else if byte == TypeHintTag::Str as u8 {
            TypeHint::Str
        } else if byte == TypeHintTag::Bool as u8 {
            TypeHint::Bool
        } else if byte == TypeHintTag::Void as u8 {
            TypeHint::Void
        } else if byte == TypeHintTag::RawPtr as u8 {
            let tysp = Box::new(self.read_typespec());
            let mutb = self.read_bool();

            TypeHint::RawPtr { tysp, mutb }
        } else {
            todo!("Type hint not yet binary encoded");
        }
    }

    fn read_state(&mut self) -> DataState {
        let byte = self.read_u8();

        if byte == StateTag::Owner as u8 {
            DataState::Owner
        } else if byte == StateTag::ImutRef as u8 {
            DataState::ImutRef
        } else if byte == StateTag::MutRef as u8 {
            DataState::MutRef
        } else if byte == StateTag::Global as u8 {
            DataState::Global
        } else if byte == StateTag::GlobalImutRef as u8 {
            DataState::GlobalImutRef
        } else if byte == StateTag::GlobalMutRef as u8 {
            DataState::GlobalMutRef
        } else if byte == StateTag::Moved as u8 {
            DataState::Moved(self.read_span())
        } else {
            panic!("Shouldn't reach this block")
        }
    }

    fn read_span(&mut self) -> Span {
        // start column
        let st_col = self.read_u32();
        // start line
        let st_lin = self.read_u32();
        // end column
        let en_col = self.read_u32();
        // end line
        let en_lin = self.read_u32();

        // and the the file location/module of span
        let file = self.read_str(); // with length

        Span {
            st: Pos {
                column: st_col as usize,
                line: st_lin as usize,
            },
            en: Pos {
                column: en_col as usize,
                line: en_lin as usize,
            },
            file,
        }
    }

    fn read_typespec(&mut self) -> TypeSpec {
        let hint = self.read_typehint();
        // span
        let span = self.read_span();

        TypeSpec { hint, span }
    }

    fn read_exprkind(&mut self) -> ExprKind {
        let kind_tag = self.read_u8();
        if kind_tag == ExprKindTag::Ident as u8 {
            let mut affix = None;

            if self.read_u8() == OptionTag::Some as u8 {
                let affix_tag = self.read_u8();
                if affix_tag == IdentAffixTag::Move as u8 {
                    let span = self.read_span();

                    affix = Some(IdentAffix::Move(span));
                } else if affix_tag == IdentAffixTag::Shared as u8 {
                    affix = Some(IdentAffix::Shared);
                } else if affix_tag == IdentAffixTag::MutBorrow as u8 {
                    affix = Some(IdentAffix::MutBorrow);
                } else if affix_tag == IdentAffixTag::Generics as u8 {
                    let num_of_tys = self.read_u32();
                    let mut tys = vec![];
                    for _ in 0..num_of_tys {
                        tys.push(self.read_typespec());
                    }

                    affix = Some(IdentAffix::Generics(tys));
                } else if affix_tag == IdentAffixTag::ReAss as u8 {
                    affix = Some(IdentAffix::ReAss(self.read_span()));
                } else {
                    panic!("Unexpected Expression kind in ir")
                }
            }

            let ident = self.read_ident();

            ExprKind::Ident { affix, ident }
        } else if kind_tag == ExprKindTag::Address as u8 {
            let mut status = MutStatus::Const;
            if self.read_u8() == MutStatusTag::Mut as u8 {
                status = MutStatus::Mut(self.read_span());
            }

            let expr = self.read_expr();
            ExprKind::Address(status, Box::new(expr))
        } else if kind_tag == ExprKindTag::AsType as u8 {
            let expr = Box::new(self.read_expr());
            let ty = Box::new(self.read_typespec());
            ExprKind::AsType { expr, ty } 
        } else {
            panic!("Couldn't read expr binary for yet")
        }
    }

    fn read_expr(&mut self) -> Expr {
        let kind = self.read_exprkind();
        let span = self.read_span();

        Expr { kind, span }
    }

    fn read_ident(&mut self) -> Ident {
        let id = self.read_u32();

        // get string from table
        let ident = self.ident_buf.get(&id);

        if ident.is_some() {
            let ident = ident.unwrap().clone();
            let span = self.read_span();

            Ident { ident, span }
        } else {
            panic!("Can't get ident of id {} from binary", id);
        }
    }

    fn read_basic_block(&mut self) -> BasicBlock {
        // doen't use usize because it's different from machine to machine
        let id = self.read_u64();

        // label
        let label = self.read_str();

        // instructions
        let mut insts = vec![];
        let num_of_insts = self.read_u16();
        for _ in 0..num_of_insts {
            // ProgromPoint
            let block = BlockId(self.read_u64() as usize);
            let inst = self.read_u64() as usize;

            // instruction kind
            let inst_tag = self.read_u8();
            let kind;
            if inst_tag == InstKindTag::Store as u8 {
                //
                // kind = InstKind::Store { name: (), value: (), is_move: (), is_init: () }
                todo!("reading InstKindTag::Store")
            } else if inst_tag == InstKindTag::Assign as u8 {
                //
                todo!("reading InstKindTag::Assign")
            } else if inst_tag == InstKindTag::Call as u8 {
                // read function name
                let func = self.read_ident();

                // now read the arguments
                let mut args = vec![];
                let arg_len = self.read_u16();
                for _ in 0..arg_len {
                    let expr = self.read_expr();

                    let own_tag = self.read_u8();
                    if own_tag == ArgOwnershipTag::Copy as u8 {
                        args.push((expr, ArgOwnership::Copy));
                    } else if own_tag == ArgOwnershipTag::MutRef as u8 {
                        args.push((expr, ArgOwnership::MutRef));
                    } else if own_tag == ArgOwnershipTag::Owned as u8 {
                        args.push((expr, ArgOwnership::Owned));
                    } else if own_tag == ArgOwnershipTag::SharedRef as u8 {
                        args.push((expr, ArgOwnership::SharedRef));
                    } else {
                        panic!("Unexpected ArgOwnership in binary")
                    }
                }

                let mut result = None;
                let res_optn = self.read_u8();
                if res_optn == OptionTag::Some as u8 {
                    result = Some(self.read_ident());
                }

                kind = InstKind::Call { func, args, result };
            } else {
                panic!("Unknown InstKind in binary")
            }

            insts.push(Inst { point: ProgramPoint { block, inst }, kind });
        }

        // terminator
        let mut terminator = None;
        let optn_tag = self.read_u8();
        if optn_tag == OptionTag::Some as u8 {
            let tmtr_tag = self.read_u8();
            if tmtr_tag == TerminatorTag::Jump as u8 {
                terminator = Option::Some(Terminator::Jump(BlockId(self.read_u64() as usize)));
            } else if tmtr_tag == TerminatorTag::Branch as u8 {
                let cond = self.read_expr();
                let true_bb = BlockId(self.read_u64() as usize);
                let false_bb = BlockId(self.read_u64() as usize);

                terminator = Some(Terminator::Branch {
                    cond,
                    true_bb,
                    false_bb,
                })
            } else if tmtr_tag == TerminatorTag::Return as u8 {
                let mut expr = None;
                if self.read_u8() == OptionTag::Some as u8 {
                    expr = Some(self.read_expr());
                }

                terminator = Some(Terminator::Return(expr));
            } else if tmtr_tag == TerminatorTag::Unreachable as u8 {
                terminator = Some(Terminator::Unreachable);
            }
        }

        BasicBlock {
            id: BlockId(id as usize),
            label,
            insts,
            terminator,
            preds: vec![],
            succs: vec![],
            state_in: HashMap::new(),
            state_out: HashMap::new(),
            loans_in: vec![],
            loans_out: vec![],
        }
    }

    fn read_cfg(&mut self) -> CFG {
        // exits, loans, and live_ranges are not added to binary since they are no longer needed
        let name = self.read_str();

        // basic blocks
        let mut blocks = vec![];
        let num_of_blocks = self.read_u16();
        for _ in 0..num_of_blocks {
            blocks.push(self.read_basic_block());
        }

        let entry = BlockId(self.read_u64() as usize);

        CFG {
            name,
            blocks,
            entry,
            exits: vec![],
            loans: vec![],
            live_ranges: HashMap::new(),
        }
    }

    pub fn gen_mod(&mut self) -> ModCFG {
        let mut result = ModCFG::new();
        if self.read_u8() != 0 {
            panic!("Expected a CFG binary but found something else");
        }
        while self.cursor < self.buf.len() {
            let byte = self.read_u8();
            // check section tag
            if byte == TopLevelTag::Idents as u8 {
                let num_of_idents = self.read_u32();
                for _ in 0..num_of_idents {
                    let id = self.read_u32();
                    let string = self.read_str();

                    self.ident_buf.insert(id, string);
                }
            } else if byte == TopLevelTag::Func as u8 {
                let num_of_func = self.read_u64();

                for _ in 0..num_of_func {
                    let name = self.read_str();
                    let ident_span = self.read_span();
                    // is_async
                    let is_async = self.read_bool();
                    // expect
                    let expect = self.read_typespec();
                    // state
                    let ret_state = self.read_state();
                    // generics
                    let mut generics = None;
                    if self.read_u8() == OptionTag::Some as u8 {
                        // the length of the generics
                        let num_of_genrs = self.read_u16();
                        let mut cart = vec![];
                        for _ in 0..num_of_genrs {
                            let name = self.read_ident();
                            let mut traits = vec![];

                            let num_of_traits = self.read_u16();
                            for _ in 0..num_of_traits {
                                // get each traits
                                traits.push(self.read_ident());
                            }

                            cart.push(Generic { name, traits });
                        }

                        generics = Option::Some(cart);
                    }

                    let mut par = vec![];
                    // write the length of parameter slice
                    let num_of_par = self.read_u32();
                    for _ in 0..num_of_par {
                        let ident = self.read_ident();
                        let state = self.read_state();
                        let ty = self.read_typespec();
                        let is_mut = self.read_bool();
                        let span = self.read_span();

                        par.push(InterfaceParam {
                            ident,
                            state,
                            ty,
                            is_mut,
                            span,
                        });
                    }

                    let mut is_extern = None;
                    if self.read_u8() == OptionTag::Some as u8 {
                        let ident_str = self.read_str();
                        let ident_span = self.read_span();

                        let ident = Ident {
                            ident: ident_str,
                            span: ident_span,
                        };
                        let mut span = None;

                        if self.read_u8() == OptionTag::Some as u8 {
                            span = Option::Some(self.read_span());
                        }

                        is_extern = Option::Some((ident, span));
                    }

                    // is_public
                    let is_public = self.read_bool();

                    // is_visible
                    let is_visible = self.read_bool();

                    // worker usage
                    let mut worker_usage = vec![];
                    // length of vec = u16
                    let num_of_wok = self.read_u16();
                    for _ in 0..num_of_wok {
                        worker_usage.push(self.read_ident());
                    }

                    // block
                    let mut block = FuncExpr::None;
                    if self.read_u8() == OptionTag::Some as u8 {
                        block = FuncExpr::Cfg(self.read_cfg());
                    }

                    let stm_span = self.read_span();

                    result.funcs.insert(
                        name,
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
                            block,
                            stm_span,
                        },
                    );
                }
            } else {
                eprintln!("library has a binary fault");
                break;
            }
        }

        result
    }
}
