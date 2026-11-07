// parser.rs
// Recursive-descent parser (hand-written).

// use std::path;

use core::fmt;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

// use std::{vec};
use crate::{
    compiler::FileKind,
    scanner::{FullTok, Pos, Span, Token, TruthVal},
};

/// Top-level AST nodes (minimal structures for the main EBNF constructs).
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum TopLevel {
    DocLines {
        lines: Vec<String>,
        span: Span,
    },
    UseStmt {
        path: Ident,
        is_sec_entry: bool,
        alias: Ident,
        is_public: bool,
        info: StmtInfo,
    },
    UseLibStmt {
        include: Ident,
        ackn_panic: bool,
        info: StmtInfo,
    },
    MultiTakeStmt {
        take: Vec<(Ident, Option<Ident>)>, // "as" as an option
        from: Vec<Ident>,
        info: StmtInfo,
    },
    TakeStmt {
        take: (Ident, Option<Ident>), // "as" as an option
        from: Vec<Ident>,
        info: StmtInfo,
    },
    GlobalDecl {
        // Always VarState::Alive
        interpret: bool,
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        value: Expr,
        public: bool,
        info: StmtInfo,
    },
    MultiGlobalDecl(Vec<Box<TopLevel>>),
    EnumDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<EnumVariants>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        info: StmtInfo,
    },
    StructDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<StructParam>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        info: StmtInfo,
    },
    GenericType {
        name: Ident,
        traits: Vec<(Ident, Vec<Ident>)>,
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
        block: Expr,
        info: StmtInfo,
    },
    OsTarget {
        arms: Vec<OsArm>,
        info: StmtInfo,
    },
    ArchTarget {
        arms: Vec<Arch>,
        info: StmtInfo,
    },
    WorkerDef {
        ident: Ident,
        func_name: Ident,
        public: bool,
        block: Vec<BlockLevel>,
        info: StmtInfo,
    },
    StaticDef {
        ident: Ident,
        public: bool,
        block: CustomBlock,
        info: StmtInfo,
    },
    TraitDef {
        ident: Ident,
        generic_types: Option<Ident>,
        // interprets: Vec<BlockLevel>,
        methods: Vec<Method>,
        is_public: bool,
        info: StmtInfo,
    },
    // WorkerUsage {
    //     workers: Vec<Ident>,
    //     info: StmtInfo,
    // },
    // MacroDef {
    //     ident: Ident,
    //     is_public: bool,
    //     cases: Vec<MacroCase>,
    //     info: StmtInfo,
    // },
    Entry {
        ident: Ident,
        expect: TypeSpec,
        worker_usage: Vec<(Ident, Vec<Ident>)>,
        block: Expr,
        info: StmtInfo,
    },
    Panic {
        ident: Ident,
        expect: TypeSpec,
        generics: Vec<Generic>,
        worker_usage: Vec<(Ident, Vec<Ident>)>,
        params: Vec<InterfaceParam>,
        public: bool,
        block: Expr,
        info: StmtInfo,
    },
    Foo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum CustomLevel {
    // a custom toplevel
    MultiTakeStmt {
        take: Vec<(Ident, Option<Ident>)>, // "as" as an option
        from: Vec<Ident>,
        info: StmtInfo,
    },
    TakeStmt {
        take: (Ident, Option<Ident>), // "as" as an option
        from: Vec<Ident>,
        info: StmtInfo,
    },
    GlobalDecl {
        interpret: bool,
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        value: Expr,
        public: bool,
        info: StmtInfo,
    },
    MultiGlobalDecl(Vec<Box<TopLevel>>),
    EnumDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<EnumVariants>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        info: StmtInfo,
    },
    StructDecl {
        ident: Ident,
        generic_types: Vec<Generic>,
        vals: Vec<StructParam>,
        traits: Vec<(Ident, Vec<Ident>)>,
        public: bool,
        info: StmtInfo,
    },
    GenericType {
        name: Ident,
        traits: Vec<(Ident, Vec<Ident>)>,
    },
    FuncDecl {
        is_async: bool,
        expect: TypeSpec,
        ret_state: DataState,
        ident: Ident,
        generics: Vec<Generic>,
        is_public: bool,
        is_visible: bool,
        is_extern: Option<(Ident, Option<Span>)>,
        par: Vec<InterfaceParam>,
        worker_usage: Vec<(Ident, Vec<Ident>)>,
        block: Expr,
        info: StmtInfo,
    },
    // PlatformNS {
    //     arms: Vec<PlatformArm>,
    //     info: StmtInfo,
    // },
    WorkerDef {
        ident: Ident,
        func_name: Ident,
        public: bool,
        block: Vec<BlockLevel>,
        info: StmtInfo,
    },
    TraitDef {
        ident: Ident,
        generic_types: Option<Ident>,
        // interprets: Vec<BlockLevel>,
        methods: Vec<Method>,
        is_public: bool,
        info: StmtInfo,
    },
    StaticDef {
        ident: Ident,
        public: bool,
        block: CustomBlock,
        info: StmtInfo,
    },
    WorkerUsage {
        workers: Vec<String>,
        info: StmtInfo,
    },
    // MacroDef {
    //     ident: Ident,
    //     is_public: bool,
    //     cases: Vec<MacroCase>,
    //     info: StmtInfo,
    // },
    Foo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct TraitImpl {
    pub trt: Ident,
    pub ty: TypeSpec,
    pub methods: Vec<Method>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct BridgeDecl {
    pub ass_state: DataState,
    pub ass_type: TypeSpec,
    pub ass_ident: Ident,
    pub ass_is_mut: bool,
    pub new_ty: Ident,
    pub public: bool,
    pub block: Expr,
    pub info: StmtInfo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum BlockLevel {
    TakeStmt {
        take: (Ident, Option<Ident>), // "as" as an option
        from: Vec<Ident>,
        info: StmtInfo,
    },
    VarDecl {
        interpret: bool,
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        catch: Option<CatchFlag>,
        value: Expr,
        mutable: bool,
        info: StmtInfo,
    },
    UnInitVarDecl {
        state: DataState,
        ty: TypeSpec,
        name: Ident,
        mutable: bool,
        info: StmtInfo,
    },
    MultiVarDecl(Vec<Box<BlockLevel>>),
    MultiUnInitVarDecl(Vec<Box<BlockLevel>>),
    VarInit {
        ident: Ident,
        expr: Expr,
        info: StmtInfo,
    },
    ReAssign {
        to_mut: Expr,               // MutableAssess to be precise
        assign_op: Option<FullTok>, // like yí iye +: 54
        //             ^
        val: Expr,
        info: StmtInfo,
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
        expect: TypeSpec,
        ident: Ident,
        generics: Option<Vec<Generic>>,
        is_extern: Option<(Ident, Option<Span>)>,
        par: Vec<InterfaceParam>,
        worker_usage: Vec<Ident>,
        block: Expr,
        info: StmtInfo,
    },
    IfStmt {
        bool_expr: Expr,
        exe: Expr,
        else_ifs: Vec<ElseIf>,
        else_exe: Option<Expr>,
        info: StmtInfo,
    },
    LoopStmt {
        exe: Vec<BlockLevel>,
        info: StmtInfo,
    },
    WhileStmt {
        expr: Expr,
        exe: Vec<BlockLevel>,
        info: StmtInfo,
    },
    DoWhileStmt {
        exe: Vec<BlockLevel>,
        expr: Expr,
        info: StmtInfo,
    },
    PeekingStmt {
        param: Parameter,
        expr: Expr,
        exe: Vec<BlockLevel>,
        info: StmtInfo,
    },
    ForStmt {
        assign: ForAssign,
        cond: ForCond, // ... <= 5, ...
        incr: ForIncr, // ... + 1): ...
        exe: Vec<BlockLevel>,
        info: StmtInfo,
    },
    MatchStmt {
        var: Ident,
        arms: Vec<Case>,
        info: StmtInfo,
    },
    // PlatformNS {
    //     // platform namespace
    //     arms: Vec<PlatformArm>,
    //     info: StmtInfo,
    // },
    FuncCall {
        func: Expr,
        info: StmtInfo,
    },
    Block {
        stms: Vec<BlockLevel>,
        info: StmtInfo,
    },
    InterpretBlock {
        block: Vec<BlockLevel>,
        info: StmtInfo,
    },
    GlobalDecl {
        // only allowed in Interpret block.
        ty: TypeSpec,
        name: Ident,
        value: Expr,
        public: bool,
        info: StmtInfo,
    },
    MultiGlobalDecl(Vec<Box<BlockLevel>>),
    Break(StmtInfo),
    Continue(StmtInfo),
    Return(StmtInfo),
    RetVal {
        expr: Expr,
        info: StmtInfo,
    },
    FlowThrough(StmtInfo),
    BindedVal {
        expr: Expr,
        info: StmtInfo,
    },
    Foo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct StmtInfo {
    pub span: Span,
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DataState {
    Owner, // Alive
    Moved(Span),
    Ref,

    Global, // Always alive
    GlobalRef,
}

// #[derive(Debug, Clone, PartialEq)]
// #[allow(dead_code)]
// pub enum LibNextedLevel {
//     To(i32),
//     All,
// }

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct ForAssign {
    pub ty: TypeSpec, // fún (8p ...
    pub name: Ident,  // fún (8p i...
    // pub mutable: bool, // fún (8p i tóyí... // mutable by default
    pub value: Box<Expr>, // fún (8p i tóyí: 43...
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct ForCond {
    pub op: FullTok,
    pub expr: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct ForIncr {
    pub incr_op: FullTok,
    pub expr: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct CustomBlock {
    pub takes: Vec<CustomLevel>,
    pub globals: Vec<CustomLevel>,
    // pub macro_def: Vec<CustomLevel>,
    pub custom_tys: Vec<CustomLevel>,
    pub traits: Vec<CustomLevel>,
    pub workers: Vec<CustomLevel>,
    pub funcs: Vec<CustomLevel>,
    pub platform_n_s: Vec<CustomLevel>,
    pub static_defs: Vec<CustomLevel>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Ident {
    pub ident: String,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct CatchFlag {
    code: Ident,
    expr: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Trinary {
    True,
    False,
    Unknown,
}

impl Display for ModPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ModPath::Ident(i) => write!(f, "{}", i.ident),
            ModPath::Str(s) => write!(f, "{}", s.ident),
            ModPath::FromLibFolder(fol, modl) => write!(f, "{}/{}", fol.ident, modl.ident),
            ModPath::UnExpected => write!(f, "Unexpected"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModPath {
    Ident(Ident),
    Str(Ident),
    FromLibFolder(Ident, Ident),
    UnExpected,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Os {
    Windows(Span),
    Linux(Span),
    Darwin(Span),
    Others(Span),
    Undefined,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Arch {
    Arm,
    X86_64,
    Others(Span),
    Undefined,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct OsArm {
    pub target: Os,
    pub block: CustomBlock,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum MethodSelf {
    MutRef,
    Ref,
    Default,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Generic {
    pub name: Ident,
    pub traits: Vec<Ident>,
}

// #[derive(Debug, PartialEq, Clone)]
// #[allow(dead_code)]
// pub struct PlatformArm {
//     pub hardware: Hardware,
//     pub os: Os,
//     pub block: Vec<BlockLevel>,
//     pub span: Span,
// }

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum CustTyAss {
    Method(Method),
    AssFunction(AssFunc),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct AssFunc {
    pub worker_usage: Vec<Ident>,
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub ident: Ident,
    pub generics: Vec<Generic>,
    pub is_public: bool,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub par: Vec<InterfaceParam>,
    pub block: Option<Expr>,
    pub info: StmtInfo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Method {
    pub worker_usage: Vec<Ident>,
    pub meth_self: (MethodSelf, Span),
    // pub inher_val: MethInherVal,
    // pub is_mut: bool,
    pub expect: TypeSpec,
    pub ret_state: DataState,
    pub ident: Ident,
    pub generics: Option<Vec<Generic>>,
    pub is_public: Trinary,
    pub is_extern: Option<(Ident, Option<Span>)>,
    pub par: Vec<InterfaceParam>,
    pub block: Option<Expr>,
    pub info: StmtInfo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct ElseIf {
    pub bool_expr: Expr,
    pub exe: Expr,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct FieldAssignm {
    pub ident: Ident,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct StructPattern {
    pub ident: Ident,
    pub named: Ident,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum IdentAffix {
    ReAss(Span),
    Move(Span), // most the added by the programmer himself/herself
    Shared,     // added by the compiler for the programmer
    MutBorrow,  //
    Generics(Vec<TypeSpec>),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum MethInherVal {
    // Method's inherent value
    // it's a retricted pattern of Expr
    SelfVal(Span),
    Assess {
        namespace: Option<Ident>,
        value: Ident,
    },
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Asy {
    // Method's inherent value
    Await(Span),
    Start(Span),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ExprKind::Number(ref n) => write!(f, "{}", n),
            ExprKind::Float(ref fl) => write!(f, "{}", fl),
            ExprKind::Ident { affix: _, ident } => write!(f, "{}", ident.ident),
            ExprKind::Str(ref s) => write!(f, "\"{}\"", s),
            ExprKind::Char(ref c) => write!(f, "'{}'", c),
            ExprKind::TruthVal(ref tv) => match tv {
                TruthVal::Both => write!(f, "both"),
                TruthVal::False => write!(f, "false"),
                TruthVal::Unknown => write!(f, "none"),
                TruthVal::True => write!(f, "true"),
            },
            ExprKind::AsType { expr, ty } => {
                let strg = format!("({} as {})", expr.kind, ty.hint);

                write!(f, "'{}'", &strg)
            }
            ExprKind::Address(status, expr) => {
                let strg = format!(
                    "@{}{}",
                    match status {
                        MutStatus::Const => format!(""),
                        MutStatus::Mut(_) => format!("tóyí "),
                    },
                    format!("{}", expr.kind)
                );

                write!(f, "'{}'", &strg)
            }
            ExprKind::ArgBuffer {
                asy: _,
                generics: _,
                args: _,
                name,
            } => {
                write!(f, "{}", format!("{}(...)", name.ident))
            }
            _ => write!(f, "{}", format!("Expr")),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum ExprKind {
    // returns a value or is a value
    Number(String),
    Float(String),
    Ident {
        affix: Option<IdentAffix>,
        ident: Ident,
    },
    Import {
        imported: Box<Expr>,
        from: ModPath,
        lib: Option<Ident>,
    },
    Str(String),
    Char(char),
    TruthVal(TruthVal),
    MLStr(String), // Multiline
    Address(MutStatus, Box<Expr>),
    List(Vec<Expr>), // A vector/array
    StructVal {
        ident: Option<Ident>,
        vals: Vec<FieldAssignm>,
    },
    // Stmt(Box<BlockLevel>, Span),
    Block(Vec<BlockLevel>),
    Unary {
        op: FullTok,
        rhs: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        op: FullTok,
        rhs: Box<Expr>,
    },
    ObjAssess {
        obj: Box<Expr>,
        fields: Vec<Expr>,
    },
    MutableIdent {
        var: Ident,
        fields: Vec<Ident>,
    },
    ArgBuffer {
        // or sometime, an enum
        asy: Option<Asy>,
        generics: Vec<TypeSpec>,
        args: Vec<Expr>,
        name: Ident,
    },
    NamespaceAccess {
        ident: Option<Ident>,
        access: Vec<Expr>,
    },
    FuncGenericRet {
        ident: Ident,
        generics: Vec<TypeSpec>,
    },
    // Macro {
    //     name: Ident,
    //     delimiter: Delimiter,
    //     tokens: Vec<FullTok>,
    // },
    AsType {
        expr: Box<Expr>,
        ty: Box<TypeSpec>,
    },
    // control flow
    If {
        bool_expr: Box<Expr>,
        exe: Box<Expr>,
        else_ifs: Vec<ElseIf>,
        else_exe: Box<Option<Expr>>,
    },
    Match {
        var: Ident,
        arms: Vec<Case>,
    },
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
    Nexted {
        init: Box<Expr>,
        next: Box<Expr>,
    },
    Undefined,
    SelfVal,
    Default, // Used in Switch Statements
    Foo,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum Platform {
    Os,
    Arch,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum EnumVariantFormat {
    Simple,
    Tuple,
    Struct,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum MutStatus {
    Mut(Span),
    Const,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Parameter {
    pub ident: Ident,
    pub state: DataState,
    pub ty: TypeSpec,
    pub is_mut: bool,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct StructParam {
    pub par: Parameter,
    pub is_public: bool,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct InterfaceParam {
    pub ident: Ident,
    pub state: DataState,
    pub ty: TypeSpec,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Case {
    pub expr: Expr,
    pub exe: Expr, // which can also be a block
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct MacroCase {
    pub pattern: MacroGroup,
    pub block: Vec<FullTok>,
    pub info: StmtInfo,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct MacroGroup {
    pub delim: Delimiter,
    pub items: Vec<MacroPatt>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum MacroToken {
    Ident(String),
    Literal(String),
    Punct(char),
    Group(Delimiter, Vec<MacroToken>),
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum RepOperator {
    // Repeat operator
    ZeroOrMore,
    OnceOrMore,
    Optional,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum FragKind {
    Ident,
    Type,
    AnyType,
    Expr,
    Path,
    Stmt,
    Block,
    Item,
    Visibility,

    Custom(String),
    String,
    Bool,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct MacroParam {
    pub pattern: MacroGroup,
    pub delimiter: Delimiter,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum MacroPatt {
    Fragment {
        name: Ident,
        kind: FragKind,
    },
    Literal(FullTok),
    Group {
        delim: Delimiter,
        items: Vec<MacroPatt>,
    },
    Repetition {
        pattern: Box<MacroPatt>,
        seperator: Option<FullTok>,
        operator: RepOperator,
    },
    Foo,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
#[allow(dead_code)]
pub enum NumOfElem {
    Num(String),
    Nan,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct TypeSpec {
    pub hint: TypeHint,
    pub span: Span,
}

impl Display for TypeHint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeHint::I8(_) => write!(f, "d8"),
            TypeHint::I16(_) => write!(f, "d16"),
            TypeHint::I32(_) => write!(f, "d32"),
            TypeHint::I64(_) => write!(f, "d64"),
            TypeHint::I128(_) => write!(f, "d128"),
            TypeHint::ISize(_) => write!(f, "àtó_d"),
            TypeHint::U8(_) => write!(f, "p8"),
            TypeHint::U16(_) => write!(f, "p16"),
            TypeHint::U32(_) => write!(f, "p32"),
            TypeHint::U64(_) => write!(f, "p64"),
            TypeHint::U128(_) => write!(f, "p128"),
            TypeHint::USize(_) => write!(f, "àtó_p"),
            TypeHint::F32(_) => write!(f, "b32"),
            TypeHint::F64(_) => write!(f, "b64"),
            TypeHint::Str => write!(f, "ọsán"),
            TypeHint::Char => write!(f, "wóró"),
            TypeHint::Bool => write!(f, "ǹjẹ́"),
            TypeHint::Trinary => write!(f, "ibùta"),
            TypeHint::Quaternary => write!(f, "ibùrin"),
            TypeHint::End => write!(f, "òpin"),
            TypeHint::Custom { get, access, .. } => {
                if access.is_empty() {
                    write!(f, "{}", get.ident)
                } else {
                    write!(f, "{}", access.last().unwrap().ident)
                }
            }
            TypeHint::Void => write!(f, "òfìfo"),
            TypeHint::SelfKw => write!(f, "òwun"),
            TypeHint::RawPtr { tysp, .. } => write![f, "*{}", tysp.hint],
            // Literals
            TypeHint::Number(s) => write!(f, "number({})", s),
            TypeHint::CharLiteral => write!(f, "char"),
            TypeHint::StrLiteral => write!(f, "str"),
            TypeHint::ArityLiteral(_) => write!(f, "arity"),
            TypeHint::FuncGenericRet { ident, .. } => write!(f, "{}", ident.ident),
            TypeHint::EnumLiteral {
                ident: Some(id), ..
            } => write!(f, "{}", id.ident),
            TypeHint::EnumLiteral { ident: None, .. } => write!(f, "<enum>"),
            TypeHint::StructLiteral {
                ident: Some(id), ..
            } => write!(f, "{}", id.ident),
            TypeHint::StructLiteral { ident: None, .. } => write!(f, "<struct>"),
            _ => write!(f, "slice"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TypeHint {
    I8(Option<IntRange>),
    I16(Option<IntRange>),
    I32(Option<IntRange>),
    I64(Option<IntRange>),
    I128(Option<IntRange>),
    ISize(Option<IntRange>),
    U8(Option<IntRange>),
    U16(Option<IntRange>),
    U32(Option<IntRange>),
    U64(Option<IntRange>),
    U128(Option<IntRange>),
    USize(Option<IntRange>),
    F32(Option<IntRange>),
    F64(Option<IntRange>),
    Number(String),
    CharLiteral,
    StrLiteral,
    ArityLiteral(TruthVal), // ArityLiteral -> true, false, None, Both
    Str,
    Char,
    Bool,
    Trinary,
    Quaternary,
    End,
    // Option(Box<TypeSpec>),
    // Result(Box<TypeSpec>, Box<TypeSpec>),
    Custom {
        get: Ident,
        access: Vec<Ident>,
        generics: Vec<TypeSpec>, // None if it as no generics defination at all <>
    },
    FuncGenericRet {
        ident: Ident,
        generics: Vec<TypeSpec>,
    },
    EnumLiteral {
        ident: Option<Ident>, // for expression inferred types with infered name e.g .{} or .Funfun
        kind: Expr,
    },
    StructLiteral {
        ident: Option<Ident>, // for expression inferred types with infered name e.g .{} or .Funfun
        args: Vec<FieldAssignm>,
    },
    //
    Void,   // òfìfo for void functions
    SelfKw, // òwun
    RawPtr {
        tysp: Box<TypeSpec>,
        mutb: bool,
    },
    // Scaler(TypeHint), // Doesn't need span, type-hint has it
    MVSTy(Box<TypeHint>, NumOfElem), // Multiple Value - Same Types
    MVMTy(Vec<TypeHint>),            // Multiple Value - Multiple Types(TypeHint, number)
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
#[allow(dead_code)]
pub struct IntRangeKey {
    st: String,
    en: String,
}

impl Display for TypeKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeKey::I8(_) => write!(f, "d8"),
            TypeKey::I16(_) => write!(f, "d16"),
            TypeKey::I32(_) => write!(f, "d32"),
            TypeKey::I64(_) => write!(f, "d64"),
            TypeKey::I128(_) => write!(f, "d128"),
            TypeKey::ISize(_) => write!(f, "àtó_d"),
            TypeKey::U8(_) => write!(f, "p8"),
            TypeKey::U16(_) => write!(f, "p16"),
            TypeKey::U32(_) => write!(f, "p32"),
            TypeKey::U64(_) => write!(f, "p64"),
            TypeKey::U128(_) => write!(f, "p128"),
            TypeKey::USize(_) => write!(f, "àtó_p"),
            TypeKey::F32(_) => write!(f, "b32"),
            TypeKey::F64(_) => write!(f, "b64"),
            TypeKey::Str => write!(f, "ọsán"),
            TypeKey::Char => write!(f, "wóró"),
            TypeKey::Bool => write!(f, "ǹjẹ́"),
            TypeKey::Trinary => write!(f, "ibùta"),
            TypeKey::Quaternary => write!(f, "ibùrin"),
            TypeKey::End => write!(f, "òpin"),
            TypeKey::Custom(ident) => write!(f, "{}", ident),
            TypeKey::Void => write!(f, "òfìfo"),
            TypeKey::SelfKw => write!(f, "òwun"),
            TypeKey::RawPtr { tysp, .. } => write![f, "*{}", *tysp.clone()],

            _ => write!(f, "slice"),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[allow(dead_code)]
pub enum TypeKey {
    I8(Option<IntRangeKey>),
    I16(Option<IntRangeKey>),
    I32(Option<IntRangeKey>),
    I64(Option<IntRangeKey>),
    I128(Option<IntRangeKey>),
    ISize(Option<IntRangeKey>),
    U8(Option<IntRangeKey>),
    U16(Option<IntRangeKey>),
    U32(Option<IntRangeKey>),
    U64(Option<IntRangeKey>),
    U128(Option<IntRangeKey>),
    USize(Option<IntRangeKey>),
    F32(Option<IntRangeKey>),
    F64(Option<IntRangeKey>),
    Str,
    Char,
    Bool,
    Trinary,
    Quaternary,
    End,
    Custom(String),
    //
    Void,   // òfìfo for void functions
    SelfKw, // òwun
    RawPtr { tysp: Box<TypeKey>, mutb: bool },
    // Scaler(TypeHint), // Doesn't need span, type-hint has it
    MVSTy(Box<TypeKey>, NumOfElem), // Multiple Value - Same Types
    MVMTy(Vec<TypeKey>),            // Multiple Value - Multiple Types(TypeHint, number)
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Ownership {
    Owner,
    Borrower { mutb: bool },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct IntRange {
    pub st: String,
    pub en: String,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum EnumVariants {
    Simple(Ident),
    Tuple {
        name: Ident,
        fields: Vec<(DataState, TypeSpec)>,
    },
    Record {
        name: Ident,
        fields: Vec<Parameter>,
    },
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Dic {
    //Dictionaries in struct form
    pub ident: Ident,
    pub ty: TypeHint,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Error {
    pub message: String,
    pub span: Span,
}

// ---------------- Parser ----------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParseResult {
    pub mod_docs: Option<TopLevel>,
    pub mods: Vec<TopLevel>, // other files to be used
    pub takes: Vec<TopLevel>,
    pub globals: Vec<TopLevel>,
    // pub macro_def: Vec<TopLevel>,
    pub custom_tys: Vec<TopLevel>,
    pub traits: Vec<TopLevel>,
    pub workers: Vec<TopLevel>,
    pub statics: Vec<TopLevel>,
    pub funcs: Vec<TopLevel>,
    pub os_n_s: Vec<TopLevel>,
    pub arch_n_s: Vec<TopLevel>,
    pub main_fn: Option<TopLevel>,
    pub panic_fn: Option<TopLevel>,

    pub bridges: HashMap<String, Vec<BridgeDecl>>,
    pub methods: HashMap<TypeKey, Vec<CustTyAss>>,
    pub trait_impls: HashMap<TypeKey, Vec<TraitImpl>>,

    pub errs: Vec<Error>,
}

#[derive(Debug)]
pub struct Parser {
    file: String,
    pub tokens: Vec<FullTok>,
    pub pos: usize,
    worker_usages: Option<TopLevel>,
    kind: FileKind,
    //                 to            from
    bridges: HashMap<String, Vec<BridgeDecl>>,
    //                type              method
    methods: HashMap<TypeKey, Vec<CustTyAss>>,
    //                    type             trait
    trait_impls: HashMap<TypeKey, Vec<TraitImpl>>,
    zero_span: Span,
    pub errs: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<FullTok>, file: String, kind: FileKind) -> Self {
        let errs = vec![];
        Self {
            tokens,
            pos: 0,
            worker_usages: None,
            kind,
            bridges: HashMap::new(),
            methods: HashMap::new(),
            trait_impls: HashMap::new(),
            zero_span: Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: file.clone(),
            },
            file,
            errs,
        }
    }

    // --- helpers ---
    fn peek(&mut self) -> Option<FullTok> {
        let mut added_pos: usize = 0;
        loop {
            let f = self.tokens.get(self.pos);

            match f.cloned() {
                Some(f) => {
                    if f.kind == Token::NewLine {
                        self.pos += 1;
                        added_pos += 1;
                    } else {
                        self.pos -= added_pos;
                        return Some(f);
                    }
                }
                None => {
                    self.pos -= added_pos;
                    return None;
                }
            }
        }
    }

    fn peek_kind(&mut self) -> Option<Token> {
        let ret = self.peek();

        match ret {
            Some(f) => Some(f.kind),
            None => None,
        }
    }
    fn peek_span(&self) -> Span {
        // There is no need to return an option because every time we peek the next span the FullTok is already confirmed to be there
        if let Some(t) = self.tokens.get(self.pos) {
            return t.span.clone();
        }
        let span = Span {
            st: Pos { column: 0, line: 0 },
            en: Pos { column: 0, line: 0 },
            file: self.file.clone(),
        };
        span
    }
    // fn peek_n(&self, n: usize) -> Option<&Token> { self.tokens.get(self.pos + n) }
    fn advance(&mut self) -> Option<FullTok> {
        loop {
            let t = self.tokens.get(self.pos).cloned();
            if t.is_some() {
                self.pos += 1;
            }
            match t {
                Some(f) => {
                    if f.kind != Token::NewLine {
                        return Some(f);
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }

    fn reverse(&mut self) {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos -= 1;
        }
    }

    fn advance_kind(&mut self) -> Option<Token> {
        let ret = self.advance();

        match ret {
            Some(f) => Some(f.kind),
            None => None,
        }
    }

    fn is_eof(&mut self) -> bool {
        matches!(self.peek(), None)
    }

    fn expect_ident(&mut self) -> Ident {
        match self.advance() {
            Some(FullTok {
                kind: Token::Identifier(s),
                span,
            }) => Ident {
                ident: s.to_string(),
                span,
            },
            Some(oth) => {
                self.errs.push(Error {
                    message: format!("ident ni hó ńretí dípò {:?}", oth.kind),
                    span: Span {
                        st: Pos {
                            column: oth.span.st.column,
                            line: oth.span.st.line,
                        },
                        en: Pos {
                            column: oth.span.en.column,
                            line: oth.span.en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                Ident {
                    ident: String::new(),
                    span: self.zero_span.clone(),
                }
            }
            None => {
                self.errs.push(Error {
                    message: String::from("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: self.file.clone(),
                    },
                });

                Ident {
                    ident: format!(""),
                    span: self.zero_span.clone(),
                }
            }
        }
    }

    fn expect_typespec(&mut self) -> TypeSpec {
        let tok = self.advance();
        // Unsigneds
        let _u8 = String::from("8p");
        let _u16 = String::from("16p");
        let _u32 = String::from("32p");
        let _u64 = String::from("64p");
        let _u128 = String::from("128p");
        let _usize = String::from("ìtó_p");
        // Signeds
        let _i8 = String::from("8d");
        let _i16 = String::from("16d");
        let _i32 = String::from("32d");
        let _i64 = String::from("64d");
        let _i128 = String::from("128d");
        let _isize = String::from("ìtó_d");
        // Floats
        let _f32 = String::from("32b");
        let _f64 = String::from("64b");
        // Others
        let _str = String::from("ọsán");
        let _char = String::from("àkàyọ");
        let _bool = String::from("ǹjẹ́");
        let _trinary = String::from("ibùta");
        let _quaternary = String::from("ibùrin");
        let _end = String::from("òpin");
        // Void
        // let _void =String::from("òfìfo");

        match tok {
            // advanced
            Some(FullTok {
                kind: Token::TypeHint(s),
                span,
            }) => {
                match self.peek_kind() {
                    Some(Token::LBracket) => {
                        self.advance(); // if next is [ then advance

                        let nxt = self.advance();
                        match nxt {
                            Some(FullTok {
                                kind: Token::Number(n),
                                span: sp1,
                            }) => {
                                match self.advance() {
                                    Some(FullTok {
                                        kind: Token::DDot, ..
                                    }) => {
                                        // most be num type
                                        match self.advance() {
                                            Some(FullTok {
                                                kind: Token::Number(n2),
                                                span: sp2,
                                            }) => {
                                                self.expect(&Token::RBracket);
                                                if s == _u8 {
                                                    TypeSpec {
                                                        hint: TypeHint::U8(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _u16 {
                                                    TypeSpec {
                                                        hint: TypeHint::U16(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _u32 {
                                                    TypeSpec {
                                                        hint: TypeHint::U32(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _u64 {
                                                    TypeSpec {
                                                        hint: TypeHint::U64(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _u128 {
                                                    TypeSpec {
                                                        hint: TypeHint::U128(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _usize {
                                                    TypeSpec {
                                                        hint: TypeHint::USize(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _i8 {
                                                    TypeSpec {
                                                        hint: TypeHint::I8(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _i16 {
                                                    TypeSpec {
                                                        hint: TypeHint::I16(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _i32 {
                                                    TypeSpec {
                                                        hint: TypeHint::I32(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _i64 {
                                                    TypeSpec {
                                                        hint: TypeHint::I64(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _i128 {
                                                    TypeSpec {
                                                        hint: TypeHint::I128(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _isize {
                                                    TypeSpec {
                                                        hint: TypeHint::ISize(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _f32 {
                                                    TypeSpec {
                                                        hint: TypeHint::F32(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else if s == _f64 {
                                                    TypeSpec {
                                                        hint: TypeHint::F64(Some(IntRange {
                                                            st: n,
                                                            en: n2,
                                                            span: Span {
                                                                st: sp1.st,
                                                                en: sp2.en.clone(),
                                                                file: self.file.clone(),
                                                            },
                                                        })),
                                                        span,
                                                    }
                                                } else {
                                                    self.errs.push(Error{
                                                        message: String::from("ọ̀kan nínú àwọn irú iye ni oó yẹ kí oó wà níbí yìí"),
                                                        span: Span {
                                                            st: Pos {
                                                                column: self.peek_span().st.column,
                                                                line: self.peek_span().st.line
                                                            },
                                                            en: Pos {
                                                                column: self.peek_span().en.column,
                                                                line: self.peek_span().en.line
                                                            },
                                                            file: self.file.clone()
                                                        }
                                                    });

                                                    TypeSpec {
                                                        hint: TypeHint::Bool,
                                                        span: self.zero_span.clone(),
                                                    }
                                                }
                                            }
                                            Some(oth) => {
                                                self.errs.push(Error{
                                                    message: format!("Òǹkà ni oó yẹ kí oó wà ní ipò yìí, dípò, '{:?}' ni oó wà níbẹ̀", oth.kind),
                                                    span: Span {
                                                        st: Pos {
                                                            column: self.peek_span().st.column,
                                                            line: self.peek_span().st.line
                                                        },
                                                        en: Pos {
                                                            column: self.peek_span().en.column,
                                                            line: self.peek_span().en.line
                                                        },
                                                        file: self.file.clone()
                                                    }
                                                });

                                                TypeSpec {
                                                    hint: TypeHint::Bool,
                                                    span: self.zero_span.clone(),
                                                }
                                            }
                                            None => {
                                                self.errs.push(Error {
                                                    message: format!(
                                                        "Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"
                                                    ),
                                                    span: Span {
                                                        st: Pos {
                                                            column: self.peek_span().st.column,
                                                            line: self.peek_span().st.line,
                                                        },
                                                        en: Pos {
                                                            column: self.peek_span().en.column,
                                                            line: self.peek_span().en.line,
                                                        },
                                                        file: self.file.clone(),
                                                    },
                                                });

                                                TypeSpec {
                                                    hint: TypeHint::Bool,
                                                    span: self.zero_span.clone(),
                                                }
                                            }
                                        }
                                    }
                                    Some(FullTok {
                                        kind: Token::RBracket,
                                        ..
                                    }) => {
                                        // then it's most str type
                                        if s == _str {
                                            TypeSpec {
                                                hint: TypeHint::Str,
                                                span,
                                            }
                                        } else {
                                            self.errs.push(Error{
                                                        message: String::from("ọ̀kan nínú àwọn irú iye ni oó yẹ kí oó wà níbí yìí"),
                                                        span: Span {
                                                            st: Pos {
                                                                column: self.peek_span().st.column,
                                                                line: self.peek_span().st.line
                                                            },
                                                            en: Pos {
                                                                column: self.peek_span().en.column,
                                                                line: self.peek_span().en.line
                                                            },
                                                            file: self.file.clone()
                                                        }
                                                    });

                                            TypeSpec {
                                                hint: TypeHint::Bool,
                                                span: self.zero_span.clone(),
                                            }
                                        }
                                    }
                                    Some(oth) => {
                                        self.errs.push(Error{
                                            message: format!("Òǹkà ni oó yẹ kí oó wà ní ipò yìí, dípò, '{:?}' ni oó wà níbẹ̀", oth.kind),
                                            span: oth.span
                                        });

                                        TypeSpec {
                                            hint: TypeHint::Bool,
                                            span: self.zero_span.clone(),
                                        }
                                    }
                                    None => {
                                        self.errs.push(Error {
                                            message: format!(
                                                "Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"
                                            ),
                                            span: Span {
                                                st: Pos {
                                                    column: self.peek_span().st.column,
                                                    line: self.peek_span().st.line,
                                                },
                                                en: Pos {
                                                    column: self.peek_span().en.column,
                                                    line: self.peek_span().en.line,
                                                },
                                                file: self.file.clone(),
                                            },
                                        });

                                        TypeSpec {
                                            hint: TypeHint::Bool,
                                            span: self.zero_span.clone(),
                                        }
                                    }
                                }
                            }
                            Some(oth) => {
                                self.errs.push(Error {
                                    message: format!(
                                        "Òǹkà ni oó yẹ kí oó wà nípò yìí, dípò, '{:?}' ni oó wà níbẹ̀",
                                        oth.kind
                                    ),
                                    span: Span {
                                        st: Pos {
                                            column: self.peek_span().st.column,
                                            line: self.peek_span().st.line,
                                        },
                                        en: Pos {
                                            column: self.peek_span().en.column,
                                            line: self.peek_span().en.line,
                                        },
                                        file: self.file.clone(),
                                    },
                                });

                                TypeSpec {
                                    hint: TypeHint::Bool,
                                    span: self.zero_span.clone(),
                                }
                            }
                            None => {
                                self.errs.push(Error {
                                    message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                                    span: Span {
                                        st: Pos {
                                            column: self.peek_span().st.column,
                                            line: self.peek_span().st.line,
                                        },
                                        en: Pos {
                                            column: self.peek_span().en.column,
                                            line: self.peek_span().en.line,
                                        },
                                        file: self.file.clone(),
                                    },
                                });

                                TypeSpec {
                                    hint: TypeHint::Bool,
                                    span: self.zero_span.clone(),
                                }
                            }
                        }
                    }
                    Some(oth) => {
                        if s == _u8 {
                            TypeSpec {
                                hint: TypeHint::U8(None),
                                span,
                            }
                        } else if s == _u16 {
                            TypeSpec {
                                hint: TypeHint::U16(None),
                                span,
                            }
                        } else if s == _u32 {
                            TypeSpec {
                                hint: TypeHint::U32(None),
                                span,
                            }
                        } else if s == _u64 {
                            TypeSpec {
                                hint: TypeHint::U64(None),
                                span,
                            }
                        } else if s == _u128 {
                            TypeSpec {
                                hint: TypeHint::U128(None),
                                span,
                            }
                        } else if s == _usize {
                            TypeSpec {
                                hint: TypeHint::USize(None),
                                span,
                            }
                        } else if s == _i8 {
                            TypeSpec {
                                hint: TypeHint::I8(None),
                                span,
                            }
                        } else if s == _i16 {
                            TypeSpec {
                                hint: TypeHint::I16(None),
                                span,
                            }
                        } else if s == _i32 {
                            TypeSpec {
                                hint: TypeHint::I32(None),
                                span,
                            }
                        } else if s == _i64 {
                            TypeSpec {
                                hint: TypeHint::I64(None),
                                span,
                            }
                        } else if s == _i128 {
                            TypeSpec {
                                hint: TypeHint::I128(None),
                                span,
                            }
                        } else if s == _isize {
                            TypeSpec {
                                hint: TypeHint::ISize(None),
                                span,
                            }
                        } else if s == _f32 {
                            TypeSpec {
                                hint: TypeHint::F32(None),
                                span,
                            }
                        } else if s == _f64 {
                            TypeSpec {
                                hint: TypeHint::F64(None),
                                span,
                            }
                        } else if s == _str {
                            TypeSpec {
                                hint: TypeHint::Str,
                                span,
                            }
                        } else if s == _char {
                            TypeSpec {
                                hint: TypeHint::Char,
                                span,
                            }
                        } else if s == _bool {
                            TypeSpec {
                                hint: TypeHint::Bool,
                                span,
                            }
                        } else if s == _trinary {
                            TypeSpec {
                                hint: TypeHint::Trinary,
                                span,
                            }
                        } else if s == _quaternary {
                            TypeSpec {
                                hint: TypeHint::Quaternary,
                                span,
                            }
                        } else if s == _end {
                            TypeSpec {
                                hint: TypeHint::End,
                                span,
                            }
                        }
                        // else if s == _void {
                        //     println!("Next>>> {:#?}", self.peek_kind());
                        //     Ok(TypeSpec {
                        //         hint: TypeHint::Void,
                        //         span
                        //     })
                        // }
                        else {
                            self.errs.push(Error{
                                message: format!("Irú ẹṣà ni oó yẹ kí oó wà nípò yìí, dípò, '{:?}' ni oó wà níbẹ̀", oth),
                                span: Span {
                                    st: Pos {
                                        column: self.peek_span().st.column,
                                        line: self.peek_span().st.line
                                    },
                                    en: Pos {
                                        column: self.peek_span().en.column,
                                        line: self.peek_span().en.line
                                    },
                                    file: self.file.clone()
                                }
                            });

                            TypeSpec {
                                hint: TypeHint::Bool,
                                span: self.zero_span.clone(),
                            }
                        }
                    }
                    None => {
                        self.errs.push(Error {
                            message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                            span: Span {
                                st: Pos {
                                    column: self.peek_span().st.column,
                                    line: self.peek_span().st.line,
                                },
                                en: Pos {
                                    column: self.peek_span().en.column,
                                    line: self.peek_span().en.line,
                                },
                                file: self.file.clone(),
                            },
                        });

                        TypeSpec {
                            hint: TypeHint::Bool,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::Void,
                span,
            }) => TypeSpec {
                hint: TypeHint::Void,
                span,
            },
            // Handles custom types
            Some(FullTok {
                kind: Token::Identifier(c_ident),
                span: c_span,
            }) => {
                let get = Ident {
                    ident: c_ident,
                    span: c_span.clone(),
                };
                let mut st = c_span.st;
                let mut en = c_span.en;
                // Then token must be custom type
                // but also might be an import on spot
                let mut access = vec![];
                let mut generics = vec![];
                while self.peek_kind() == Some(Token::DColon) {
                    // then it's an import
                    // kí get::mod
                    self.advance();
                    st = self.peek_span().st;
                    access.push(self.expect_ident());
                    // we need to confirm if it's the last thus the type
                    match self.peek_kind() {
                        Some(Token::DColon) => {
                            // kí get::mod::
                            continue;
                        }
                        Some(Token::Less) => {
                            // then it's the last item; the type with generics
                            self.advance();

                            while self.peek_kind() != Some(Token::Greater) {
                                generics.push(self.expect_typespec());

                                if self.peek_kind() == Some(Token::Comma) {
                                    self.advance();
                                } else if self.peek_kind() == Some(Token::Greater) {
                                    self.advance();
                                    break;
                                } else {
                                    let kind = self.peek_kind();
                                    self.errs.push(Error {
                                        message: format!(
                                            "'{:?}' kọ́ ni oó yẹ kí oó wà níbí. Lo ',' dípò",
                                            kind
                                        ),
                                        span: Span {
                                            st: Pos {
                                                column: self.peek_span().st.column,
                                                line: self.peek_span().st.line,
                                            },
                                            en: Pos {
                                                column: self.peek_span().en.column,
                                                line: self.peek_span().en.line,
                                            },
                                            file: self.file.clone(),
                                        },
                                    });
                                }
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                    en = self.peek_span().en;
                }

                TypeSpec {
                    hint: TypeHint::Custom {
                        get,
                        access,
                        generics,
                    },
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                }
            }
            Some(FullTok {
                kind: Token::Star,
                span,
            }) => {
                let st = span.st.clone();

                let mutb;

                match self.peek_kind() {
                    Some(Token::Mut) => {
                        mutb = true;
                        self.advance();
                    }
                    _ => {
                        mutb = false;
                    }
                }

                let spec = self.expect_typespec();
                match &spec {
                    TypeSpec { hint: _, span } => TypeSpec {
                        hint: TypeHint::RawPtr {
                            tysp: Box::new(spec.clone()),
                            mutb,
                        },
                        span: Span {
                            st,
                            en: span.en.clone(),
                            file: self.file.clone(),
                        },
                    },
                }
            }
            Some(FullTok {
                kind: Token::SelfKw,
                span,
            }) => TypeSpec {
                hint: TypeHint::SelfKw,
                span,
            },
            Some(oth) => {
                self.errs.push(Error {
                    message: format!(
                        "Irú ẹṣà ni oó yẹ kí oó wà nípò yìí, dípò, '{:?}' ni oó wà níbẹ̀",
                        oth.kind
                    ),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                TypeSpec {
                    hint: TypeHint::Bool,
                    span: self.zero_span.clone(),
                }
            }
            None => {
                self.errs.push(Error {
                    message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                TypeSpec {
                    hint: TypeHint::Bool,
                    span: self.zero_span.clone(),
                }
            }
        }
    }

    fn expect_param(&mut self) -> Parameter {
        let state;
        if self.peek_kind() == Some(Token::Ampersand) {
            self.advance();

            state = DataState::Ref;
        } else {
            state = DataState::Owner;
        }

        let ty = self.expect_typespec();

        let ident = self.expect_ident();

        let is_mut;
        if self.peek_kind() == Some(Token::Mut) {
            self.advance();
            is_mut = true;
        } else {
            is_mut = false;
        }

        Parameter {
            ident,
            state,
            ty,
            is_mut,
        }
    }

    fn consume_newlines(&mut self) {
        loop {
            let full_tok = self.tokens.get(self.pos);

            match full_tok {
                Some(FullTok {
                    kind: Token::NewLine,
                    span: _,
                }) => {
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn expect_newline(&mut self) -> Span {
        let full_tok = self.tokens.get(self.pos);

        match full_tok {
            Some(FullTok {
                kind: Token::NewLine,
                span,
            }) => span.clone(),
            _ => Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.file.clone(),
            },
        }
    }

    fn expect(&mut self, t: &Token) -> Span {
        let sp;
        let peeked = self.peek();

        if peeked.is_some() {
            let peeked = peeked.unwrap();
            match peeked {
                FullTok { kind, span } => {
                    if *t == kind {
                        sp = span.clone()
                    } else {
                        match t {
                            &Token::LBrace => {
                                self.reverse();
                                self.errs.push(Error {
                                    message: format!("{{ ni òǹṣàkópọ̀ ń retí níbí"),
                                    span: span.clone(),
                                });
                                self.advance();
                                sp = span;
                            }
                            _ => {
                                self.reverse();
                                self.errs.push(Error {
                                    message: format!("{:?}' ni hó yẹ kí hó wà níbí", t,),
                                    span: span.clone(),
                                });
                                self.advance();
                                sp = span;
                            }
                        }
                    }
                }
            }
        } else {
            let span = Span {
                st: Pos { column: 0, line: 0 },
                en: Pos { column: 0, line: 0 },
                file: self.file.clone(),
            };
            self.errs.push(Error {
                message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                span: span.clone(),
            });
            sp = span;
        }

        self.advance();
        sp
    }

    // fn init_stm(&mut self, t: &Token) -> (Vec<Ident>, Pos) {
    //     let stm_st; // this is useful because calling self.peek_span can return the span of a newline character

    //     match self.peek() {
    //         Some(FullTok { kind: _, span }) => {
    //             stm_st = span.st;
    //         }
    //         None => {
    //             stm_st = Pos { column: 0, line: 0 };
    //         }
    //     }

    //     let mut ret = vec![];
    //     if self.worker_usages != None {
    //         let span = match self.worker_usages.clone() {
    //             Some(TopLevel::WorkerUsage { workers: _, info }) => info.span,
    //             _ => Span {
    //                 st: Pos { column: 0, line: 0 },
    //                 en: Pos { column: 0, line: 0 },
    //                 file: self.file.clone(),
    //             },
    //         };
    //         match t {
    //             &Token::Function | &Token::Enum | &Token::Struct => {
    //                 match self.worker_usages.clone() {
    //                     Some(TopLevel::WorkerUsage { workers, info: _ }) => {
    //                         ret = workers;
    //                     }
    //                     Some(_) => (),
    //                     None => {
    //                         self.errs.push(Error {
    //                             message: format!("Ìlo-àríjú yìí-ò wúlò níbí"),
    //                             span: Span {
    //                                 // This spans the whole statement
    //                                 st: Pos {
    //                                     column: span.st.column,
    //                                     line: span.st.line,
    //                                 },
    //                                 en: Pos {
    //                                     column: span.en.column,
    //                                     line: span.en.line,
    //                                 },
    //                                 file: self.file.clone(),
    //                             },
    //                         })
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 match &self.worker_usages {
    //                     Some(TopLevel::WorkerUsage { workers: _, info }) => self.errs.push(Error {
    //                         message: format!("Ìlo-àríjú yìí-ò wúlò níbí"),
    //                         span: info.span.clone(),
    //                     }),
    //                     _ => {
    //                         self.errs.push(Error {
    //                             message: format!("Ìlo-àríjú yìí-ò wúlò níbí"),
    //                             span: Span {
    //                                 // This spans the whole statement
    //                                 st: Pos {
    //                                     column: span.st.column,
    //                                     line: span.st.line,
    //                                 },
    //                                 en: Pos {
    //                                     column: span.en.column,
    //                                     line: span.en.line,
    //                                 },
    //                                 file: self.file.clone(),
    //                             },
    //                         });
    //                     }
    //                 }
    //             }
    //         }
    //         self.worker_usages = None;
    //     }

    //     self.advance();

    //     (ret, stm_st)
    // }

    // --- program / top-level --- \\
    pub fn parse_program(&mut self) -> ParseResult {
        let mut mods = vec![];
        let mut mod_docs = None;
        let mut globals = vec![];
        let mut funcs = vec![];
        let mut os_n_s = vec![];
        let mut arch_n_s = vec![];
        let mut takes = vec![];
        // let mut macro_def = vec![];
        let mut custom_tys = vec![];
        let mut workers = vec![];
        let mut statics = vec![];
        let mut traits = vec![];
        let mut main_fn = None;
        let mut panic_fn = None;

        while !self.is_eof() {
            let result = self.parse_toplevel(false);

            if self.errs.is_empty() {
                match result.clone() {
                    TopLevel::DocLines { .. } => {
                        mod_docs = Some(result);
                    }
                    TopLevel::UseStmt {
                        path,
                        is_sec_entry,
                        alias,
                        is_public,
                        info,
                    } => {
                        mods.push(TopLevel::UseStmt {
                            path,
                            is_sec_entry,
                            alias,
                            is_public,
                            info,
                        });
                    }
                    TopLevel::UseLibStmt {
                        include,
                        ackn_panic,
                        info,
                    } => {
                        mods.push(TopLevel::UseLibStmt {
                            include,
                            ackn_panic,
                            info,
                        });
                    }
                    TopLevel::MultiTakeStmt { take, from, info } => {
                        // converts the multiple takes into individual takes

                        for t in take {
                            takes.push(TopLevel::TakeStmt {
                                take: t,
                                from: from.clone(),
                                info: info.clone(),
                            });
                        }
                    }
                    TopLevel::TakeStmt { take, from, info } => {
                        takes.push(TopLevel::TakeStmt { take, from, info });
                    }
                    // TopLevel::WorkerUsage { workers, info } => {
                    //     self.worker_usages = // this field is checked for every statement being parsed
                    //             Some(TopLevel::WorkerUsage { workers, info });
                    // }
                    TopLevel::GlobalDecl {
                        interpret,
                        state,
                        ty,
                        name,
                        value,
                        public,
                        info,
                    } => {
                        globals.push(TopLevel::GlobalDecl {
                            interpret,
                            state,
                            ty,
                            name,
                            value,
                            public,
                            info,
                        });
                    }
                    TopLevel::MultiGlobalDecl(v) => {
                        for each in v {
                            let glob = *each;
                            globals.push(glob);
                        }
                    }
                    TopLevel::Entry {
                        ident,
                        expect,
                        worker_usage,
                        block,
                        info,
                    } => {
                        main_fn = Some(TopLevel::Entry {
                            ident,
                            expect,
                            worker_usage,
                            block,
                            info,
                        });
                    }
                    TopLevel::Panic {
                        ident,
                        expect,
                        worker_usage,
                        params,
                        public,
                        generics,
                        block,
                        info,
                    } => {
                        panic_fn = Some(TopLevel::Panic {
                            ident,
                            expect,
                            worker_usage,
                            params,
                            public,
                            block,
                            generics,
                            info,
                        });
                    }
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
                        funcs.push(TopLevel::FuncDecl {
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
                    TopLevel::OsTarget { arms, info } => {
                        os_n_s.push(TopLevel::OsTarget { arms, info })
                    }
                    TopLevel::ArchTarget { arms, info } => {
                        arch_n_s.push(TopLevel::ArchTarget { arms, info });
                    }
                    // TopLevel::MacroDef {
                    //     ident,
                    //     is_public,
                    //     cases,
                    //     info,
                    // } => {
                    //     macro_def.push(TopLevel::MacroDef {
                    //         ident,
                    //         is_public,
                    //         cases,
                    //         info,
                    //     });
                    // }
                    TopLevel::EnumDecl {
                        ident,
                        generic_types,
                        vals,
                        traits,
                        public,
                        info,
                    } => {
                        custom_tys.push(TopLevel::EnumDecl {
                            ident,
                            generic_types,
                            vals,
                            traits,
                            public,
                            info,
                        });
                    }
                    TopLevel::StructDecl {
                        ident,
                        generic_types,
                        vals,
                        traits,
                        public,
                        info,
                    } => {
                        custom_tys.push(TopLevel::StructDecl {
                            ident,
                            generic_types,
                            vals,
                            traits,
                            public,
                            info,
                        });
                    }
                    TopLevel::GenericType { name: _, traits: _ } => (), // Non used in parser
                    TopLevel::TraitDef {
                        ident,
                        generic_types,
                        // interprets,
                        methods,
                        is_public,
                        info,
                    } => {
                        traits.push(TopLevel::TraitDef {
                            ident,
                            generic_types,
                            // interprets,
                            methods,
                            is_public,
                            info,
                        });
                    }
                    TopLevel::WorkerDef {
                        ident,
                        func_name,
                        public,
                        block,
                        info,
                    } => {
                        workers.push(TopLevel::WorkerDef {
                            ident,
                            func_name,
                            public,
                            block,
                            info,
                        });
                    }
                    TopLevel::StaticDef {
                        ident,
                        public,
                        block,
                        info,
                    } => {
                        let static_def = TopLevel::StaticDef {
                            ident,
                            public,
                            block,
                            info,
                        };
                        statics.push(static_def);
                    }
                    TopLevel::Foo => (), //   _ => {
                                         //       items.push(value);
                                         //   }
                }
            }
        }

        // Before returning ParseResult, put each bridge in there place

        ParseResult {
            mods,
            mod_docs,
            takes,
            globals,
            // macro_def,
            custom_tys,
            traits,
            workers,
            statics,
            funcs,
            os_n_s,
            arch_n_s,
            main_fn,
            panic_fn,

            bridges: self.bridges.clone(),
            methods: self.methods.clone(),
            trait_impls: self.trait_impls.clone(),
            errs: self.errs.clone(),
        }
    }

    pub fn parse_stmts(&mut self) -> (Vec<BlockLevel>, Pos) {
        let mut items = vec![];

        // self.expect(&Token::LBrace);

        while self.peek_kind() != Some(Token::RBrace) {
            let value = self.parse_blocklevel();

            match value.clone() {
                BlockLevel::MultiGlobalDecl(g) => {
                    for glob in g {
                        match *glob.clone() {
                            BlockLevel::GlobalDecl {
                                ty: _,
                                name: _,
                                value,
                                public: _,
                                info: _,
                            } => match value {
                                Expr {
                                    kind: ExprKind::Nexted { init: _, next },
                                    span,
                                } => {
                                    self.errs.push(Error {
                                        message: format!(
                                            "You can't return value {:?} at toplevel",
                                            next
                                        ),
                                        span: Span {
                                            st: Pos {
                                                column: span.st.column,
                                                line: span.st.line,
                                            },
                                            en: Pos {
                                                column: span.en.column,
                                                line: span.en.line,
                                            },
                                            file: self.file.clone(),
                                        },
                                    });
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                        items.push(*glob);
                    }
                }
                BlockLevel::GlobalDecl {
                    ty: _,
                    name: _,
                    value: expr,
                    public: _,
                    info: _,
                } => {
                    match expr {
                        Expr {
                            kind: ExprKind::Nexted { init: _, next },
                            span,
                        } => {
                            self.errs.push(Error {
                                message: format!("You can't return value {:?} at toplevel", next),
                                span: Span {
                                    st: Pos {
                                        column: span.st.column,
                                        line: span.st.line,
                                    },
                                    en: Pos {
                                        column: span.en.column,
                                        line: span.en.line,
                                    },
                                    file: self.file.clone(),
                                },
                            });
                        }
                        _ => (),
                    }
                    items.push(value);
                }
                BlockLevel::MultiVarDecl(v) => {
                    for var in v {
                        match *var.clone() {
                            BlockLevel::VarDecl {
                                interpret,
                                state,
                                ty,
                                name,
                                catch,
                                value,
                                mutable,
                                info,
                            } => {
                                match value {
                                    Expr {
                                        kind: ExprKind::Nexted { init, next },
                                        span: nexted_span,
                                    } => {
                                        items.push(BlockLevel::VarDecl {
                                            interpret,
                                            state,
                                            ty,
                                            name,
                                            catch,
                                            value: *init,
                                            mutable,
                                            info,
                                        });
                                        items.push(BlockLevel::BindedVal {
                                            expr: *next,
                                            info: StmtInfo {
                                                span: nexted_span,
                                                docs: vec![],
                                            },
                                        }); // The span needs to be of the next expr to_do()
                                    }
                                    _ => {
                                        items.push(*var);
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                BlockLevel::MultiUnInitVarDecl(v) => {
                    for var in v {
                        match *var.clone() {
                            BlockLevel::UnInitVarDecl {
                                state: _,
                                ty: _,
                                name: _,
                                mutable: _,
                                info: _,
                            } => {
                                items.push(*var);
                            }
                            _ => (),
                        }
                    }
                }
                BlockLevel::FuncDecl {
                    expect: _,
                    ident: _,
                    generics: _,
                    is_extern: _,
                    worker_usage: _,
                    par: _,
                    block: _,
                    info: _,
                } => {
                    items.push(value);
                }
                BlockLevel::VarDecl {
                    interpret,
                    state,
                    ty,
                    name,
                    catch,
                    value: var_val,
                    mutable,
                    info,
                } => {
                    match var_val {
                        Expr {
                            kind: ExprKind::Nexted { init, next },
                            span: nexted_span,
                        } => {
                            items.push(BlockLevel::VarDecl {
                                interpret,
                                state,
                                ty,
                                name,
                                catch,
                                value: *init,
                                mutable,
                                info,
                            });
                            items.push(BlockLevel::BindedVal {
                                expr: *next,
                                info: StmtInfo {
                                    span: nexted_span,
                                    docs: vec![],
                                },
                            }); // The span needs to be of the next expr to_do()
                        }
                        _ => {
                            items.push(value);
                        }
                    }
                }
                BlockLevel::UnInitVarDecl {
                    state: _,
                    ty: _,
                    name: _,
                    mutable: _,
                    info: _,
                } => {
                    items.push(value);
                }
                BlockLevel::BindedVal { expr, info } => match expr.clone() {
                    Expr {
                        kind: ExprKind::Nexted { init, next },
                        span: nexted_span,
                    } => {
                        items.push(BlockLevel::BindedVal { expr: *init, info });
                        items.push(BlockLevel::BindedVal {
                            expr: *next,
                            info: StmtInfo {
                                span: nexted_span,
                                docs: vec![],
                            },
                        });
                    }
                    _ => {
                        items.push(value);
                    }
                },
                _ => {
                    items.push(value);
                }
            }
        }

        let stm_en = self.peek_span().en.clone();
        self.expect(&Token::RBrace);
        // comsume the ending only if it's not a switch case block

        (items, stm_en)
    }

    fn parse_toplevel(&mut self, is_platform: bool) -> TopLevel {
        match self.peek() {
            Some(FullTok {
                kind: Token::DocLines(_),
                span: _,
            }) => self.parse_mod_docs(),
            Some(FullTok {
                kind: Token::Os,
                span: _,
            }) => self.parse_os_target(),
            Some(FullTok {
                kind: Token::Use,
                span: _,
            }) => self.parse_use(),
            Some(FullTok {
                kind: Token::Take,
                span: _,
            }) => self.parse_take(),
            Some(FullTok {
                kind: Token::Global,
                span: _,
            }) => self.parse_global(),
            Some(FullTok {
                kind: Token::Bridge,
                span: _,
            }) => self.parse_bridge(),
            Some(FullTok {
                kind: Token::Enum,
                span: _,
            }) => self.parse_enum(),
            Some(FullTok {
                kind: Token::Struct,
                span: _,
            }) => self.parse_struct(),
            Some(FullTok {
                kind: Token::Async,
                span: _,
            }) => self.parse_func(is_platform),
            Some(FullTok {
                kind: Token::Function,
                span: _,
            }) => self.parse_func(is_platform),
            Some(FullTok {
                kind: Token::Interpret,
                span: _,
            }) => self.parse_global(),
            Some(FullTok {
                kind: Token::Worker,
                span: _,
            }) => self.parse_worker(),
            // Some(FullTok {
            //     kind: Token::Static,
            //     span: _,
            // }) => self.parse_static(),
            Some(FullTok {
                kind: Token::Trait,
                span: _,
            }) => self.parse_trait(),
            Some(FullTok {
                kind: Token::NumSign,
                span: _,
            }) => self.parse_worker_usage(is_platform),
            Some(FullTok {
                kind: Token::Apply,
                span: _,
            }) => self.parse_impl(),
            // Some(FullTok {
            //     kind: Token::Macro,
            //     span: _,
            // }) => self.parse_macro(),
            Some(oth) => match &oth.kind {
                Token::Identifier(i) => {
                    self.errs.push(Error {
                        message: format!("Ident '{}'-ò yẹ kí oó wà níbí", i),
                        span: Span {
                            st: Pos {
                                column: self.peek_span().st.column,
                                line: self.peek_span().st.line,
                            },
                            en: Pos {
                                column: self.peek_span().en.column,
                                line: self.peek_span().en.line,
                            },
                            file: self.file.clone(),
                        },
                    });

                    TopLevel::Foo
                }
                Token::TypeHint(t) => {
                    self.errs.push(Error {
                        message: format!("Irú ẹṣà '{}'-ò yẹ kí oó wà níbí", t),
                        span: Span {
                            st: Pos {
                                column: self.peek_span().st.column,
                                line: self.peek_span().st.line,
                            },
                            en: Pos {
                                column: self.peek_span().en.column,
                                line: self.peek_span().en.line,
                            },
                            file: self.file.clone(),
                        },
                    });

                    TopLevel::Foo
                }
                oth => {
                    self.errs.push(Error {
                        message: format!("'{:?}'-ò yẹ kí oó wà níbí", oth),
                        span: Span {
                            st: Pos {
                                column: self.peek_span().st.column,
                                line: self.peek_span().st.line,
                            },
                            en: Pos {
                                column: self.peek_span().en.column,
                                line: self.peek_span().en.line,
                            },
                            file: self.file.clone(),
                        },
                    });

                    TopLevel::Foo
                }
            },
            None => {
                self.errs.push(Error {
                    message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                TopLevel::Foo
            }
        }
    }

    fn is_expr_block(&self, expr: &Expr) -> bool {
        match expr {
            Expr {
                kind: ExprKind::Block(_),
                ..
            } => true,
            _ => false,
        }
    }

    fn parse_blocklevel(&mut self) -> BlockLevel {
        match self.peek() {
            Some(FullTok {
                kind: Token::Take,
                span: _,
            }) => {
                let top_level_take = self.parse_take();

                match top_level_take {
                    TopLevel::TakeStmt { take, from, info } => {
                        BlockLevel::TakeStmt { take, from, info }
                    }
                    _ => BlockLevel::Foo,
                }
            }
            // Some(FullTok {
            //     kind: Token::Platform,
            //     span: _,
            // }) => Ok(self.parse_b_l_platform()?),
            Some(FullTok {
                kind: Token::If,
                span: _,
            }) => self.parse_if(),
            Some(FullTok {
                kind: Token::For,
                span: _,
            }) => self.parse_for(),
            Some(FullTok {
                kind: Token::Peeking,
                span: _,
            }) => self.parse_peeking(),
            Some(FullTok {
                kind: Token::Var,
                span: _,
            }) => self.parse_var(),
            Some(FullTok {
                kind: Token::Have,
                span: _,
            }) => self.parse_uninit_var(),
            Some(FullTok {
                kind: Token::Assign,
                span: _,
            }) => self.parse_reassign(),
            Some(FullTok {
                kind: Token::Function,
                span: _,
            }) => self.parse_nexted_func(),
            Some(FullTok {
                kind: Token::Tilde,
                span: _,
            }) => self.parse_func_call(),
            Some(FullTok {
                kind: Token::Loop,
                span: _,
            }) => self.parse_loop(),
            Some(FullTok {
                kind: Token::While,
                span: _,
            }) => self.parse_while(),
            Some(FullTok {
                kind: Token::Match,
                span: _,
            }) => self.parse_match(),
            Some(FullTok {
                kind: Token::Interpret,
                span: _,
            }) => self.parse_var(),
            Some(FullTok {
                kind: Token::Do,
                span: _,
            }) => self.parse_do_while(),
            Some(FullTok {
                kind: Token::Colon,
                span: _,
            }) => self.parse_expr_call_or_var_init(),
            Some(FullTok {
                kind: Token::RBrace,
                span: _,
            }) => self.parse_block(),
            Some(FullTok {
                kind: Token::Identifier(_),
                span: _,
            }) => self.parse_var_init(),
            Some(FullTok {
                kind: Token::Break,
                span: _,
            }) => {
                let st = self.peek_span().st.clone();
                self.advance();
                let en = self.peek_span().en.clone();

                let docs = self.end_stmt_parse(false);
                BlockLevel::Break(StmtInfo {
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                    docs,
                })
            }
            Some(FullTok {
                kind: Token::Continue,
                span: _,
            }) => {
                let st = self.peek_span().st.clone();
                self.advance();
                let en = self.peek_span().en.clone();

                let docs = self.end_stmt_parse(false);
                BlockLevel::Continue(StmtInfo {
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                    docs,
                })
            }
            Some(FullTok {
                kind: Token::FlowThrough,
                span: _,
            }) => {
                let st = self.peek_span().st.clone();
                self.advance();
                let en = self.peek_span().en.clone();

                let docs = self.end_stmt_parse(false);

                BlockLevel::FlowThrough(StmtInfo {
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                    docs,
                })
            }
            Some(FullTok {
                kind: Token::Return,
                span: _,
            }) => {
                let st = self.peek_span().st.clone();
                self.advance();
                let en = self.peek_span().en.clone();

                let docs = self.end_stmt_parse(false);

                BlockLevel::Return(StmtInfo {
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                    docs,
                })
            }
            Some(FullTok {
                kind: Token::RetVal,
                span,
            }) => {
                let st = span.st.clone();
                self.advance();

                let expr = self.parse_expr(0, false);
                let en = self.peek_span().en.clone();

                let docs = self.end_stmt_parse(self.is_expr_block(&expr));

                BlockLevel::RetVal {
                    expr,
                    info: StmtInfo {
                        span: Span {
                            st,
                            en,
                            file: self.file.clone(),
                        },
                        docs,
                    },
                }
            }
            Some(oth) => {
                self.errs.push(Error {
                    message: format!(
                        "'{:?}' kọ́ ni oó yẹ kí oó wà níbí\n bóyá kí o fi ':' sí wájú rẹ",
                        oth.kind
                    ),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                BlockLevel::Foo
            }
            None => {
                self.errs.push(Error {
                    message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                BlockLevel::Foo
            }
        }
    }

    fn end_stmt_parse(&mut self, is_block_ending: bool) -> Vec<String> {
        let mut docs = vec![];

        while let Some(Token::LineDoc(strg)) = self.peek_kind() {
            docs.push(strg);
            self.advance();
        }

        if docs.len() > 0 || !is_block_ending {
            // this should be done either when there is docs or
            // the expression is not a block
            let nxt = self.peek_kind();
            if nxt != Some(Token::RBrace) {
                self.expect_newline();
            }
        }

        docs
    }

    // fn parse_mod_pub(&mut self) -> Result<TopLevel, ()> {
    //     let st = self.expect(&Token::DMinus).st;

    //     let mod_pub_name = self.expect_ident()?;

    //     self.expect(&Token::Public);

    //     let en = self.expect(&Token::DMinus).en;

    //     self.expect_newline();

    //     Ok(TopLevel::ModPub {
    //         mod_pub_name,
    //         span: Span {
    //             st,
    //             en,
    //             file: self.file.clone(),
    //         },
    //     })
    // }

    fn parse_mod_docs(&mut self) -> TopLevel {
        let mod_docs = self.advance();
        if mod_docs.is_some() {
            let mod_docs = mod_docs.unwrap();
            match mod_docs.kind {
                Token::DocLines(docs) => TopLevel::DocLines {
                    lines: docs.clone(),
                    span: mod_docs.span,
                },
                _ => {
                    self.errs.push(Error {
                        message: format!("Unexpected Expression for Module documentation"),
                        span: mod_docs.span,
                    });

                    TopLevel::Foo
                }
            }
        } else {
            TopLevel::Foo
        }
    }

    // --- Use Module ---
    fn parse_use_helper(
        &mut self,
        mod_str: String,
        mod_span: Span,
        is_sec_entry: bool,
        stm_st: Pos,
    ) -> TopLevel {
        let path = Ident {
            ident: mod_str,
            span: mod_span,
        };

        self.expect(&Token::Colon);

        let alias = self.expect_ident();
        // let as_ = ident.ident;

        let mut is_public = false;
        if self.peek_kind() == Some(Token::Public) {
            self.advance();
            is_public = true;
        }

        let stm_en = alias.span.en.clone();

        let docs = self.end_stmt_parse(false);
        TopLevel::UseStmt {
            path,
            is_sec_entry,
            alias,
            is_public,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        }
    }
    fn parse_use(&mut self) -> TopLevel {
        let span = self.expect(&Token::Use);
        let stm_st = span.st;

        if self.kind == FileKind::Simple {
            // emit error
            self.errs.push(Error {
                message: format!("lo statement cannot be used in a non-entry module"),
                span: span.clone(),
            });
        } // but parse it too

        match self.advance() {
            Some(FullTok {
                kind: Token::StringLiteral(mod_str),
                span: mod_span,
            }) => self.parse_use_helper(mod_str, mod_span, false, stm_st),
            Some(FullTok {
                kind: Token::Star,
                span: _,
            }) => {
                self.advance();
                match self.peek() {
                    Some(FullTok {
                        kind: Token::StringLiteral(mod_str),
                        span: mod_span,
                    }) => {
                        self.advance();
                        self.parse_use_helper(mod_str, mod_span, true, stm_st)
                    }
                    _ => {
                        todo!("Expected string literal error")
                    }
                }
            }
            Some(FullTok {
                kind: Token::Identifier(mod_ident),
                span: mod_span,
            }) => {
                if self.kind == FileKind::SecEntry {
                    self.errs.push(Error {
                        message: format!(
                            "Library inclusion is not allowed on a secondary entry module"
                        ),
                        span,
                    });
                }
                let include = Ident {
                    ident: mod_ident,
                    span: mod_span.clone(),
                };

                let ackn_panic;
                match self.peek_kind() {
                    Some(Token::AcknPanic) => {
                        self.advance();
                        ackn_panic = true;
                    }
                    _ => {
                        ackn_panic = false;
                    }
                }

                let docs = self.end_stmt_parse(false);

                TopLevel::UseLibStmt {
                    include,
                    ackn_panic,
                    info: StmtInfo {
                        span: Span {
                            st: stm_st,
                            en: mod_span.en,
                            file: self.file.clone(),
                        },
                        docs,
                    },
                }
            }
            _ => {
                self.errs.push(Error {
                    message: format!("Ipa fáílì náà gbodọ̀ wa nínu \" \""),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                TopLevel::Foo
            }
        }
    }

    fn get_take_elem(&mut self) -> (Ident, Option<Ident>) {
        let take;

        let ident;
        let st = self.peek_span().st.clone();
        let en = self.peek_span().en.clone();
        match self.advance() {
            Some(FullTok {
                kind: Token::Identifier(i),
                span,
            }) => {
                ident = Ident { ident: i, span };
            }
            // Some(FullTok {
            //     kind: Token::MacroCall(i),
            //     span,
            // }) => {
            //     ident = Ident {
            //         ident: i,
            //         span,
            //     };
            // }
            _ => {
                ident = Ident {
                    ident: String::from(""),
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: self.file.clone(),
                    },
                };

                self.errs.push(Error {
                    message: format!(""),
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                });
            }
        }

        match self.peek_kind() {
            Some(Token::NameAs) => {
                self.advance();

                let as_result = self.expect_ident();

                let to_be_called = Some(as_result);

                take = (ident, to_be_called);
            }
            _ => {
                take = (ident, None);
            }
        }

        take
    }

    fn get_take_mutip_elems(&mut self) -> Vec<(Ident, Option<Ident>)> {
        let mut takes = Vec::new();
        while self.peek_kind() != Some(Token::RBrace) {
            takes.push(self.get_take_elem());

            let nxt = self.peek_kind();

            if nxt == Some(Token::RBrace) {
                self.advance();
                break;
            } else {
                if nxt != Some(Token::Comma) {
                    self.errs.push(Error {
                        message: format!("Comma gbọdọ̀ dá wọn láàrin"),
                        span: Span {
                            st: Pos {
                                column: self.peek_span().st.column,
                                line: self.peek_span().st.line,
                            },
                            en: Pos {
                                column: self.peek_span().en.column,
                                line: self.peek_span().en.line,
                            },
                            file: self.file.clone(),
                        },
                    })
                }
                self.advance();
            }
        }

        takes
    }

    // --- The module import ---
    fn parse_take(&mut self) -> TopLevel {
        // For: mú blablabla::xyz
        let stm_st = self.peek_span().st.clone();
        self.expect(&Token::Take);

        let mut from = vec![self.expect_ident()];
        self.expect(&Token::DColon);
        // mú lib::
        let mut ends_on_dcolon = false;
        let mut stm_en = self.peek_span().en;
        while let Some(FullTok {
            kind: Token::Identifier(ident),
            span,
        }) = self.peek()
        {
            // mú lib::ident
            self.advance();
            stm_en = span.en.clone();
            from.push(Ident { ident, span });
            if self.peek_kind() == Some(Token::DColon) {
                self.advance();
                ends_on_dcolon = true;
            } else {
                ends_on_dcolon = false;
                break;
            }
        }

        // after :: or ident(where :: doesn't follow)
        if ends_on_dcolon {
            // after ::
            match self.peek() {
                Some(FullTok {
                    kind: Token::LBrace,
                    ..
                }) => {
                    self.advance();
                    // mú mod::{...}
                    let take = self.get_take_mutip_elems();

                    let docs = self.end_stmt_parse(true);

                    TopLevel::MultiTakeStmt {
                        take,
                        from,
                        info: StmtInfo {
                            span: Span {
                                st: stm_st,
                                en: stm_en,
                                file: self.file.clone(),
                            },
                            docs,
                        },
                    }
                }
                _ => {
                    todo!("Unexpected import element error")
                }
            }
        } else {
            // after ident(where :: doesn't follow)
            // mú lib::ident
            let take = from.pop();
            let alias = if self.peek_kind() == Some(Token::NameAs) {
                self.advance();

                Some(self.expect_ident())
            } else {
                None
            };

            if take.is_some() {
                let take = take.unwrap();
                let docs = self.end_stmt_parse(true);

                TopLevel::TakeStmt {
                    take: (take, alias),
                    from,
                    info: StmtInfo {
                        span: Span {
                            st: stm_st,
                            en: stm_en,
                            file: self.file.clone(),
                        },
                        docs,
                    },
                }
            } else {
                todo!("NOT found error")
            }
        }
        // match self.peek() {
        //     Some(FullTok {
        //         kind: Token::LBrace,
        //         ..
        //     }) => {
        //         self.advance();
        //         // mú mod/{...}
        //         let take = self.get_take_mutip_elems();

        //         let docs = self.end_stmt_parse(true);

        //         TopLevel::MultiTakeStmt {
        //             take,
        //             from: ModPath::Ident(lib_or_mod_ident.clone()),
        //             info: StmtInfo {
        //                 span: Span {
        //                     st: stm_st,
        //                     en: lib_or_mod_ident.span.en,
        //                     file: self.file.clone(),
        //                 },
        //                 docs,
        //             },
        //         }
        //     }
        //     Some(FullTok {
        //         kind: Token::MacroCall(_),
        //         ..
        //     }) => {
        //         // mú mod/#macrocall
        //         let take = self.get_take_elem();

        //         let docs = self.end_stmt_parse(true);
        //         TopLevel::TakeStmt {
        //             take: take,
        //             from: ModPath::Ident(lib_or_mod_ident.clone()),
        //             lib: None,
        //             info: StmtInfo {
        //                 span: Span {
        //                     st: stm_st,
        //                     en: lib_or_mod_ident.span.en,
        //                     file: self.file.clone(),
        //                 },
        //                 docs,
        //             },
        //         }
        //     }
        //     Some(FullTok {
        //         kind: Token::StringLiteral(s),
        //         span,
        //     }) => {
        //         // mú mod/"path/mod.ope"/#macrocall
        //         self.advance();

        //         let from_str = Ident { ident: s, span };

        //         self.expect(&Token::DColon);

        //         if self.peek_kind() == Some(Token::LBrace) {
        //             // mú mod/"path/mod.ope"/{...}
        //             self.advance();
        //             let takes = self.get_take_mutip_elems();

        //             let docs = self.end_stmt_parse(true);

        //             TopLevel::MultiTakeStmt {
        //                 info: StmtInfo {
        //                     span: Span {
        //                         st: stm_st,
        //                         en: lib_or_mod_ident.span.en,
        //                         file: self.file.clone(),
        //                     },
        //                     docs,
        //                 },
        //                 take: takes,
        //                 from: ModPath::Str(from_str),
        //                 lib: Some(lib_or_mod_ident),
        //             }
        //         } else {
        //             // mú mod\"path/mod.ope"\#macrocall
        //             let take = self.get_take_elem();

        //             let docs = self.end_stmt_parse(true);

        //             TopLevel::TakeStmt {
        //                 info: StmtInfo {
        //                     span: Span {
        //                         st: stm_st,
        //                         en: lib_or_mod_ident.span.en,
        //                         file: self.file.clone(),
        //                     },
        //                     docs,
        //                 },
        //                 take: take,
        //                 from: ModPath::Str(from_str),
        //                 lib: Some(lib_or_mod_ident),
        //             }
        //         }
        //     }
        //     Some(FullTok {
        //         kind: Token::Identifier(ident1),
        //         span: span1,
        //     }) => {
        //         self.advance();
        //         // mú mod/ident1
        //         match self.peek_kind() {
        //             Some(Token::DColon) => {
        //                 // still in business
        //                 self.advance();
        //                 // mú lib/ident1/

        //                 match self.peek() {
        //                     Some(FullTok {
        //                         kind: Token::LBrace,
        //                         ..
        //                     }) => {
        //                         self.advance();
        //                         // mú lib/ident1/{...}
        //                         let take = self.get_take_mutip_elems();

        //                         let docs = self.end_stmt_parse(true);

        //                         TopLevel::MultiTakeStmt {
        //                             info: StmtInfo {
        //                                 span: Span {
        //                                     st: stm_st,
        //                                     en: lib_or_mod_ident.span.en,
        //                                     file: self.file.clone(),
        //                                 },
        //                                 docs,
        //                             },
        //                             take,
        //                             from: ModPath::Ident(Ident {
        //                                 ident: ident1,
        //                                 span: span1,
        //                             }),
        //                             lib: Some(lib_or_mod_ident),
        //                         }
        //                     }
        //                     Some(FullTok {
        //                         kind: Token::Identifier(ident2),
        //                         span: span2,
        //                     }) => {
        //                         self.advance();
        //                         // mú lib/ident1/ident2
        //                         if self.peek_kind() == Some(Token::DColon) {
        //                             self.advance();
        //                             // mú lib/ident1/ident2/
        //                             let lib_folder = Ident {
        //                                 ident: ident1,
        //                                 span: span1,
        //                             };
        //                             let lib_mod = Ident {
        //                                 ident: ident2,
        //                                 span: span2,
        //                             };

        //                             if self.peek_kind() == Some(Token::LBrace) {
        //                                 self.advance();
        //                                 // mú lib/ident1/ident2/{...}
        //                                 let take = self.get_take_mutip_elems();

        //                                 let docs = self.end_stmt_parse(true);

        //                                 TopLevel::MultiTakeStmt {
        //                                     info: StmtInfo {
        //                                         span: Span {
        //                                             st: stm_st,
        //                                             en: lib_or_mod_ident.span.en,
        //                                             file: self.file.clone(),
        //                                         },
        //                                         docs,
        //                                     },
        //                                     take,
        //                                     from: ModPath::FromLibFolder(lib_folder, lib_mod),
        //                                     lib: Some(lib_or_mod_ident),
        //                                 }
        //                             } else {
        //                                 // mú lib/ident1/ident2/#macrocall
        //                                 let take = self.get_take_elem();

        //                                 let docs = self.end_stmt_parse(true);
        //                                 TopLevel::TakeStmt {
        //                                     info: StmtInfo {
        //                                         span: Span {
        //                                             st: stm_st,
        //                                             en: lib_or_mod_ident.span.en,
        //                                             file: self.file.clone(),
        //                                         },
        //                                         docs,
        //                                     },
        //                                     take: take,
        //                                     from: ModPath::FromLibFolder(lib_folder, lib_mod),
        //                                     lib: Some(lib_or_mod_ident),
        //                                 }
        //                             }
        //                         } else {
        //                             self.reverse();
        //                             // mú lib/ident1/ident2
        //                             //               ^^^^^^
        //                             let take = self.get_take_elem();

        //                             let docs = self.end_stmt_parse(false);

        //                             TopLevel::TakeStmt {
        //                                 info: StmtInfo {
        //                                     span: Span {
        //                                         st: stm_st,
        //                                         en: lib_or_mod_ident.span.en,
        //                                         file: self.file.clone(),
        //                                     },
        //                                     docs,
        //                                 },
        //                                 take: take,
        //                                 from: ModPath::Ident(Ident {
        //                                     ident: ident1,
        //                                     span: span1,
        //                                 }),
        //                                 lib: Some(lib_or_mod_ident),
        //                             }
        //                         }
        //                     }
        //                     Some(FullTok {
        //                         kind: Token::MacroCall(_),
        //                         ..
        //                     }) => {
        //                         // mú lib/ident1/#macr
        //                         let take = self.get_take_elem();

        //                         let docs = self.end_stmt_parse(false);

        //                         TopLevel::TakeStmt {
        //                             info: StmtInfo {
        //                                 span: Span {
        //                                     st: stm_st,
        //                                     en: lib_or_mod_ident.span.en,
        //                                     file: self.file.clone(),
        //                                 },
        //                                 docs,
        //                             },
        //                             take: take,
        //                             from: ModPath::Ident(Ident {
        //                                 ident: ident1,
        //                                 span: span1,
        //                             }),
        //                             lib: Some(lib_or_mod_ident),
        //                         }
        //                     }
        //                     Some(oth) => {
        //                         self.errs.push(Error {
        //                             message: format!("Unexpected importation argument"),
        //                             span: oth.span,
        //                         });

        //                         TopLevel::Foo
        //                     }
        //                     None => {
        //                         self.errs.push(Error {
        //                             message: format!("Kò tíì yẹ kí fáílì yìí ó dópin"),
        //                             span: Span {
        //                                 st: Pos { column: 0, line: 0 },
        //                                 en: Pos { column: 0, line: 0 },
        //                                 file: self.file.clone(),
        //                             },
        //                         });

        //                         TopLevel::Foo
        //                     }
        //                 }
        //             }
        //             _ => {
        //                 self.reverse();
        //                 // mú mod/ident1
        //                 //        ^^^^^^
        //                 let take = self.get_take_elem();

        //                 let docs = self.end_stmt_parse(false);

        //                 TopLevel::TakeStmt {
        //                     take,
        //                     from: ModPath::Ident(lib_or_mod_ident.clone()),
        //                     lib: None,
        //                     info: StmtInfo {
        //                         span: Span {
        //                             st: stm_st,
        //                             en: lib_or_mod_ident.span.en,
        //                             file: self.file.clone(),
        //                         },
        //                         docs,
        //                     },
        //                 }
        //             }
        //         }
        //     }
        //     Some(oth) => {
        //         self.errs.push(Error {
        //             message: format!("Unexpected importation argument"),
        //             span: oth.span,
        //         });

        //         TopLevel::Foo
        //     }
        //     None => {
        //         self.errs.push(Error {
        //             message: format!("Kò tíì yẹ kí fáílì yìí ó dópin"),
        //             span: Span {
        //                 st: Pos { column: 0, line: 0 },
        //                 en: Pos { column: 0, line: 0 },
        //                 file: self.file.clone(),
        //             },
        //         });

        //         TopLevel::Foo
        //     }
        // }
    }

    fn precedence(&self, op: &FullTok) -> u8 {
        match &op.kind {
            &Token::Plus | &Token::Minus => 13,
            &Token::Star | &Token::Mul | &Token::BackSlash | &Token::Div => 14,
            &Token::Pipe => 15,
            &Token::NotEqual
            | &Token::Equal
            | &Token::Greater
            | &Token::GreaterEq
            | &Token::Less
            | &Token::LessEq => 16,
            // &Token::LParen | &Token::LBrace => 17,
            &Token::DColon => 18,
            &Token::Dot | Token::DDot => 19,
            // &Token::BackwardArrow => 20,
            // &Token::ForwardArrow => 21,
            &Token::As => 22,
            _ => 0,
        }
    }

    fn parse_args(&mut self, end: Token) -> Vec<Expr> {
        // ( is alrealdy consumed
        let mut args = vec![];

        while self.peek_kind() != Some(end.clone()) {
            args.push(self.parse_expr(0, false));
            if self.peek_kind() == Some(Token::Comma) {
                self.advance();
            } else if self.peek_kind() == Some(end.clone()) {
                break;
            } else {
                let kind = self.peek_kind();
                self.errs.push(Error {
                    message: format!("'{:?}' kọ́ ni oó yẹ kí oó wà níbí. Lo ',' dípò", kind),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });
            }
        }
        self.advance();
        args
    }

    fn parse_field_assignms(&mut self) -> Vec<FieldAssignm> {
        let mut assigns = vec![];

        while self.peek_kind() != Some(Token::RBrace) {
            let ident = self.expect_ident();
            match self.peek_kind() {
                Some(Token::Colon) => {
                    self.advance();
                    // then it's labeled

                    let expr = self.parse_expr(0, false);

                    assigns.push(FieldAssignm { ident, expr });

                    if self.peek_kind() == Some(Token::Comma) {
                        self.advance();
                    }
                }
                _ => {
                    assigns.push(FieldAssignm {
                        ident: ident.clone(),
                        expr: Expr {
                            kind: ExprKind::Ident {
                                affix: None,
                                ident: ident.clone(),
                            },
                            span: ident.span,
                        },
                    });

                    if self.peek_kind() == Some(Token::Comma) {
                        self.advance();
                    }
                }
            }
        }

        self.expect(&Token::RBrace);
        assigns
    }

    // fn parse_macro_expr(&mut self, name: Ident, expr_st: Pos) -> Expr {
    //     let mut tokens = vec![];
    //     let delimiter;
    //     let mut _expr_en;

    //     match self.peek_kind() {
    //         Some(Token::LParen) => {
    //             delimiter = Delimiter::Paren;

    //             self.advance();
    //             while let Some(tok) = self.peek() {
    //                 match tok.kind {
    //                     Token::RParen => {
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                         break;
    //                     }
    //                     Token::LParen => {
    //                         // if it's another {
    //                         while let Some(in_group) = self.peek() {
    //                             tokens.push(in_group.clone());
    //                             _expr_en = self.peek_span().en.clone();
    //                             self.advance();
    //                             if in_group.kind == Token::RParen {
    //                                 tokens.push(in_group.clone());
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                     _ => {
    //                         tokens.push(tok);
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                     }
    //                 }
    //             }

    //             _expr_en = self.peek_span().en.clone();
    //         }
    //         Some(Token::LBracket) => {
    //             delimiter = Delimiter::Bracket;
    //             self.advance();
    //             while let Some(tok) = self.peek() {
    //                 match tok.kind {
    //                     Token::RBracket => {
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                         break;
    //                     }
    //                     Token::LBracket => {
    //                         // if it's another {
    //                         while let Some(in_group) = self.peek() {
    //                             tokens.push(in_group.clone());
    //                             _expr_en = self.peek_span().en.clone();
    //                             self.advance();
    //                             if in_group.kind == Token::RBracket {
    //                                 tokens.push(in_group.clone());
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                     _ => {
    //                         tokens.push(tok);
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                     }
    //                 }
    //             }
    //             _expr_en = self.peek_span().en.clone();
    //         }
    //         Some(Token::LBrace) => {
    //             delimiter = Delimiter::Brace;
    //             self.advance();
    //             while let Some(tok) = self.peek() {
    //                 match tok.kind {
    //                     Token::RBrace => {
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                         break;
    //                     }
    //                     Token::LBrace => {
    //                         // if it's another {
    //                         while let Some(in_group) = self.peek() {
    //                             tokens.push(in_group.clone());
    //                             _expr_en = self.peek_span().en.clone();
    //                             self.advance();
    //                             if in_group.kind == Token::RBrace {
    //                                 tokens.push(in_group.clone());
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                     _ => {
    //                         tokens.push(tok);
    //                         _expr_en = self.peek_span().en.clone();
    //                         self.advance();
    //                     }
    //                 }
    //             }
    //             _expr_en = self.peek_span().en.clone();
    //         }
    //         _ => {
    //             _expr_en = self.peek_span().en.clone();

    //             self.errs.push(Error {
    //                 message: format!("Delimiter gbodọ̀ tẹ̀lé e"),
    //                 span: Span {
    //                     st: Pos {
    //                         column: self.peek_span().st.column,
    //                         line: self.peek_span().st.line,
    //                     },
    //                     en: Pos {
    //                         column: self.peek_span().en.column,
    //                         line: self.peek_span().en.line,
    //                     },
    //                     file: self.file.clone(),
    //                 },
    //             });
    //             delimiter = Delimiter::Paren;
    //         }
    //     }

    //     Expr {
    //         kind: ExprKind::Macro {
    //             name,
    //             delimiter,
    //             tokens,
    //         },
    //         span: Span {
    //             st: expr_st,
    //             en: _expr_en,
    //             file: self.file.clone(),
    //         },
    //     }
    // }

    fn after_ident_expr(&mut self, ident: &Ident) -> Expr {
        match self.peek() {
            Some(FullTok {
                kind: Token::Less,
                span: _,
            }) => {
                self.advance();
                // Might be a function call with generics

                self.advance(); // advance temporarily, to check for the next token
                                // ident<Type ( now if , or < or > follows, that means its a function)
                match self.peek_kind() {
                    Some(Token::Comma) | Some(Token::Less) | Some(Token::Greater) => {
                        self.reverse(); // go back to the right position

                        let expr = self.parse_generical_call(ident.clone());

                        expr
                    }
                    _ => {
                        //if it's not a function call generics
                        self.reverse(); // we still need to go back to the right postion
                        Expr {
                            span: ident.span.clone(),
                            kind: ExprKind::Ident {
                                affix: None,
                                ident: ident.clone(),
                            },
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::LParen,
                span: _,
            }) => {
                self.advance();
                // a function call
                let args = self.parse_args(Token::RParen);

                Expr {
                    kind: ExprKind::ArgBuffer {
                        asy: None,
                        generics: vec![],
                        args,
                        name: ident.clone(),
                    },
                    span: ident.span.clone(),
                }
            }
            Some(FullTok {
                kind: Token::LBrace,
                span: _,
            }) => {
                let name = Some(ident.clone());
                let st = self.peek_span().st.clone();
                let assigns = self.parse_field_assignms();
                let en = self.peek_span().en.clone();

                Expr {
                    kind: ExprKind::StructVal {
                        vals: assigns,
                        ident: name,
                    },
                    span: Span {
                        st,
                        en,
                        file: self.file.clone(),
                    },
                }
            }
            // Some(FullTok {
            //     kind: Token::Slash,
            //     span: _,
            // }) => {
            //     self.advance();
            //     // on spot importation
            //     // ident = left
            //     // now the pattern is = ident/
            //     self.advance(); // in advance
            //     match self.peek_kind() {
            //         Some(Token::Slash) => {
            //             // then it is from a ident
            //             // mú ident/mod_ident/
            //             self.reverse();
            //             let mod_path;

            //             match self.peek() {
            //                 Some(FullTok {
            //                     kind: Token::Identifier(_),
            //                     span: _,
            //                 }) => {
            //                     mod_path = ModPath::Ident(self.expect_ident());
            //                 }
            //                 Some(FullTok {
            //                     kind: Token::StringLiteral(strlit),
            //                     span,
            //                 }) => {
            //                     mod_path = ModPath::Str(Ident {
            //                         ident: strlit,
            //                         span,
            //                     })
            //                 }
            //                 Some(oths) => {
            //                     mod_path = ModPath::UnExpected;
            //                     self.errs.push(Error {
            //                         message: format!(
            //                             "ident tàbí str ni hó ńretí dípò {:?}",
            //                             oths.kind
            //                         ),
            //                         span: Span {
            //                             st: Pos {
            //                                 column: oths.span.st.column,
            //                                 line: oths.span.st.line,
            //                             },
            //                             en: Pos {
            //                                 column: oths.span.en.column,
            //                                 line: oths.span.en.line,
            //                             },
            //                             file: self.file.clone(),
            //                         },
            //                     });
            //                 }
            //                 None => {
            //                     mod_path = ModPath::UnExpected;
            //                     self.errs.push(Error {
            //                         message: String::from("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
            //                         span: Span {
            //                             st: Pos { column: 0, line: 0 },
            //                             en: Pos { column: 0, line: 0 },
            //                             file: self.file.clone(),
            //                         },
            //                     });
            //                 }
            //             }

            //             self.advance(); // its currently on \ token
            //             let take = self.nud(false);

            //             Expr {
            //                 kind: ExprKind::Import {
            //                     imported: Box::new(take.clone()),
            //                     from: mod_path,
            //                     lib: Some(ident.clone()),
            //                 },
            //                 span: Span {
            //                     st: ident.span.st,
            //                     en: take.span.en,
            //                     file: self.file.clone(),
            //                 },
            //             }
            //         }
            //         _ => {
            //             self.reverse();
            //             // mú ident\import
            //             let imported = Box::new(self.nud(false));

            //             Expr {
            //                 span: Span {
            //                     st: ident.span.st,
            //                     en: imported.as_ref().span.en,
            //                     file: self.file.clone(),
            //                 },
            //                 kind: ExprKind::Import {
            //                     imported,
            //                     from: ModPath::Ident(ident.clone()),
            //                     lib: None,
            //                 },
            //             }
            //         }
            //     }
            // }
            Some(FullTok {
                kind: Token::DColon,
                span: _,
            }) => {
                let mut access = vec![];

                while self.peek_kind() == Some(Token::DColon) {
                    self.advance();

                    let new_left = self.nud(false);
                    match new_left.kind.clone() {
                        ExprKind::Ident {
                            affix: af,
                            ident: _,
                        } => {
                            if af.is_none() {
                                // it's either a module, function, enum variant or a global variable
                                match self.peek() {
                                    Some(FullTok {
                                        kind: Token::Less,
                                        span,
                                    }) => {
                                        // then it's a function
                                        self.advance();
                                        let with_genr = self.led(
                                            new_left,
                                            FullTok {
                                                kind: Token::Less,
                                                span,
                                            },
                                            0,
                                            false,
                                        );

                                        match self.peek() {
                                            Some(FullTok {
                                                kind: Token::LParen,
                                                span,
                                            }) => {
                                                self.advance();
                                                let call = self.led(
                                                    with_genr,
                                                    FullTok {
                                                        kind: Token::LParen,
                                                        span,
                                                    },
                                                    0,
                                                    false,
                                                );

                                                access.push(call);
                                            }
                                            _ => {
                                                self.expect(&Token::LParen);

                                                access.push(Expr {
                                                    kind: ExprKind::Foo,
                                                    span: self.zero_span.clone(),
                                                });
                                            }
                                        }

                                        break;
                                    }
                                    Some(FullTok {
                                        kind: Token::LParen,
                                        span,
                                    }) => {
                                        self.advance();
                                        let buffer = self.led(
                                            new_left,
                                            FullTok {
                                                kind: Token::LParen,
                                                span,
                                            },
                                            0,
                                            false,
                                        );

                                        access.push(buffer);

                                        break;
                                    }
                                    _ => {
                                        access.push(new_left);
                                        // doesn't have to break, because it might be a module or an enum
                                    }
                                }
                            } else {
                                return new_left;
                            }
                        }
                        // ExprKind::Macro {
                        //     name,
                        //     delimiter,
                        //     tokens,
                        // } => {
                        //     let call = Expr {
                        //         kind: ExprKind::Macro {
                        //             name: name.clone(),
                        //             delimiter,
                        //             tokens,
                        //         },
                        //         span: name.span,
                        //     };

                        //     access.push(call);
                        // }
                        ExprKind::ArgBuffer {
                            asy: _,
                            generics: _,
                            args: _,
                            name: _,
                        } => {
                            access.push(new_left);
                        }
                        // Expr::Block(b) => { // mainly for macro expansion
                        //     let call = Box::new(Expr::Block(b));
                        //     Ok(Expr::FromStatic { static_ident, call })
                        // }
                        _ => todo!("Expression not support for Namespace access"),
                    }
                }

                Expr {
                    kind: ExprKind::NamespaceAccess {
                        ident: Some(ident.clone()),
                        access,
                    },
                    span: ident.span.clone(),
                }
            }
            _ => Expr {
                span: ident.span.clone(),
                kind: ExprKind::Ident {
                    affix: None,
                    ident: ident.clone(),
                },
            },
        }
    }

    fn nud(&mut self, _is_pattern: bool) -> Expr {
        const PREFIX_PREC: u8 = 15;
        // let expr_st = self.peek_span().st.clone();
        match self.advance() {
            Some(FullTok {
                kind: Token::Number(n),
                span,
            }) => Expr {
                kind: ExprKind::Number(n),
                span,
            },
            Some(FullTok {
                kind: Token::SelfKw,
                span,
            }) => Expr {
                kind: ExprKind::SelfVal,
                span,
            },
            Some(FullTok {
                kind: Token::ArityLiteral(b),
                span,
            }) => Expr {
                kind: ExprKind::TruthVal(b),
                span,
            },
            Some(FullTok {
                kind: Token::StringLiteral(s),
                span,
            }) => Expr {
                kind: ExprKind::Str(s),
                span,
            },
            Some(FullTok {
                kind: Token::Assign,
                span: _,
            }) => {
                let to_assign = self.parse_mutable_ident();

                to_assign
            }
            Some(FullTok {
                kind: Token::Move,
                span,
            }) => {
                let ident = self.expect_ident();

                Expr {
                    span: Span {
                        st: span.st,
                        en: ident.span.en,
                        file: self.file.clone(),
                    },
                    kind: ExprKind::Ident {
                        affix: Some(IdentAffix::Move(span)),
                        ident: ident.clone(),
                    },
                }
            }
            Some(FullTok {
                kind: Token::Await,
                span,
            }) => {
                let name = self.expect_ident();

                match self.peek_kind() {
                    Some(Token::Less) => {
                        self.advance();
                        let call = self.parse_generical_call(name.clone());

                        match call.kind.clone() {
                            ExprKind::ArgBuffer {
                                asy: _,
                                generics,
                                args,
                                name,
                            } => Expr {
                                kind: ExprKind::ArgBuffer {
                                    asy: Some(Asy::Await(span)),
                                    generics,
                                    args,
                                    name,
                                },
                                span: call.span,
                            },
                            _ => Expr {
                                kind: ExprKind::Foo,
                                span: self.zero_span.clone(),
                            },
                        }
                    }
                    Some(Token::LParen) => {
                        self.advance();
                        let args = self.parse_args(Token::RParen);

                        Expr {
                            span: Span {
                                st: span.st,
                                en: name.span.en,
                                file: self.file.clone(),
                            },
                            kind: ExprKind::ArgBuffer {
                                asy: Some(Asy::Await(span)),
                                generics: vec![],
                                args,
                                name: name.clone(),
                            },
                        }
                    }
                    _ => {
                        self.errs.push(Error {
                            message: format!("iṣé nìkan ni ètò lè dúró de"),
                            span: span,
                        });

                        Expr {
                            kind: ExprKind::Foo,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::Start,
                span,
            }) => {
                let name = self.expect_ident();

                match self.peek_kind() {
                    Some(Token::Less) => {
                        self.advance();
                        let call = self.parse_generical_call(name.clone());

                        match call.kind.clone() {
                            ExprKind::ArgBuffer {
                                asy: _,
                                generics,
                                args,
                                name,
                            } => Expr {
                                kind: ExprKind::ArgBuffer {
                                    asy: Some(Asy::Start(span)),
                                    generics,
                                    args,
                                    name,
                                },
                                span: call.span,
                            },
                            _ => Expr {
                                kind: ExprKind::Foo,
                                span: self.zero_span.clone(),
                            },
                        }
                    }
                    Some(Token::LParen) => {
                        self.advance();
                        let args = self.parse_args(Token::RParen);

                        Expr {
                            span: Span {
                                st: span.st,
                                en: name.span.en,
                                file: self.file.clone(),
                            },
                            kind: ExprKind::ArgBuffer {
                                asy: Some(Asy::Start(span)),
                                generics: vec![],
                                args,
                                name: name.clone(),
                            },
                        }
                    }
                    _ => {
                        self.errs.push(Error {
                            message: format!("iṣé nìkan ni o lè bẹ̀rẹ̀ idúró fún"),
                            span: span,
                        });

                        Expr {
                            kind: ExprKind::Foo,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::Identifier(ident),
                span,
            }) => {
                let ident = Ident { ident, span };
                self.after_ident_expr(&ident)
            }
            Some(FullTok {
                kind: Token::If,
                span: _,
            }) => {
                self.reverse();
                let smt = self.parse_if();

                match smt {
                    BlockLevel::IfStmt {
                        bool_expr,
                        exe,
                        else_ifs,
                        else_exe,
                        info,
                    } => Expr {
                        kind: ExprKind::If {
                            bool_expr: Box::new(bool_expr),
                            exe: Box::new(exe),
                            else_ifs,
                            else_exe: Box::new(else_exe),
                        },
                        span: info.span,
                    },
                    _ => Expr {
                        kind: ExprKind::Foo,
                        span: self.zero_span.clone(),
                    },
                }
            }
            Some(FullTok {
                kind: Token::Match,
                span: _,
            }) => {
                self.reverse();
                let smt = self.parse_match();
                match smt {
                    BlockLevel::MatchStmt { var, arms, info } => Expr {
                        kind: ExprKind::Match { var, arms },
                        span: info.span,
                    },
                    _ => Expr {
                        kind: ExprKind::Foo,
                        span: self.zero_span.clone(),
                    },
                }
            }
            Some(FullTok {
                kind: Token::Minus,
                span,
            }) => {
                let rhs = self.parse_expr(PREFIX_PREC, false);
                Expr {
                    span: Span {
                        st: span.st,
                        en: rhs.span.en,
                        file: self.file.clone(),
                    },
                    kind: ExprKind::Unary {
                        op: FullTok {
                            kind: Token::Minus,
                            span,
                        },
                        rhs: Box::new(rhs.clone()),
                    },
                }
            }
            Some(FullTok {
                kind: Token::At,
                span: at_span,
            }) => {
                match self.advance() {
                    Some(FullTok {
                        kind: Token::Identifier(i),
                        span,
                    }) => {
                        // Only an variable can have a memory address
                        Expr {
                            span: Span {
                                st: at_span.st,
                                en: span.en,
                                file: self.file.clone(),
                            },
                            kind: ExprKind::Address(
                                MutStatus::Const,
                                Box::new(Expr {
                                    kind: ExprKind::Ident {
                                        affix: None,
                                        ident: Ident {
                                            ident: i,
                                            span: span.clone(),
                                        },
                                    },
                                    span,
                                }),
                            ),
                        }
                    } // kí *tóyí Okùn àpẹrẹ tóyí: @yí orísùu
                    // orísùu most be mutable since its assigned to a mut ptr
                    Some(FullTok {
                        kind: Token::Assign,
                        span,
                    }) => match self.advance() {
                        Some(FullTok {
                            kind: Token::Identifier(i),
                            span: ident_span,
                        }) => Expr {
                            kind: ExprKind::Address(
                                MutStatus::Mut(span.clone()),
                                Box::new(Expr {
                                    kind: ExprKind::Ident {
                                        affix: None,
                                        ident: Ident {
                                            ident: i,
                                            span: ident_span.clone(),
                                        },
                                    },
                                    span,
                                }),
                            ),
                            span: Span {
                                st: at_span.st,
                                en: ident_span.en,
                                file: self.file.clone(),
                            },
                        },
                        _ => {
                            self.errs.push(Error {
                                message: format!("Identifier ni òǹṣàkópọ̀ ń retí níbí yìí"),
                                span: Span {
                                    st: Pos {
                                        column: self.peek_span().st.column,
                                        line: self.peek_span().st.line,
                                    },
                                    en: Pos {
                                        column: self.peek_span().en.column,
                                        line: self.peek_span().en.line,
                                    },
                                    file: self.file.clone(),
                                },
                            });

                            Expr {
                                kind: ExprKind::Foo,
                                span: self.zero_span.clone(),
                            }
                        }
                    },
                    _ => {
                        self.errs.push(Error {
                            message: format!("Identifier tàbí ni òǹṣàkópọ̀ ń retí níbí yìí"),
                            span: Span {
                                st: Pos {
                                    column: self.peek_span().st.column,
                                    line: self.peek_span().st.line,
                                },
                                en: Pos {
                                    column: self.peek_span().en.column,
                                    line: self.peek_span().en.line,
                                },
                                file: self.file.clone(),
                            },
                        });

                        Expr {
                            kind: ExprKind::Foo,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::Exclam,
                span,
            }) => {
                let rhs = self.parse_expr(PREFIX_PREC, false);

                Expr {
                    span: Span {
                        st: span.st,
                        en: rhs.span.en,
                        file: self.file.clone(),
                    },
                    kind: ExprKind::Unary {
                        op: FullTok {
                            kind: Token::Exclam,
                            span,
                        },
                        rhs: Box::new(rhs.clone()),
                    },
                }
            }
            // Some(FullTok {
            //     kind: Token::MacroCall(n),
            //     span,
            // }) => self.parse_macro_expr(Ident { ident: n, span }, expr_st),
            Some(FullTok {
                kind: Token::LParen,
                span: _,
            }) => {
                let expr = self.parse_expr(0, false);
                self.expect(&Token::RParen);
                expr
            }
            Some(FullTok {
                kind: Token::LBracket,
                span,
            }) => {
                let mut exprs: Vec<Expr> = Vec::new();
                loop {
                    exprs.push(self.parse_expr(0, false));
                    if self.peek_kind() == Some(Token::Comma) {
                        self.advance();
                    } else if self.peek_kind() == Some(Token::RBracket) {
                        break;
                    } else {
                        let kind = self.peek_kind();

                        self.errs.push(Error {
                            message: format!("'{:?}' kọ́ ni oó yẹ kí oó wà níbí", kind),
                            span: Span {
                                st: Pos {
                                    column: self.peek_span().st.column,
                                    line: self.peek_span().st.line,
                                },
                                en: Pos {
                                    column: self.peek_span().en.column,
                                    line: self.peek_span().en.line,
                                },
                                file: self.file.clone(),
                            },
                        });
                    }
                }

                self.advance();
                Expr {
                    kind: ExprKind::List(exprs),
                    span,
                }
            }
            Some(FullTok {
                kind: Token::Dot,
                span: dot_span,
            }) => {
                // it remains Token::Dot, well it does follow an expression
                match self.peek() {
                    Some(FullTok {
                        kind: Token::LBrace,
                        span: _,
                    }) => {
                        // .{}
                        // let st = self.peek_span().st.clone();
                        self.advance();
                        let assigns = self.parse_field_assignms();
                        let en = self.peek_span().en.clone();

                        Expr {
                            kind: ExprKind::StructVal {
                                ident: None,
                                vals: assigns,
                            },
                            span: Span {
                                st: dot_span.st,
                                en,
                                file: self.file.clone(),
                            },
                        }
                    }
                    Some(FullTok {
                        kind: Token::Identifier(ident),
                        span,
                    }) => {
                        // .ExprORVariant()
                        self.advance();
                        let kind;

                        // let st = self.peek_span().st.clone();
                        match self.peek_kind() {
                            Some(Token::LParen) => {
                                // .Variant()
                                self.advance();
                                let buf_args = self.parse_args(Token::RParen);

                                kind = Expr {
                                    kind: ExprKind::ArgBuffer {
                                        asy: None,
                                        generics: vec![],
                                        args: buf_args,
                                        name: Ident {
                                            ident,
                                            span: span.clone(),
                                        },
                                    },
                                    span,
                                };
                            }
                            Some(Token::LBrace) => {
                                // .Variant {}
                                let st = self.peek_span().st.clone();
                                self.advance();
                                // this is a variant of struct form
                                let assigns = self.parse_field_assignms();
                                let en = self.peek_span().en.clone();

                                kind = Expr {
                                    kind: ExprKind::StructVal {
                                        ident: Some(Ident { ident, span }), // always some for enum variant of struct
                                        vals: assigns,
                                    },
                                    span: Span {
                                        st,
                                        en,
                                        file: self.file.clone(),
                                    },
                                };
                            }
                            _ => {
                                // .Variant
                                kind = Expr {
                                    kind: ExprKind::Ident {
                                        affix: None,
                                        ident: Ident {
                                            ident,
                                            span: span.clone(),
                                        },
                                    },
                                    span,
                                };
                            }
                        }
                        let en = self.peek_span().en.clone();

                        Expr {
                            kind: ExprKind::NamespaceAccess {
                                ident: None,
                                access: vec![kind],
                            },
                            span: Span {
                                st: dot_span.st,
                                en,
                                file: self.file.clone(),
                            },
                        } // name is infered
                    }
                    _ => {
                        self.errs.push(Error {
                            message: format!("Expected an Enum variant or Struct value"),
                            span: Span {
                                st: Pos {
                                    column: self.peek_span().st.column,
                                    line: self.peek_span().st.line,
                                },
                                en: Pos {
                                    column: self.peek_span().en.column,
                                    line: self.peek_span().en.line,
                                },
                                file: self.file.clone(),
                            },
                        });

                        Expr {
                            kind: ExprKind::Foo,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Some(FullTok {
                kind: Token::LBrace,
                span,
            }) => {
                let stms = self.parse_stmts();

                Expr {
                    kind: ExprKind::Block(stms.0),
                    span: Span {
                        st: span.st,
                        en: stms.1,
                        file: self.file.clone(),
                    },
                }
            }
            Some(oth) => {
                self.errs.push(Error {
                    message: format!("'{:?}' kọ́ ni oó yẹ kí oó wà níbí", oth.kind),
                    span: oth.span,
                });

                Expr {
                    kind: ExprKind::Foo,
                    span: self.zero_span.clone(),
                }
            }
            None => {
                self.errs.push(Error {
                    message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos {
                            column: self.peek_span().st.column,
                            line: self.peek_span().st.line,
                        },
                        en: Pos {
                            column: self.peek_span().en.column,
                            line: self.peek_span().en.line,
                        },
                        file: self.file.clone(),
                    },
                });

                Expr {
                    kind: ExprKind::Foo,
                    span: self.zero_span.clone(),
                }
            }
        }
    }

    fn parse_generical_call(&mut self, ident: Ident) -> Expr {
        let mut generics = Vec::new();

        while self.peek_kind() != Some(Token::Greater) {
            generics.push(self.expect_typespec());
            match self.peek_kind() {
                Some(Token::Comma) => {
                    continue;
                }
                Some(Token::Greater) => {
                    break;
                }
                _ => (),
            }
        }

        let nxt = self.peek();

        if nxt.is_some() {
            match nxt.clone().unwrap().kind {
                Token::DGreater => {
                    self.tokens[self.pos] = FullTok {
                        kind: Token::Greater,
                        span: nxt.unwrap().span,
                    };
                    // because of àrà<bóyá<ọsán>>(kó a)
                    //                         __ -> right there
                }
                _ => {
                    self.expect(&Token::Greater);
                }
            }
        } else {
            self.expect(&Token::Greater);
        }

        self.expect(&Token::LParen);
        let args = self.parse_args(Token::RParen);

        Expr {
            kind: ExprKind::ArgBuffer {
                asy: None,
                generics,
                args,
                name: ident.clone(),
            },
            span: ident.span,
        }
    }

    fn led(&mut self, left: Expr, op: FullTok, prec: u8, is_pattern: bool) -> Expr {
        // const POSTFIX_PREC: u8 = 16;
        match op.kind {
            // infix binary ops
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::BackSlash
            | Token::Mul
            | Token::Div
            | Token::Greater
            | Token::LessEq
            | Token::GreaterEq
            | Token::Equal
            | Token::NotEqual => {
                let rhs = self.parse_expr(prec + 1, false);
                Expr {
                    kind: ExprKind::Binary {
                        lhs: Box::new(left.clone()),
                        op,
                        rhs: Box::new(rhs.clone()),
                    },
                    span: Span {
                        st: left.span.st,
                        en: rhs.span.en,
                        file: self.file.clone(),
                    },
                }
            }
            Token::Less => {
                let rhs = self.parse_expr(prec + 1, false);

                Expr {
                    kind: ExprKind::Binary {
                        lhs: Box::new(left.clone()),
                        op,
                        rhs: Box::new(rhs.clone()),
                    },
                    span: Span {
                        st: left.span.st,
                        en: rhs.span.en,
                        file: self.file.clone(),
                    },
                }
            }
            Token::Dot => {
                match self.peek_kind() {
                    Some(Token::LBrace) => {
                        // because .{} denotes a struct with an infered name
                        // . in .{} is not a led
                        self.reverse(); // back to Token::Dot
                        self.reverse(); // because we need to end it properly with Token::NewLine
                        let nxt = self.peek_kind();
                        if nxt != Some(Token::RBrace) {
                            self.expect_newline();
                        }

                        // after this; the compiler will parse Token::Dot as a nud first
                        let st = self.peek_span().st.clone();
                        let _ = self.parse_expr(0, false);
                        let en = self.peek_span().en.clone();

                        self.errs.push(Error {
                            message: format!("':' sí wájú iyì àjọ yìí"),
                            span: Span {
                                st,
                                en,
                                file: self.file.clone(),
                            },
                        });

                        left
                    }
                    _ => {
                        let obj = Box::new(left);
                        let mut fields = vec![];
                        loop {
                            let f = self.nud(is_pattern);

                            match f.kind {
                                ExprKind::Ident {
                                    affix: None,
                                    ident: i,
                                } => match self.peek_kind() {
                                    Some(Token::LParen) => {
                                        self.advance();
                                        let args = self.parse_args(Token::RParen);

                                        fields.push(Expr {
                                            kind: ExprKind::ArgBuffer {
                                                asy: None,
                                                generics: vec![],
                                                args,
                                                name: i.clone(),
                                            },
                                            span: i.span,
                                        });
                                    }
                                    _ => {
                                        fields.push(Expr {
                                            kind: ExprKind::Ident {
                                                affix: None,
                                                ident: i.clone(),
                                            },
                                            span: i.span,
                                        });
                                    }
                                },
                                _ => {
                                    fields.push(f);
                                }
                            }

                            if self.peek_kind() != Some(Token::Dot) {
                                break;
                            }
                            self.advance();
                            // match self.peek_kind() {
                            //     Some(Token::LBrace) => {
                            //         // because .{} denotes a struct with an infered name
                            //         // . in .{} is not a led
                            //         self.reverse(); // back to Token::Dot
                            //         return Ok(Expr::Assess { obj, fields });
                            //         // after this; the compiler will try to parse Token::Dot as a nud which will break
                            //     }
                            //     _ => () // continues to parse
                            // }
                        }

                        Expr {
                            kind: ExprKind::ObjAssess {
                                obj: obj.clone(),
                                fields: fields.clone(),
                            },
                            span: Span {
                                st: obj.as_ref().span.st,
                                en: fields[fields.len() - 1].span.en,
                                file: self.file.clone(),
                            },
                        }
                    }
                }
            }
            Token::DDot => {
                let rhs = self.parse_expr(prec + 1, false);
                match &left {
                    Expr {
                        kind: ExprKind::Number(n1),
                        span: s1,
                    } => match &rhs {
                        Expr {
                            kind: ExprKind::Number(n2),
                            span: s2,
                        } => Expr {
                            kind: ExprKind::Binary {
                                lhs: Box::new(Expr {
                                    kind: ExprKind::Number(n1.clone()),
                                    span: s1.clone(),
                                }),
                                op,
                                rhs: Box::new(Expr {
                                    kind: ExprKind::Number(n2.clone()),
                                    span: s2.clone(),
                                }),
                            },
                            span: Span {
                                st: left.span.st,
                                en: rhs.span.en,
                                file: self.file.clone(),
                            },
                        },
                        _ => {
                            self.errs.push(Error{
                                    message: format!("òǹkà ni oó yẹ kí oó wà lápá òsì àti ọ̀tún '..' , dípò '{:?}' tí oó wà lápá ọ̀tún", rhs),
                                    span: Span {
                                        st: self.peek_span().st.clone(),
                                        en: self.peek_span().en.clone(),
                                        file: self.file.clone()
                                    }
                                });

                            Expr {
                                kind: ExprKind::Foo,
                                span: self.zero_span.clone(),
                            }
                        }
                    },
                    _ => {
                        self.errs.push(Error{
                                message: format!("òǹkà ni oó yẹ kí oó wà lápá òsì àti ọ̀tún '..' , dípò '{:?}' tí oó wà lápá òsì", left),
                                span: Span {
                                    st: self.peek_span().st.clone(),
                                    en: self.peek_span().en.clone(),
                                    file: self.file.clone()
                                }
                            });

                        Expr {
                            kind: ExprKind::Foo,
                            span: self.zero_span.clone(),
                        }
                    }
                }
            }
            Token::As => {
                let ty = Box::new(self.expect_typespec());

                Expr {
                    span: Span {
                        st: left.span.st,
                        en: ty.span.en,
                        file: self.file.clone(),
                    },
                    kind: ExprKind::AsType {
                        expr: Box::new(left),
                        ty,
                    },
                }
            }
            _ => {
                self.errs.push(Error {
                    message: format!("'{:?}' kọ́ ni oó yẹ kí oó wà níbí", op.kind),
                    span: Span {
                        st: self.peek_span().st.clone(),
                        en: self.peek_span().en.clone(),
                        file: self.file.clone(),
                    },
                });

                Expr {
                    kind: ExprKind::Foo,
                    span: self.zero_span.clone(),
                }
            } // This shouldn't be reached anyway
        }
    }

    fn parse_mutable_ident(&mut self) -> Expr {
        let var = self.expect_ident();
        let mut fields = vec![];
        while self.peek_kind() == Some(Token::Dot) {
            self.advance();

            fields.push(self.expect_ident());
        }

        if fields.len() == 0 {
            Expr {
                kind: ExprKind::MutableIdent {
                    var: var.clone(),
                    fields,
                },
                span: var.span,
            }
        } else {
            Expr {
                kind: ExprKind::MutableIdent {
                    var: var.clone(),
                    fields: fields.clone(),
                },
                span: Span {
                    st: var.span.st,
                    en: fields[fields.len() - 1].span.en,
                    file: self.file.clone(),
                },
            }
        }
    }

    pub fn parse_expr(&mut self, min_prec: u8, is_pattern: bool) -> Expr {
        // THE PRECEDENCE CLIMBING ALGORITHM
        // 1. Parse prefix/primary via nud(null denotations)

        let mut left = self.nud(is_pattern);
        // 2. Parse infix/postfix in loop
        while let Some(op) = self.peek() {
            let prec = self.precedence(&op);
            if prec < min_prec || prec == 0 {
                break;
            } //else if prec == 0 { break; } // if prec == 0; that means the token is not an op

            //Advance past the operator
            self.advance();
            // Dispatch to led(left denotation) handler
            left = self.led(left, op, prec, is_pattern);
        }

        left
    }

    fn global_helper(&mut self) -> (TypeSpec, Ident, Expr, bool, StmtInfo) {
        // Type: Single TypeHint | [TypeHint ,|. Number] | none

        let stm_st = self.peek_span().st.clone();

        let ty = self.expect_typespec();

        let name = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        let public;
        let nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            public = true;
            self.advance();
        } else {
            public = false;
        }
        self.expect(&Token::Colon); // If the next isn't colon, return an error

        let expr = self.parse_expr(0, false);

        let stm_en = self.expect_newline().st;

        let value;
        match expr.clone() {
            Expr {
                kind: ExprKind::Nexted { init, next },
                span,
            } => {
                value = *init;
                self.errs.push(Error {
                    message: format!("Expression {:#?} is not allowed in this scope", *next),
                    span: span,
                });
            }
            _ => {
                value = expr;
            }
        }

        let docs = self.end_stmt_parse(self.is_expr_block(&value));

        (
            ty,
            name,
            value,
            public,
            StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        )
    }

    fn parse_global(&mut self) -> TopLevel {
        let stm_st = self.peek_span().st.clone();
        let interpret;
        if self.peek_kind() == Some(Token::Interpret) {
            interpret = true;
            self.advance();
        } else {
            interpret = false;
        }

        self.expect(&Token::Global); // checks, then advance if true

        let mut arms = Vec::new();

        match self.peek_kind() {
            Some(Token::LBrace) => {
                self.advance();
                while self.peek_kind() != Some(Token::RBrace) {
                    let state;
                    if self.peek_kind() == Some(Token::Ampersand) {
                        self.advance();

                        state = DataState::GlobalRef;
                    } else {
                        state = DataState::Global;
                    }

                    let res = self.global_helper();
                    let mut info = res.4;
                    info.span.st = stm_st;
                    arms.push((true, state, res.0, res.1, res.2, res.3, info));
                }
                self.advance();
            }
            _ => {
                let state;
                if self.peek_kind() == Some(Token::Ampersand) {
                    self.advance();

                    state = DataState::GlobalRef;
                } else {
                    state = DataState::Global;
                }

                let res = self.global_helper();

                let mut info = res.4;
                info.span.st = stm_st;
                arms.push((false, state, res.0, res.1, res.2, res.3, info));
            }
        }

        let out;

        if arms.len() == 1 {
            let glob = arms[0].clone();
            if glob.0 == false {
                // if it has no block
                out = TopLevel::GlobalDecl {
                    interpret,
                    state: glob.1,
                    ty: glob.2,
                    name: glob.3,
                    value: glob.4,
                    public: glob.5,
                    info: glob.6,
                }
            } else {
                let mut t_ls = Vec::new();

                t_ls.push(Box::new(TopLevel::GlobalDecl {
                    interpret,
                    state: glob.1,
                    ty: glob.2,
                    name: glob.3,
                    value: glob.4,
                    public: glob.5,
                    info: glob.6,
                }));
                out = TopLevel::MultiGlobalDecl(t_ls);
            }
        } else {
            let mut t_ls = Vec::new();
            for glob in arms {
                t_ls.push(Box::new(TopLevel::GlobalDecl {
                    interpret,
                    state: glob.1,
                    ty: glob.2,
                    name: glob.3,
                    value: glob.4,
                    public: glob.5,
                    info: glob.6,
                }));
            }
            out = TopLevel::MultiGlobalDecl(t_ls);
        }

        out
    }

    fn parse_a_method_def(&mut self, is_for_trait: bool) -> Method {
        let stm_st = self.peek_span().st.clone();

        let ret_state;
        if self.peek_kind() == Some(Token::Ampersand) {
            self.advance();

            ret_state = DataState::Owner;
        } else {
            ret_state = DataState::Ref;
        }

        let expect = self.expect_typespec();

        let meth_self;

        let meth_self_st = self.peek_span().st.clone();
        match self.peek_kind() {
            Some(Token::Ampersand) => {
                self.advance();
                match self.peek_kind() {
                    Some(Token::Mut) => {
                        self.advance();
                        meth_self = MethodSelf::MutRef;
                    }
                    _ => {
                        meth_self = MethodSelf::Ref;
                    }
                }
            }
            _ => meth_self = MethodSelf::Default,
        }
        let meth_self_en = self.peek_span().en.clone();

        // Method's inherent value
        self.expect(&Token::SelfKw);

        self.expect(&Token::Dot);

        let is_public;
        let is_extern;

        let ident = self.expect_ident();

        // check for generic types
        let generics;

        match self.peek_kind() {
            // check the current token
            Some(Token::Less) => {
                // if it's < then there is a generic defination
                self.advance(); // consume it
                let mut genr = Vec::new(); //the genr variable contains all the generics of Generic type
                while self.peek_kind() != Some(Token::Greater) {
                    // in as much as the generit notation is not closed with >
                    let name = self.expect_ident();
                    self.expect(&Token::Colon); // <T:

                    let mut traits = Vec::new(); // this contains all the traits of the generic type
                    match self.peek_kind() {
                        Some(Token::LParen) => {
                            // if the next token is ( then trait is probably more than one
                            self.advance();
                            while self.peek_kind() != Some(Token::RParen) {
                                // as long is it's not yet closed with )
                                traits.push(self.expect_ident()); // add the identifier at current position to the traits
                                                                  // <T: (Clone
                                match self.peek_kind() {
                                    Some(Token::Comma) => {
                                        // <T: (Clone,
                                        self.advance();
                                    }
                                    _ => {
                                        // probably <T: (Clone)
                                        break;
                                    }
                                }
                            }
                            self.expect(&Token::RParen); // confirm that the token is )
                        }
                        _ => {
                            traits.push(
                                self.expect_ident(), // <T: Clone
                            );
                        }
                    }

                    genr.push(Generic { name, traits });

                    match self.peek_kind() {
                        Some(Token::Comma) => {
                            // <T: Clone, ___or___ <T: (Clone),
                            self.advance();
                            if self.peek_kind() == Some(Token::Greater) {
                                break;
                            }
                        }
                        _ => {
                            // <T: Clone> ___or___ <T: (Clone)>
                            break;
                        }
                    }
                }
                self.expect(&Token::Greater); // confirm that the token is >

                generics = Some(genr);
            }
            _ => {
                generics = None;
            }
        }

        let par_result = self.parse_func_params();
        let par = par_result.0;
        let variadic = par_result.1;

        // Flags Start
        // # public
        let mut nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            is_public = Trinary::True;
            self.advance();
        } else if nxt == Some(Token::QuestMark) {
            // a trait can sent its publicity to optional for implementer to choose
            if is_for_trait {
                is_public = Trinary::Unknown;
            } else {
                is_public = Trinary::False;
                self.errs.push(Error {
                    message: format!(
                        "Publicity most be explicite; remove ? to make method private"
                    ),
                    span: self.peek_span().clone(),
                })
            }
            self.advance();
        } else {
            is_public = Trinary::False;
        }

        // # Extern
        let func_expr;
        let stm_en;

        let docs;
        nxt = self.peek_kind();
        if nxt == Some(Token::Extern) {
            self.advance();

            stm_en = self.peek_span().en.clone();

            let abi_st = self.peek_span().st.clone();
            let expr = self.parse_expr(0, false);
            let abi_en = self.peek_span().en.clone();
            let abi;

            match expr {
                Expr {
                    kind: ExprKind::Str(ident),
                    span,
                } => {
                    abi = Ident { ident, span };
                }
                _ => {
                    abi = Ident {
                        ident: String::new(),
                        span: Span {
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: self.file.clone(),
                        },
                    };
                    self.errs.push(Error {
                        message: format!("A string is expected"),
                        span: Span {
                            // This spans the whole statement
                            st: abi_st,
                            en: abi_en,
                            file: self.file.clone(),
                        },
                    })
                }
            }

            is_extern = Some((abi, variadic));

            docs = self.end_stmt_parse(false);

            func_expr = None;
        } else {
            if variadic.is_some() {
                // Error:
                self.errs.push(Error {
                    message: format!(
                        "Kui functions cannot be variadic only an extern function can"
                    ),
                    span: variadic.unwrap(),
                })
            }

            is_extern = None;

            match self.peek_kind() {
                Some(Token::Colon) => {
                    self.advance();
                    func_expr = Some(self.parse_expr(0, false));
                    stm_en = self.peek_span().en.clone();

                    docs = self.end_stmt_parse(self.is_expr_block(func_expr.as_ref().unwrap()));
                }
                _ => {
                    func_expr = None;
                    stm_en = self.peek_span().en.clone();
                    docs = self.end_stmt_parse(false);
                }
            }
        }

        Method {
            worker_usage: vec![],
            meth_self: (
                meth_self,
                Span {
                    st: meth_self_st,
                    en: meth_self_en,
                    file: self.file.clone(),
                },
            ),
            // inher_val,
            // is_mut,
            expect,
            ret_state,
            ident,
            generics,
            is_public,
            is_extern,
            par,
            block: func_expr,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        }
    }

    fn parse_method_defs(&mut self, is_for_trait: bool) -> (Vec<Method>, Pos) {
        let mut methods = Vec::new();

        self.expect(&Token::LBrace);

        let en = Pos { column: 0, line: 0 };
        while self.peek_kind() != Some(Token::RBrace) {
            // -> {}
            methods.push(self.parse_a_method_def(is_for_trait));
        }
        self.expect(&Token::RBrace);

        (methods, en)
    }

    fn parse_cust_ty_associateds(&mut self, name: &String) {
        let mut ass = Vec::new();

        self.expect(&Token::LBrace);

        while self.peek_kind() != Some(Token::RBrace) {
            // -> {}
            match self.peek_kind() {
                Some(Token::Function) => {
                    let func = self.parse_func(false);

                    match func {
                        TopLevel::FuncDecl {
                            is_async: _,
                            expect,
                            ret_state,
                            ident,
                            generics,
                            is_visible: _,
                            is_public,
                            is_extern,
                            par,
                            worker_usage: _,
                            block,
                            info,
                        } => {
                            ass.push(CustTyAss::AssFunction(AssFunc {
                                worker_usage: vec![],
                                expect,
                                ret_state,
                                ident,
                                generics,
                                is_public,
                                is_extern,
                                par,
                                block: Some(block),
                                info,
                            }));
                        }
                        _ => (),
                    }
                }
                _ => {
                    let method = self.parse_a_method_def(false);

                    ass.push(CustTyAss::Method(method));
                }
            }
        }
        self.expect(&Token::RBrace);
        let ty = TypeKey::Custom(name.clone());
        if self.methods.contains_key(&ty) {
            self.methods.get_mut(&ty).unwrap().extend_from_slice(&ass);
        } else {
            self.methods.insert(ty, ass);
        }
    }

    fn parse_bridge(&mut self) -> TopLevel {
        let stm_st = self.peek_span().st.clone();
        let mut _public = false;

        self.expect(&Token::Bridge);

        let ass_state;
        if self.peek_kind() == Some(Token::Ampersand) {
            self.advance();

            ass_state = DataState::Ref;
        } else {
            ass_state = DataState::Owner;
        }

        let ass_type = self.expect_typespec();

        let ass_ident = self.expect_ident();

        let ass_is_mut;
        if self.peek_kind() == Some(Token::Mut) {
            self.advance();
            ass_is_mut = true;
        } else {
            ass_is_mut = false;
        }

        self.expect(&Token::To);

        let new_ty = self.expect_ident();

        self.expect(&Token::Colon);

        let block = self.parse_expr(0, false);

        let stm_en = self.peek_span().en.clone();

        let docs = self.end_stmt_parse(self.is_expr_block(&block));
        self.reverse();
        // It has to reverse, because it always return Err(()) which advances by default

        let bridge = BridgeDecl {
            ass_state,
            ass_type,
            ass_ident,
            ass_is_mut,
            new_ty: new_ty.clone(),
            public: _public,
            block,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        };

        if self.bridges.contains_key(&new_ty.ident) {
            self.bridges.get_mut(&new_ty.ident).unwrap().push(bridge);
        } else {
            self.bridges.insert(new_ty.ident.clone(), vec![bridge]);
        }

        TopLevel::Foo
    }

    fn parse_enum_variants(&mut self) -> EnumVariants {
        let name = self.expect_ident();
        let mut form = EnumVariantFormat::Simple;

        match self.peek_kind() {
            Some(Token::LParen) => {
                self.advance();
                form = EnumVariantFormat::Tuple;
            }
            Some(Token::LBrace) => {
                self.advance();
                form = EnumVariantFormat::Struct;
            }
            _ => (), // default
        }

        if form == EnumVariantFormat::Simple {
            EnumVariants::Simple(name)
        } else if form == EnumVariantFormat::Struct {
            self.parse_struct_variant(name)
        } else {
            // Tuple
            self.parse_tuple_variant(name)
        }
    }

    fn parse_tuple_variant(&mut self, name: Ident) -> EnumVariants {
        let mut fields = Vec::new();

        while self.peek_kind() != Some(Token::RParen) {
            match self.peek_kind() {
                Some(Token::Ampersand) => {
                    self.advance();
                    let state = DataState::Ref;

                    fields.push((state, self.expect_typespec()));
                }
                _ => {
                    fields.push((DataState::Owner, self.expect_typespec()));
                }
            }

            if self.peek_kind() == Some(Token::Comma) {
                self.advance();
            }
        }

        self.expect(&Token::RParen);

        EnumVariants::Tuple { name, fields }
    }

    fn parse_struct_variant(&mut self, name: Ident) -> EnumVariants {
        let mut fields = Vec::new();

        while self.peek_kind() != Some(Token::RBrace) {
            let state;
            if self.peek_kind() == Some(Token::Ampersand) {
                self.advance();

                state = DataState::Ref;
            } else {
                state = DataState::Owner;
            }

            let ty = self.expect_typespec();
            let ident = self.expect_ident();

            let is_mut;
            if self.peek_kind() == Some(Token::Mut) {
                self.advance();
                is_mut = true;
            } else {
                is_mut = false;
            }

            fields.push(Parameter {
                ident,
                state,
                ty,
                is_mut,
            });

            if self.peek_kind() == Some(Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);

        EnumVariants::Record { name, fields }
    }

    fn parse_enum(&mut self) -> TopLevel {
        let mut _public = false;
        let stm_init = self.expect(&Token::Enum).st;

        let ident = self.expect_ident();

        // check for generic types
        let generic_types = self.get_def_generics();

        let nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            _public = true;
            self.advance();
        }

        self.expect(&Token::LBrace);

        let mut vals = Vec::new();

        while self.peek_kind() != Some(Token::RBrace) {
            vals.push(self.parse_enum_variants());

            match self.peek_kind() {
                Some(Token::Comma) => {
                    self.advance();
                }
                Some(Token::Identifier(ref _s)) => {
                    self.errs.push(Error {
                        message: format!("Fi ',' dá wọn láàrin"),
                        span: Span {
                            st: self.peek_span().st.clone(),
                            en: self.peek_span().en.clone(),
                            file: self.file.clone(),
                        },
                    });
                }
                Some(_oth) => {
                    break;
                }
                None => {
                    self.errs.push(Error {
                        message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                        span: Span {
                            st: self.peek_span().st.clone(),
                            en: self.peek_span().en.clone(),
                            file: self.file.clone(),
                        },
                    });
                }
            }
        }
        let stm_en = self.peek_span().en.clone();
        self.expect(&Token::RBrace);

        if self.peek_kind() == Some(Token::ForwardArrow) {
            self.advance();
            self.parse_cust_ty_associateds(&ident.ident);
        }

        let docs = self.end_stmt_parse(true);

        TopLevel::EnumDecl {
            ident,
            generic_types,
            vals,
            traits: vec![],
            public: _public,
            info: StmtInfo {
                span: Span {
                    st: stm_init,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        }
    }

    fn parse_struct_params(&mut self) -> Vec<StructParam> {
        let mut params = Vec::new();

        while self.peek_kind() != Some(Token::RBrace) {
            let state;
            if self.peek_kind() == Some(Token::Ampersand) {
                self.advance();

                state = DataState::Ref;
            } else {
                state = DataState::Owner;
            }

            let ty = self.expect_typespec();
            let ident = self.expect_ident();

            let is_mut;
            if self.peek_kind() == Some(Token::Mut) {
                self.advance();
                is_mut = true;
            } else {
                is_mut = false;
            }

            let is_public;
            match self.peek_kind() {
                Some(Token::Public) => {
                    self.advance();
                    is_public = true;
                }
                _ => {
                    is_public = false;
                }
            }

            params.push(StructParam {
                par: Parameter {
                    ty,
                    state,
                    ident,
                    is_mut,
                },
                is_public,
            });

            match self.advance_kind() {
                // This advances pass both , and }
                Some(Token::Comma) => {
                    continue;
                }
                Some(Token::RBrace) => {
                    break;
                }
                Some(oth) => {
                    self.errs.push(Error {
                        message: format!(" '}}' ni oó yẹ kí oó wà níbí dípò '{:?}'", oth),
                        span: Span {
                            st: self.peek_span().st.clone(),
                            en: self.peek_span().en.clone(),
                            file: self.file.clone(),
                        },
                    });
                }
                None => {
                    self.errs.push(Error {
                        message: format!("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                        span: Span {
                            st: self.peek_span().st.clone(),
                            en: self.peek_span().en.clone(),
                            file: self.file.clone(),
                        },
                    });
                }
            }
        }

        params
    }

    fn parse_struct(&mut self) -> TopLevel {
        let mut public = false;
        let struct_init = self.expect(&Token::Struct);

        let ident = self.expect_ident();

        // check for generic types
        let generic_types = self.get_def_generics();

        let nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            public = true;
            self.advance();
        }

        self.expect(&Token::LBrace);

        let vals = self.parse_struct_params();

        if self.peek_kind() == Some(Token::ForwardArrow) {
            self.advance();
            self.parse_cust_ty_associateds(&ident.ident);
        }

        let docs = self.end_stmt_parse(true);

        TopLevel::StructDecl {
            ident,
            generic_types,
            vals,
            traits: vec![],
            public,
            info: StmtInfo {
                span: Span {
                    st: struct_init.st,
                    en: self.peek_span().en.clone(),
                    file: self.file.clone(),
                },
                docs,
            },
        }
    }

    fn parse_impl(&mut self) -> TopLevel {
        self.expect(&Token::Apply);

        let trt = self.expect_ident();

        self.expect(&Token::Resemble);

        let ty = self.expect_typespec();

        let methods = self.parse_method_defs(false).0;

        let value = TraitImpl {
            trt,
            ty: ty.clone(),
            methods,
        };

        let res = get_typekey(&ty.hint);
        if let Ok(trt_ty) = res {
            if self.trait_impls.contains_key(&trt_ty) {
                self.trait_impls.get_mut(&trt_ty).unwrap().push(value);
            } else {
                self.trait_impls.insert(trt_ty, vec![value]);
            }
        } else if let Err(err) = res {
            panic!("{:#?}", err)
        }

        TopLevel::Foo
    }

    fn parse_worker(&mut self) -> TopLevel {
        // worker is decorator
        let stm_st = self.peek_span().st.clone();
        let mut public = false;
        self.expect(&Token::Worker);

        let ident = self.expect_ident();

        self.expect(&Token::LParen);
        let func_name = self.expect_ident();
        self.expect(&Token::RParen);

        let nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            public = true;
            self.advance();
        }

        self.expect(&Token::LBrace);
        let from_block = self.parse_stmts();
        let block = from_block.0;
        let stm_en = from_block.1;
        let docs = self.end_stmt_parse(true);

        TopLevel::WorkerDef {
            ident,
            func_name,
            public,
            block,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        }
    }

    // fn parse_static(&mut self) -> TopLevel {
    //     let stm_st = self.peek_span().st.clone();
    //     let mut public = false;
    //     self.init_stm(&Token::Static);

    //     let ident = self.expect_ident();

    //     let nxt = self.peek_kind();
    //     if nxt == Some(Token::Public) {
    //         public = true;
    //         self.advance();
    //     }

    //     self.expect(&Token::LBrace);

    //     let mut globals = vec![];
    //     let mut funcs = vec![];
    //     let mut platform_n_s = vec![];
    //     let mut takes = vec![];
    //     // let mut macro_def = vec![];
    //     let mut custom_tys = vec![];
    //     let mut workers = vec![];
    //     let mut traits = vec![];

    //     while self.peek_kind() != Some(Token::RBrace) {
    //         let start_span = self.peek_span().clone();
    //         let element = self.parse_toplevel(false);
    //         match element.clone() {
    //             TopLevel::WorkerUsage { workers, info } => {
    //                 self.worker_usages = // this field is checked for every statement being parsed
    //                     Some(TopLevel::WorkerUsage { workers, info });
    //             }
    //             TopLevel::EnumDecl {
    //                 ident,
    //                 generic_types,
    //                 vals,
    //                 traits,
    //                 public,
    //                 info,
    //             } => {
    //                 if public {
    //                     // Error
    //                     self.errs.push(Error{
    //                         message: format!("Only global variable, function, and macro definations can be public in a static namespace"),
    //                         span: start_span.clone()
    //                     })
    //                 } else {
    //                     custom_tys.push(CustomLevel::EnumDecl {
    //                         ident,
    //                         generic_types,
    //                         vals,
    //                         traits,
    //                         public,
    //                         info,
    //                     });
    //                 }
    //             }
    //             TopLevel::StructDecl {
    //                 ident,
    //                 generic_types,
    //                 vals,
    //                 traits,
    //                 public,
    //                 info,
    //             } => {
    //                 if public {
    //                     // Error
    //                     self.errs.push(Error{
    //                         message: format!("Only global variable, function, and macro definations can be public in a static namespace"),
    //                         span: start_span.clone()
    //                     })
    //                 } else {
    //                     custom_tys.push(CustomLevel::StructDecl {
    //                         ident,
    //                         generic_types,
    //                         vals,
    //                         traits,
    //                         public,
    //                         info,
    //                     });
    //                 }
    //             }
    //             TopLevel::GenericType { name: _, traits: _ } => (), // Non used in parser
    //             TopLevel::TraitDef {
    //                 ident,
    //                 generic_types,
    //                 interprets,
    //                 methods,
    //                 is_public,
    //                 info,
    //             } => {
    //                 if is_public {
    //                     // Error
    //                     self.errs.push(Error{
    //                         message: format!("Only global variable, function, and macro definations can be public in a static namespace"),
    //                         span: start_span.clone()
    //                     })
    //                 } else {
    //                     traits.push(CustomLevel::TraitDef {
    //                         ident,
    //                         generic_types,
    //                         interprets,
    //                         methods,
    //                         is_public,
    //                         info,
    //                     });
    //                 }
    //             }
    //             TopLevel::Entry {
    //                 ident: _,
    //                 expect: _,
    //                 worker_usage: _,
    //                 block: _,
    //                 info: _,
    //             } => {
    //                 self.errs.push(Error {
    //                     message: format!("A static namespace cannot contain a main function"),
    //                     span: start_span.clone(),
    //                 });
    //             }
    //             TopLevel::UseStmt {
    //                 module: _,
    //                 nexted_level: _,
    //                 as_: _,
    //                 is_public: _,
    //                 info: _,
    //             } => {
    //                 self.errs.push(Error{
    //                     message: format!(
    //                         "A static namespace cannot contain a use statement.\nit can only either use an already existing module in unit or from a global lib e.g ::std"
    //                     ),
    //                     span: start_span.clone()
    //                 });
    //             }
    //             TopLevel::UseLibStmt { module: _, info: _ } => {
    //                 self.errs.push(Error{
    //                     message: format!(
    //                         "A static namespace cannot contain a use statement.\nit can only either use an already existing module in unit or from a global lib e.g ::std"
    //                     ),
    //                     span: start_span.clone()
    //                 });
    //             }
    //             TopLevel::DocLines { .. } => {
    //                 self.errs.push(Error{
    //                     message: format!(
    //                         "Documentation of a static namespace is to be defined after it's declaration.\n i.e adágún {:#?} gbangba {{",
    //                         ident
    //                     ),
    //                     span: start_span.clone()
    //                 });
    //             }
    //             TopLevel::MultiTakeStmt {
    //                 take,
    //                 from,
    //                 lib,
    //                 info,
    //             } => {
    //                 // converts the multiple takes into individual takes

    //                 for t in take {
    //                     takes.push(CustomLevel::TakeStmt {
    //                         take: t,
    //                         from: from.clone(),
    //                         lib: lib.clone(),
    //                         info: info.clone(),
    //                     });
    //                 }
    //             }
    //             TopLevel::TakeStmt {
    //                 take,
    //                 from,
    //                 lib,
    //                 info,
    //             } => {
    //                 takes.push(CustomLevel::TakeStmt {
    //                     take,
    //                     from,
    //                     lib,
    //                     info,
    //                 });
    //             }
    //             TopLevel::GlobalDecl {
    //                 interpret,
    //                 state,
    //                 ty,
    //                 name,
    //                 value,
    //                 public,
    //                 info,
    //             } => {
    //                 globals.push(CustomLevel::GlobalDecl {
    //                     interpret,
    //                     state,
    //                     ty,
    //                     name,
    //                     value,
    //                     public,
    //                     info,
    //                 });
    //             }
    //             TopLevel::MultiGlobalDecl(v) => {
    //                 for each in v {
    //                     let glob = *each;
    //                     match glob {
    //                         TopLevel::GlobalDecl {
    //                             interpret,
    //                             state,
    //                             ty,
    //                             name,
    //                             value,
    //                             public,
    //                             info,
    //                         } => {
    //                             globals.push(CustomLevel::GlobalDecl {
    //                                 interpret,
    //                                 state,
    //                                 ty,
    //                                 name,
    //                                 value,
    //                                 public,
    //                                 info,
    //                             });
    //                         }
    //                         _ => (),
    //                     }
    //                 }
    //             }
    //             TopLevel::FuncDecl {
    //                 is_async,
    //                 expect,
    //                 ret_state,
    //                 ident,
    //                 generics,
    //                 is_visible,
    //                 is_public,
    //                 is_extern,
    //                 par,
    //                 worker_usage,
    //                 block,
    //                 info,
    //             } => {
    //                 funcs.push(CustomLevel::FuncDecl {
    //                     is_async,
    //                     expect,
    //                     ret_state,
    //                     ident,
    //                     generics,
    //                     // only true toplevel functions can be visible in a platform match
    //                     is_public,
    //                     is_visible,
    //                     is_extern,
    //                     par,
    //                     worker_usage,
    //                     block,
    //                     info,
    //                 });
    //             }
    //             TopLevel::PlatformNS { arms, info } => {
    //                 platform_n_s.push(CustomLevel::PlatformNS { arms, info })
    //             }
    //             // TopLevel::MacroDef {
    //             //     ident,
    //             //     is_public,
    //             //     cases,
    //             //     info,
    //             // } => {
    //             //     macro_def.push(CustomLevel::MacroDef {
    //             //         ident,
    //             //         is_public,
    //             //         cases,
    //             //         info,
    //             //     });
    //             // }
    //             TopLevel::WorkerDef {
    //                 ident,
    //                 func_name,
    //                 public,
    //                 block,
    //                 info,
    //             } => {
    //                 workers.push(CustomLevel::WorkerDef {
    //                     ident,
    //                     func_name,
    //                     public,
    //                     block,
    //                     info,
    //                 });
    //             }
    //             TopLevel::StaticDef {
    //                 ident: _,
    //                 public: _,
    //                 block: _,
    //                 info: _,
    //             } => {
    //                 self.errs.push(Error {
    //                     message: format!("Static alápètúpè(recursive) is not allowed"),
    //                     span: start_span.clone(),
    //                 });
    //             }
    //             TopLevel::Foo => (),
    //         }
    //     }
    //     let stm_en = self.peek_span().en.clone();
    //     self.expect(&Token::RBrace);

    //     let docs = self.end_stmt_parse(true);

    //     let block = CustomBlock {
    //         takes,
    //         globals,
    //         // macro_def,
    //         custom_tys,
    //         traits,
    //         workers,
    //         funcs,
    //         platform_n_s,
    //         static_defs: vec![],
    //     };

    //     TopLevel::StaticDef {
    //         ident,
    //         public,
    //         block: block,
    //         info: StmtInfo {
    //             span: Span {
    //                 st: stm_st,
    //                 en: stm_en,
    //                 file: self.file.clone(),
    //             },
    //             docs,
    //         },
    //     }
    // }

    fn parse_worker_usage(&mut self, is_platform: bool) -> TopLevel {
        // true if custom follows
        let use_st = self.peek_span().st.clone();
        self.expect(&Token::NumSign);

        self.expect(&Token::LBracket);
        let mut workers = vec![];

        while self.peek_kind() != Some(Token::RBracket) {
            let ident = self.expect_ident();
            self.expect(&Token::LParen);
            let mut args = vec![];
            while self.peek_kind() != Some(Token::RParen) {
                args.push(self.expect_ident());
                match self.peek_kind() {
                    Some(Token::Comma) => {
                        self.advance();
                        continue;
                    }
                    _ => {
                        break;
                    }
                }
            }
            workers.push((ident, args));
            match self.peek_kind() {
                Some(Token::Comma) => {
                    self.advance();
                    continue;
                }
                _ => {
                    break;
                }
            }
        }

        self.expect(&Token::RBracket);

        let use_en = self.peek_span().st;
        self.expect_newline();

        // now this most be followed by either a function or a custom type
        let next_stm = self.parse_toplevel(is_platform);

        match next_stm {
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
                worker_usage: _,
                block,
                info,
            } => TopLevel::FuncDecl {
                is_async,
                expect,
                ret_state,
                ident,
                generics,
                is_visible,
                is_public,
                is_extern,
                par,
                worker_usage: workers,
                block,
                info,
            },
            TopLevel::StructDecl {
                ident,
                generic_types,
                vals,
                traits: _,
                public,
                info,
            } => TopLevel::StructDecl {
                ident,
                generic_types,
                vals,
                traits: workers,
                public,
                info,
            },
            TopLevel::EnumDecl {
                ident,
                generic_types,
                vals,
                traits: _,
                public,
                info,
            } => TopLevel::EnumDecl {
                ident,
                generic_types,
                vals,
                traits: workers,
                public,
                info,
            },
            _ => {
                // next_stm
                todo!(
                    "This statement cannot uses a worker error [{:#?}..{:#?}]",
                    use_st,
                    use_en
                )
            }
        }
        // TopLevel::WorkerUsage {
        //     workers,
        //     info: StmtInfo {
        //         span: Span {
        //             st: stm_st,
        //             en: stm_en,
        //             file: self.file.clone(),
        //         },
        //         docs: self.end_stmt_parse(false),
        //     },
        // }
    }

    fn parse_trait(&mut self) -> TopLevel {
        let stm_st = self.peek_span().st.clone();
        let is_public;
        self.expect(&Token::Trait);

        let ident = self.expect_ident();

        // check for generics
        let generic_types;
        match self.peek_kind() {
            Some(Token::Less) => {
                self.advance();
                generic_types = Some(self.expect_ident());
                self.expect(&Token::Greater);
            }
            _ => generic_types = None,
        }

        let nxt = self.peek_kind();
        if nxt == Some(Token::Public) {
            is_public = true;
            self.advance();
        } else {
            is_public = false;
        }

        // let interprets;

        // match self.peek_kind() {
        //     Some(Token::Interpret) => {
        //         self.expect(&Token::LBrace);
        //         interprets = self.parse_stmts().0;
        //     }
        //     _ => {
        //         interprets = vec![];
        //     }
        // }

        let mut stm_en = self.peek_span().en.clone();
        // let _ = stm_en;

        let methods;
        if self.peek_kind() == Some(Token::ForwardArrow) {
            self.advance();
            let methods_result = self.parse_method_defs(true);

            methods = methods_result.0;
            stm_en = methods_result.1;
        } else {
            methods = vec![];
        }

        TopLevel::TraitDef {
            ident,
            generic_types,
            // interprets,
            methods,
            is_public,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(false),
            },
        }
    }

    // fn get_frag_kind(&mut self) -> FragKind {
    //     match self.peek_kind() {
    //         Some(Token::Identifier(i)) => {
    //             self.advance();
    //             if i == String::from("ry") {
    //                 FragKind::AnyType
    //             } else if i == String::from("àfipè") {
    //                 FragKind::Ident
    //             } else if i == String::from("ìtúpayá") {
    //                 FragKind::Expr
    //             } else {
    //                 FragKind::Custom(i)
    //             }
    //         }
    //         Some(Token::TypeHint(t)) => {
    //             self.advance();
    //             if t == String::from("ọsán") {
    //                 FragKind::String
    //             } else if t == String::from("ǹjẹ́") {
    //                 FragKind::Bool
    //             } else {
    //                 self.errs.push(Error {
    //                     message: format!("kò sí irú ẹṣà tí oó ń jé {t}"),
    //                     span: Span {
    //                         st: self.peek_span().st.clone(),
    //                         en: self.peek_span().en.clone(),
    //                         file: self.file.clone(),
    //                     },
    //                 });

    //                 FragKind::Type
    //             }
    //         }
    //         _ => {
    //             self.errs.push(Error {
    //                 message: format!("Irú tókìn tí oó bá jé ni kí o kọ síbí"),
    //                 span: Span {
    //                     st: self.peek_span().st.clone(),
    //                     en: self.peek_span().en.clone(),
    //                     file: self.file.clone(),
    //                 },
    //             });

    //             FragKind::Type
    //         }
    //     }
    // }

    // fn parse_macro_pattern(&mut self) -> MacroPatt {
    //     // this function passes a single Macro Pattern; like Fragment e.t.c
    //     let mut pattern = MacroPatt::Foo;

    //     match self.peek() {
    //         Some(FullTok {
    //             kind: Token::Percent,
    //             span: _,
    //         }) => {
    //             // a metadata
    //             self.advance();

    //             match self.peek_kind() {
    //                 Some(Token::Identifier(_) | Token::TypeHint(_)) => {
    //                     // a fragment
    //                     let kind = self.get_frag_kind();

    //                     let name = self.expect_ident();

    //                     pattern = MacroPatt::Fragment { name, kind };
    //                 }
    //                 Some(Token::LParen) => {
    //                     // a repetition
    //                     self.advance();

    //                     let pttn = self.parse_macro_pattern();

    //                     self.expect(&Token::RParen);
    //                     let seperator;
    //                     let operator;

    //                     match self.peek() {
    //                         Some(FullTok {
    //                             kind: Token::Star,
    //                             span: _,
    //                         }) => {
    //                             operator = RepOperator::OnceOrMore;
    //                             seperator = None;
    //                             self.advance();
    //                         }
    //                         Some(FullTok {
    //                             kind: Token::Plus,
    //                             span: _,
    //                         }) => {
    //                             operator = RepOperator::ZeroOrMore;
    //                             seperator = None;
    //                             self.advance();
    //                         }
    //                         Some(FullTok {
    //                             kind: Token::QuestMark,
    //                             span: _,
    //                         }) => {
    //                             operator = RepOperator::Optional;
    //                             seperator = None;
    //                             self.advance();
    //                         }
    //                         Some(oth) => {
    //                             seperator = Some(oth);
    //                             self.advance();

    //                             match self.advance_kind() {
    //                                 Some(Token::Star) => {
    //                                     operator = RepOperator::OnceOrMore;
    //                                 }
    //                                 Some(Token::Plus) => {
    //                                     operator = RepOperator::ZeroOrMore;
    //                                 }
    //                                 Some(Token::QuestMark) => {
    //                                     operator = RepOperator::Optional;
    //                                 }
    //                                 _ => {
    //                                     self.errs.push(Error {
    //                                         message: format!("Lo *, + tàbí ?:\n"),
    //                                         span: Span {
    //                                             st: self.peek_span().st.clone(),
    //                                             en: self.peek_span().en.clone(),
    //                                             file: self.file.clone(),
    //                                         },
    //                                     });
    //                                     operator = RepOperator::OnceOrMore;
    //                                 }
    //                             }
    //                         }
    //                         None => {
    //                             self.errs.push(Error {
    //                                 message: format!("Lo *, + tàbí ?:\n"),
    //                                 span: Span {
    //                                     st: self.peek_span().st.clone(),
    //                                     en: self.peek_span().en.clone(),
    //                                     file: self.file.clone(),
    //                                 },
    //                             });

    //                             seperator = None;
    //                             operator = RepOperator::OnceOrMore;
    //                         }
    //                     }

    //                     pattern = MacroPatt::Repetition {
    //                         pattern: Box::new(pttn),
    //                         seperator,
    //                         operator,
    //                     }
    //                 }
    //                 _ => {
    //                     self.errs.push(Error {
    //                         message: format!("Lo *, + tàbí ?:\n"),
    //                         span: Span {
    //                             st: self.peek_span().st.clone(),
    //                             en: self.peek_span().en.clone(),
    //                             file: self.file.clone(),
    //                         },
    //                     });
    //                 }
    //             }
    //         }
    //         Some(FullTok {
    //             kind: Token::LParen,
    //             span: _,
    //         }) => {
    //             // to create another group
    //             let group = self.parse_macro_group();
    //             let delim = group.delim;
    //             let items = group.items;
    //             pattern = MacroPatt::Group { delim, items };
    //         }
    //         Some(literal) => {
    //             self.advance();
    //             pattern = MacroPatt::Literal(literal);
    //         }
    //         None => {
    //             self.errs.push(Error {
    //                 message: format!("Ko yẹ kí fáílì yìí-ó tíì parí níbí"),
    //                 span: Span {
    //                     st: self.peek_span().st.clone(),
    //                     en: self.peek_span().en.clone(),
    //                     file: self.file.clone(),
    //                 },
    //             });
    //         }
    //     }

    //     pattern
    // }

    // fn parse_macro_group(&mut self) -> MacroGroup {
    //     let delim;
    //     let mut items = vec![];

    //     if self.peek_kind() == Some(Token::LParen) {
    //         delim = Delimiter::Paren;
    //         self.advance();
    //         while self.peek_kind() != Some(Token::RParen) {
    //             items.push(self.parse_macro_pattern()); // parse one pattern after the other
    //         }
    //         self.advance();
    //     } else if self.peek_kind() == Some(Token::LBracket) {
    //         delim = Delimiter::Bracket;
    //         self.advance();
    //         while self.peek_kind() != Some(Token::RBracket) {
    //             items.push(self.parse_macro_pattern());
    //         }
    //         self.advance();
    //     } else if self.peek_kind() == Some(Token::LBrace) {
    //         delim = Delimiter::Brace;
    //         self.advance();
    //         while self.peek_kind() != Some(Token::RBrace) {
    //             items.push(self.parse_macro_pattern());
    //         }
    //         self.advance();
    //     } else {
    //         delim = Delimiter::Paren;
    //         items.push(MacroPatt::Literal(FullTok {
    //             kind: Token::Foo,
    //             span: Span {
    //                 st: Pos { column: 0, line: 0 },
    //                 en: Pos { column: 0, line: 0 },
    //                 file: self.file.clone(),
    //             },
    //         }));
    //     }

    //     MacroGroup { delim, items }
    // }

    // fn parse_macro(&mut self) -> TopLevel {
    //     let stm_st = self.peek_span().st.clone();
    //     self.init_stm(&Token::Macro);

    //     let ident = self.expect_ident();

    //     let is_public;
    //     let nxt = self.peek_kind();
    //     if nxt == Some(Token::Public) {
    //         is_public = true;
    //         self.advance();
    //     } else {
    //         is_public = false;
    //     }
    //     let mut cases = vec![];

    //     self.expect(&Token::LBrace);

    //     while self.peek_kind() != Some(Token::RBrace) {
    //         let arm_st = self.peek_span().st.clone();

    //         let pattern = self.parse_macro_group();

    //         self.expect(&Token::Colon);

    //         self.expect(&Token::LBrace);

    //         let mut block: Vec<FullTok> = vec![];

    //         let mut blocks: u32 = 0; // the first { of the arm

    //         let mut last_span = Span {
    //             st: Pos { column: 0, line: 0 },
    //             en: Pos { column: 0, line: 0 },
    //             file: self.file.clone(),
    //         };

    //         while let Some(tok) = self.tokens.get(self.pos).cloned() {
    //             match tok.clone() {
    //                 FullTok {
    //                     kind: Token::RBrace,
    //                     span: _,
    //                 } => {
    //                     if blocks == 0 {
    //                         // if there are no nexted {}s
    //                         // block.push(
    //                         //     FullTok {
    //                         //         kind: Token::RBrace,
    //                         //         span: last_span.clone()
    //                         //     }
    //                         // ); // always add a RBrace token after every last token
    //                         // might be vital to the parsing
    //                         break; // break the loop
    //                     }
    //                     let _ = last_span;
    //                     block.push(tok); // if doesn't break, push it
    //                     blocks -= 1;
    //                 }
    //                 FullTok {
    //                     kind: Token::LBrace,
    //                     span,
    //                 } => {
    //                     last_span = span;
    //                     block.push(tok); // t should be FullTok of Token::LBrace
    //                     blocks = blocks + 1;
    //                 }
    //                 _ => {
    //                     last_span = tok.span.clone();
    //                     block.push(tok);
    //                 }
    //             }
    //             self.pos += 1;
    //         }

    //         let arm_en = self.peek_span().en.clone();
    //         self.advance();

    //         if !block.is_empty() {
    //             match block[0].clone() {
    //                 FullTok {
    //                     kind: Token::NewLine,
    //                     span: _,
    //                 } => {
    //                     block.remove(0);
    //                 }
    //                 _ => (),
    //             }
    //         }

    //         cases.push(MacroCase {
    //             pattern,
    //             block,
    //             info: StmtInfo {
    //                 span: Span {
    //                     st: arm_st,
    //                     en: arm_en,
    //                     file: self.file.clone(),
    //                 },
    //                 docs: self.end_stmt_parse(true),
    //             },
    //         });
    //     }

    //     let stm_en = self.peek_span().en.clone();
    //     self.advance(); // consume }

    //     TopLevel::MacroDef {
    //         ident,
    //         is_public,
    //         cases,
    //         info: StmtInfo {
    //             span: Span {
    //                 st: stm_st,
    //                 en: stm_en,
    //                 file: self.file.clone(),
    //             },
    //             docs: self.end_stmt_parse(true),
    //         },
    //     }
    // }

    fn parse_func_params(&mut self) -> (Vec<InterfaceParam>, Option<Span>) {
        self.expect(&Token::LParen);
        let mut params = Vec::new();
        let mut is_variadic = None;
        while self.peek_kind() != Some(Token::RParen) {
            self.consume_newlines();
            // consumed all newline chars so that peek_span() wouldn't peek the span of a newline char
            let st = self.peek_span().st.clone();

            match self.peek() {
                Some(FullTok {
                    kind: Token::TDot,
                    span,
                }) => {
                    self.advance();
                    // iṣẹ́ 8d wá_iye(...) látòde "C"
                    self.expect(&Token::RParen);
                    self.reverse();
                    is_variadic = Some(span);
                    break;
                }
                _ => {
                    let param = self.expect_param();
                    let t = self.peek();

                    match t {
                        None => {
                            self.errs.push(Error {
                                message: format!("Kò yẹ kí fáílì yìí-ó tíì dópìn-in"),
                                span: Span {
                                    st: Pos { column: 0, line: 0 },
                                    en: Pos { column: 0, line: 0 },
                                    file: self.file.clone(),
                                },
                            });
                        }
                        _ => {
                            let mut is_mut = false;
                            let kind = t.as_ref().unwrap().kind.clone();
                            if kind == Token::Mut {
                                // let index = params.len()-1;
                                is_mut = true;

                                self.advance();

                                match param.state {
                                    DataState::Ref => {
                                        let span = t.unwrap().span;
                                        self.errs.push(
                                            Error {
                                                message: format!(
                                                    "Ìríjú var kan-ò lè yí sí ìrújú òmíràn; yọ `tóyí` kúrò torí {:?}-ò lè dalẹ̀ olówó rẹ̀",
                                                    param.ident
                                                ),
                                                span: span
                                            }
                                        );
                                    }
                                    _ => (),
                                }
                            }
                            let en = self.peek_span().en.clone();

                            if kind == Token::Comma {
                                self.advance();
                            }
                            params.push(InterfaceParam {
                                ident: param.ident.clone(),
                                state: param.state,
                                ty: param.ty,
                                is_mut,
                                span: Span {
                                    st,
                                    en,
                                    file: self.file.clone(),
                                },
                            }); // we need to do this, weither it breaks below or not
                            if kind == Token::RParen {
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.advance();
        (params, is_variadic)
    }

    fn get_def_generics(&mut self) -> Vec<Generic> {
        let generics;

        match self.peek_kind() {
            // check the current token
            Some(Token::Less) => {
                // if it's < then there is a generic defination
                self.advance(); // consume it
                let mut genr = Vec::new(); //the genr variable contains all the generics of Generic type
                while self.peek_kind() != Some(Token::Greater) {
                    // in as much as the generit notation is not closed with >
                    let name = self.expect_ident();
                    let mut traits = Vec::new(); // this contains all the traits of the generic type

                    match self.peek_kind() {
                        // traits are optional
                        Some(Token::Colon) => {
                            self.advance();
                            match self.peek_kind() {
                                Some(Token::LParen) => {
                                    // if the next token is ( then trait is probably more than one
                                    self.advance();
                                    while self.peek_kind() != Some(Token::RParen) {
                                        // as long is it's not yet closed with )
                                        traits.push(self.expect_ident()); // add the identifier at current position to the traits
                                                                          // <T: (Clone
                                        match self.peek_kind() {
                                            Some(Token::Comma) => {
                                                // <T: (Clone,
                                                self.advance();
                                            }
                                            _ => {
                                                // probably <T: (Clone)
                                                break;
                                            }
                                        }
                                    }
                                    self.expect(&Token::RParen); // confirm that the token is )
                                }
                                _ => {
                                    traits.push(
                                        self.expect_ident(), // <T: Clone
                                    );
                                }
                            }
                        }
                        _ => (),
                    }

                    genr.push(Generic { name, traits });

                    match self.peek_kind() {
                        Some(Token::Comma) => {
                            // <T: Clone, ___or___ <T: (Clone),
                            self.advance();
                            if self.peek_kind() == Some(Token::Greater) {
                                break;
                            }
                        }
                        _ => {
                            // <T: Clone> ___or___ <T: (Clone)>
                            break;
                        }
                    }
                }
                self.expect(&Token::Greater); // confirm that the token is >

                generics = genr;
            }
            _ => {
                generics = vec![];
            }
        }

        generics
    }

    fn parse_func(&mut self, is_platform: bool) -> TopLevel {
        let public;
        let visible;
        let is_extern;

        let is_async;
        if self.peek_kind() == Some(Token::Async) {
            is_async = true;
            self.advance();
        } else {
            is_async = false;
        }

        let func_init = self.expect(&Token::Function).st;

        let ret_state;
        if self.peek_kind() == Some(Token::Ampersand) {
            self.advance();

            ret_state = DataState::Ref;
        } else {
            ret_state = DataState::Owner;
        }

        let expect = self.expect_typespec();

        let ident = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        // check for generic types
        let generics = self.get_def_generics();

        let from_params = self.parse_func_params();
        let params = from_params.0;
        let variadic = from_params.1;

        // CHECK PLATFORM VISIBILITY
        let mut nxt = self.peek_kind();
        if nxt == Some(Token::UseOutside) {
            if !is_platform {
                self.errs.push(Error {
                    message: format!("iṣẹ́ aláfojúsùn nìkan ni àṣíá 'síta' wà fún"),
                    span: self.peek_span(),
                });
            }
            visible = true;
            self.advance();
        } else {
            visible = false;
        }

        // CHECK PUBLICITY
        if nxt == Some(Token::Public) {
            public = true;
            self.advance();
        } else {
            public = false;
        }

        let items;
        let mut stm_en;

        nxt = self.peek_kind();
        if nxt == Some(Token::Extern) {
            self.advance();
            stm_en = self.peek_span().en.clone();

            let abi_st = self.peek_span().st.clone();
            let expr = self.parse_expr(0, false);
            let abi_en = self.peek_span().en.clone();
            let abi;

            match expr {
                Expr {
                    kind: ExprKind::Str(ident),
                    span,
                } => {
                    abi = Ident { ident, span };
                }
                _ => {
                    abi = Ident {
                        ident: String::new(),
                        span: Span {
                            // This spans the whole statement
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: self.file.clone(),
                        },
                    };
                    self.errs.push(Error {
                        message: format!("A string is expected"),
                        span: Span {
                            // This spans the whole statement
                            st: abi_st,
                            en: abi_en,
                            file: self.file.clone(),
                        },
                    })
                }
            }

            is_extern = Some((abi, variadic));

            let nxt = self.peek_kind();
            if nxt != Some(Token::RBrace) {
                stm_en = self.expect_newline().st;
            }

            items = Expr {
                kind: ExprKind::Foo,
                span: Span {
                    st: Pos { column: 0, line: 0 },
                    en: Pos { column: 0, line: 0 },
                    file: self.file.clone(),
                },
            };
        } else {
            if variadic.is_some() {
                self.errs.push(Error {
                    message: format!(
                        "Kui functions cannot be variadic only an extern function can"
                    ),
                    span: variadic.unwrap(),
                })
            }
            is_extern = None;

            self.expect(&Token::Colon);

            items = self.parse_expr(0, false);

            stm_en = self.peek_span().en.clone();
        }

        // Parse Function-Level

        match ident.clone() {
            Ident { ident: i, span } => {
                if i == String::from("gbòógì") {
                    if is_async {
                        self.errs.push(Error {
                            message: format!("Ọ 'ò lè lo tàṣé fún iṣé gbòógì"),
                            span: span.clone(),
                        });
                    }
                    if !generics.is_empty() {
                        self.errs.push(Error {
                            message: format!("Ọ 'ò lè lo onírúnrú fún iṣé gbòógì"),
                            span: span.clone(),
                        });
                    }
                    if self.kind == FileKind::PrimEntry {
                        if params.is_empty() {
                            if is_async {
                                self.errs.push(Error {
                                    message: format!(
                                        "The main function cannot be an async function"
                                    ),
                                    span: span.clone(),
                                })
                            }
                            if is_extern.is_some() {
                                self.errs.push(Error {
                                    message: format!(
                                        "The main function cannot be an external function"
                                    ),
                                    span: span.clone(),
                                })
                            }
                            if items.kind == ExprKind::Foo {
                                self.errs.push(Error {
                                    message: format!("The main function most have a body"),
                                    span: span.clone(),
                                });
                                TopLevel::Foo
                            } else {
                                if is_platform {
                                    self.errs.push(Error {
                                        message: format!("isẹ́ gbòógì-ò lè wà níbí"),
                                        span: span.clone(),
                                    });

                                    TopLevel::Foo
                                } else {
                                    TopLevel::Entry {
                                        ident,
                                        info: StmtInfo {
                                            span: Span {
                                                st: func_init,
                                                en: stm_en.clone(),
                                                file: self.file.clone(),
                                            },
                                            docs: self.end_stmt_parse(self.is_expr_block(&items)),
                                        },
                                        expect,
                                        worker_usage: vec![],
                                        block: items,
                                    }
                                }
                            }
                        } else {
                            self.errs.push(Error {
                                message: format!("Iṣé gbòógì-ò nílò parameter kan-kan "),
                                span: span.clone(),
                            });

                            TopLevel::Foo
                        }
                    } else {
                        self.errs.push(Error {
                            message: format!("Inú fáílì àbáwọlé nìkan ni iṣé gbòógì lè wà"),
                            span: span.clone(),
                        });

                        TopLevel::Foo
                    }
                } else if i == String::from("fòyà") {
                    if is_async {
                        self.errs.push(Error {
                            message: format!("Ọ 'ò lè lo tàṣé fún iṣé fòyà"),
                            span: span.clone(),
                        });
                    }
                    if self.kind == FileKind::PrimEntry {
                        if is_platform {
                            self.errs.push(Error {
                                message: format!("isẹ́ fòyà-ò lè wà níbí"),
                                span: span.clone(),
                            });

                            TopLevel::Foo
                        } else {
                            TopLevel::Panic {
                                ident,
                                generics,
                                params,
                                public,
                                info: StmtInfo {
                                    span: Span {
                                        st: func_init,
                                        en: stm_en.clone(),
                                        file: self.file.clone(),
                                    },
                                    docs: self.end_stmt_parse(self.is_expr_block(&items)),
                                },
                                expect,
                                worker_usage: vec![],
                                block: items,
                            }
                        }
                    } else {
                        self.errs.push(Error {
                            message: format!("Inú fáílì àbáwọlé nìkan ni iṣé fòyà lè wà"),
                            span: span.clone(),
                        });

                        TopLevel::Foo
                    }
                } else {
                    TopLevel::FuncDecl {
                        is_async,
                        expect,
                        ret_state,
                        ident,
                        generics,
                        is_visible: visible,
                        is_public: public,
                        is_extern,
                        par: params,
                        worker_usage: vec![],
                        info: StmtInfo {
                            span: Span {
                                st: func_init,
                                en: stm_en,
                                file: self.file.clone(),
                            },
                            docs: self.end_stmt_parse(self.is_expr_block(&items)),
                        },
                        block: items,
                    }
                }
            }
        }
    }

    fn get_os(&mut self) -> Os {
        match self.peek() {
            Some(FullTok {
                kind: Token::Identifier(ident),
                span,
            }) => {
                if ident == format!("Linux") || ident == format!("linux") {
                    Os::Linux(span)
                } else if ident == format!("Darwin") || ident == format!("darwin") {
                    Os::Darwin(span)
                } else if ident == format!("Windows") || ident == format!("windows") {
                    Os::Windows(span)
                } else if ident == format!("Òmíràn") || ident == format!("òmíràn") {
                    Os::Others(span)
                } else {
                    Os::Undefined
                }
            }
            Some(oth) => {
                self.errs.push(Error {
                    message: format!(
                        "Inú Token yìí-ò yẹ kí oó wà níbí; irúfẹ́ km ni oó lè wà ní bẹ̀"
                    ),
                    span: oth.span,
                });
                Os::Undefined
            }
            None => {
                self.errs.push(Error {
                    message: String::from("Kò tíì yẹ kí fáílì yìí'ó dópìn-in níbí"),
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: self.file.clone(),
                    },
                });
                Os::Undefined
            }
        }
    }

    fn parse_os_target(&mut self) -> TopLevel {
        // it's just like a match statement, only not exactly
        let st;
        // let tar;
        match self.peek() {
            Some(FullTok {
                kind: Token::Os,
                span,
            }) => {
                st = span.st;
                // tar = "os";
            }
            //  Some(FullTok { kind: Token::Arch, span }) => {
            //     st = span.st;
            //     // tar = "arch";
            // }
            _ => panic!("Not an Os target"),
        }

        let mut arms = vec![];

        let en = self.expect(&Token::LBrace).en;
        while self.peek_kind() != Some(Token::RBrace) {
            let b_st = self.peek_span().st.clone(); //

            // type checker will check is the patterns are correct
            let target = self.get_os();

            // let en = self.peek_span().en.clone();
            self.expect(&Token::Colon);

            let mut block = CustomBlock {
                takes: vec![],
                globals: vec![],
                // macro_def: vec![],
                custom_tys: vec![],
                traits: vec![],
                workers: vec![],
                funcs: vec![],
                platform_n_s: vec![],
                static_defs: vec![],
            };

            self.expect(&Token::LBrace); // Only accept block
            while self.peek_kind() != Some(Token::RBrace) {
                // at toplevel, platform statement can only supports global variables, function, and macro definations
                let st = self.peek_span().st.clone();
                let top_level = self.parse_toplevel(true);
                let en = self.peek_span().en.clone();

                match top_level {
                    TopLevel::TakeStmt { take, from, info } => {
                        block.takes.push(CustomLevel::TakeStmt { take, from, info });
                    }
                    TopLevel::GlobalDecl {
                        interpret,
                        state,
                        ty,
                        name,
                        value,
                        public,
                        info,
                    } => {
                        block.globals.push(CustomLevel::GlobalDecl {
                            interpret,
                            state,
                            ty,
                            name,
                            value,
                            public,
                            info,
                        });
                    }
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
                        block: b,
                        info,
                    } => {
                        block.funcs.push(CustomLevel::FuncDecl {
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
                            block: b,
                            info,
                        });
                    }
                    TopLevel::StaticDef {
                        ident,
                        public,
                        block: sta_bk,
                        info,
                    } => {
                        block.static_defs.push(CustomLevel::StaticDef {
                            ident,
                            public,
                            block: sta_bk,
                            info,
                        });
                    }
                    // TopLevel::MacroDef {
                    //     ident,
                    //     is_public,
                    //     cases,
                    //     info,
                    // } => {
                    //     block.macro_def.push(CustomLevel::MacroDef {
                    //         ident,
                    //         is_public,
                    //         cases,
                    //         info,
                    //     });
                    // }
                    TopLevel::Foo => (), // Err(s) aleady handled
                    _ => {
                        self.errs.push(Error {
                            message: format!(
                                "Place this statement outside platform specific block"
                            ),
                            span: Span {
                                st,
                                en,
                                file: self.file.clone(),
                            },
                        });
                    }
                }
            }

            let b_en = self.peek_span().en.clone();
            self.expect(&Token::RBrace);

            arms.push(OsArm {
                target,
                block,
                span: Span {
                    st: b_st,
                    en: b_en,
                    file: self.file.clone(),
                },
            });
        }

        self.expect(&Token::RBrace);

        TopLevel::OsTarget {
            arms,
            info: StmtInfo {
                span: Span {
                    st,
                    en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    // fn parse_macro_use(&mut self) -> Result<TopLevel, ()> {
    //     let top_span = self.peek_span().clone();
    //     match self.peek_kind() {
    //         Some(Token::MacroCall(name)) => {
    //             self.init_stm(&Token::MacroCall(name.clone()));
    //             let expr = self.parse_macro_expr(name, top_span.line)?;

    //             match expr {
    //                 Expr::Macro { name, delimiter, tokens, span } => {
    //                     Ok(
    //                         TopLevel::MacroUse { name, delimiter, tokens, expanded: None, span }
    //                     )
    //                 }
    //                 _ => {
    //                     Err(
    //                         self.errs.push(
    //                             Error {
    //                                 message: format!(""),
    //                                 span: top_span
    //                             }
    //                         )
    //                     )
    //                 }
    //             }
    //         }
    //         _ => {
    //             Err(
    //                 self.errs.push(
    //                     Error {
    //                         message: format!(""),
    //                         span: top_span
    //                     }
    //                 )
    //             )
    //         }
    //     }
    // }

    // BLOCKLEVEL

    fn var_helper(&mut self) -> (TypeSpec, Ident, Expr, bool, Option<CatchFlag>, StmtInfo) {
        let stm_st = self.peek_span().st.clone();
        let mutable;

        let ty = self.expect_typespec();

        let name = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        let mut nxt = self.peek_kind();
        if nxt == Some(Token::Mut) {
            mutable = true;
            self.advance();
        } else {
            mutable = false;
        }

        let catch;
        nxt = self.peek_kind();
        if nxt == Some(Token::Catch) {
            self.advance();
            self.expect(&Token::LParen);
            let code = self.expect_ident();
            self.expect(&Token::RParen);
            self.expect(&Token::Colon);

            let expr = self.parse_expr(0, false);

            catch = Some(CatchFlag { code, expr });
        } else {
            catch = None;
        }
        self.expect(&Token::Colon); // If the next isn't colon, emits an error

        let value = self.parse_expr(0, false);

        let stm_en = self.peek_span().en.clone();

        let docs = self.end_stmt_parse(self.is_expr_block(&value));

        (
            ty,
            name,
            value,
            mutable,
            catch,
            StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs,
            },
        )
    }

    fn uninit_var_helper(&mut self) -> (TypeSpec, Ident, bool, StmtInfo) {
        let stm_st = self.peek_span().st.clone();
        let mutable;

        let ty = self.expect_typespec();

        let name = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        let nxt = self.peek_kind();
        if nxt == Some(Token::Mut) {
            mutable = true;
            self.advance();
        } else {
            mutable = false;
        }

        let mut stm_en = self.peek_span().en.clone();

        let nxt = self.peek_kind();
        if nxt != Some(Token::RBrace) {
            stm_en = self.expect_newline().st;
        }

        let info = StmtInfo {
            span: Span {
                st: stm_st,
                en: stm_en,
                file: self.file.clone(),
            },
            docs: self.end_stmt_parse(false),
        };

        (ty, name, mutable, info)
    }

    fn parse_uninit_var(&mut self) -> BlockLevel {
        let stm_st = self.peek_span().st.clone();
        self.expect(&Token::Have); // checks, then advance if true

        match self.peek_kind() {
            Some(Token::LBrace) => {
                self.advance();
                let mut uninits = Vec::new();
                while self.peek_kind() != Some(Token::RBrace) {
                    let state;
                    if self.peek_kind() == Some(Token::Ampersand) {
                        self.advance();

                        state = DataState::Ref;
                    } else {
                        state = DataState::Owner;
                    }

                    let res = self.uninit_var_helper();
                    let info = res.3;

                    uninits.push(Box::new(BlockLevel::UnInitVarDecl {
                        state,
                        ty: res.0,
                        name: res.1,
                        mutable: res.2,
                        info,
                    }));
                }
                self.advance();
                BlockLevel::MultiUnInitVarDecl(uninits)
            }
            _ => {
                let state;
                if self.peek_kind() == Some(Token::Ampersand) {
                    self.advance();

                    state = DataState::Ref;
                } else {
                    state = DataState::Owner;
                }

                let res = self.uninit_var_helper();

                let mut info = res.3;
                info.span.st = stm_st;
                BlockLevel::UnInitVarDecl {
                    state,
                    ty: res.0,
                    name: res.1,
                    mutable: res.2,
                    info,
                }
            }
        }
    }

    fn parse_var(&mut self) -> BlockLevel {
        let stm_st = self.peek_span().st.clone();

        let interpret;
        if self.peek_kind() == Some(Token::Interpret) {
            interpret = true;
            self.advance();
        } else {
            interpret = false;
        }

        self.expect(&Token::Var); // checks, then advance if true

        match self.peek_kind() {
            Some(Token::LBrace) => {
                self.advance();
                let mut globs = Vec::new();
                while self.peek_kind() != Some(Token::RBrace) {
                    let state;
                    if self.peek_kind() == Some(Token::Ampersand) {
                        self.advance();

                        state = DataState::Ref;
                    } else {
                        state = DataState::Owner;
                    }

                    let res = self.var_helper();
                    let mut info = res.5;
                    info.span.st = stm_st;
                    // add checks for:
                    // kí &tóyí Okùn èyí: yí èyun
                    // Or
                    // kí &Òkùn èyí: èyun

                    globs.push(Box::new(BlockLevel::VarDecl {
                        interpret,
                        state,
                        ty: res.0,
                        name: res.1,
                        catch: res.4,
                        value: res.2,
                        mutable: res.3,
                        info,
                    }));
                }
                self.advance();
                BlockLevel::MultiVarDecl(globs)
            }
            _ => {
                let state;
                if self.peek_kind() == Some(Token::Ampersand) {
                    self.advance();

                    state = DataState::Ref;
                } else {
                    state = DataState::Owner;
                }

                let res = self.var_helper();

                let mut info = res.5;
                info.span.st = stm_st;

                BlockLevel::VarDecl {
                    interpret,
                    state,
                    ty: res.0,
                    name: res.1,
                    catch: res.4,
                    value: res.2,
                    mutable: res.3,
                    info,
                }
            }
        }
    }

    fn parse_reassign(&mut self) -> BlockLevel {
        let stm_st = self.peek_span().st.clone();

        self.expect(&Token::Assign);

        let to_mut = self.parse_mutable_ident();

        let assign_op;

        match self.peek() {
            Some(FullTok {
                kind: Token::Plus,
                span,
            }) => {
                assign_op = Some(FullTok {
                    kind: Token::Plus,
                    span,
                });
                self.advance();
            }
            Some(FullTok {
                kind: Token::Minus,
                span,
            }) => {
                assign_op = Some(FullTok {
                    kind: Token::Minus,
                    span,
                });
                self.advance();
            }
            _ => {
                assign_op = None;
            }
        }

        self.expect(&Token::Colon);

        let val = self.parse_expr(0, false);

        let stm_en = self.peek_span().en.clone();

        BlockLevel::ReAssign {
            to_mut,
            assign_op,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(self.is_expr_block(&val)),
            },
            val,
        }
    }

    fn parse_nexted_func(&mut self) -> BlockLevel {
        let is_extern;

        let func_init = self.expect(&Token::Function).st;

        let expect = self.expect_typespec();

        let ident = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        // check for generic types
        let generics;

        match self.peek_kind() {
            // check the current token
            Some(Token::Less) => {
                // if it's < then there is a generic defination
                self.advance(); // consume it
                let mut genr = Vec::new(); //the genr variable contains all the generics of Generic type
                while self.peek_kind() != Some(Token::Greater) {
                    // in as much as the generit notation is not closed with >
                    let name = self.expect_ident();
                    self.expect(&Token::Colon); // <T:

                    let mut traits = Vec::new(); // this contains all the traits of the generic type
                    match self.peek_kind() {
                        Some(Token::LParen) => {
                            // if the next token is ( then trait is probably more than one
                            self.advance();
                            while self.peek_kind() != Some(Token::RParen) {
                                // as long is it's not yet closed with )
                                traits.push(self.expect_ident()); // add the identifier at current position to the traits
                                                                  // <T: (Clone
                                match self.peek_kind() {
                                    Some(Token::Comma) => {
                                        // <T: (Clone,
                                        self.advance();
                                    }
                                    _ => {
                                        // probably <T: (Clone)
                                        break;
                                    }
                                }
                            }
                            self.expect(&Token::RParen); // confirm that the token is )
                        }
                        _ => {
                            traits.push(
                                self.expect_ident(), // <T: Clone
                            );
                        }
                    }

                    genr.push(Generic { name, traits });

                    match self.peek_kind() {
                        Some(Token::Comma) => {
                            // <T: Clone, ___or___ <T: (Clone),
                            self.advance();
                            if self.peek_kind() == Some(Token::Greater) {
                                break;
                            }
                        }
                        _ => {
                            // <T: Clone> ___or___ <T: (Clone)>
                            break;
                        }
                    }
                }
                self.expect(&Token::Greater); // confirm that the token is >

                generics = Some(genr);
            }
            _ => {
                generics = None;
            }
        }

        let from_params = self.parse_func_params();
        let params = from_params.0;
        let variadic = from_params.1;

        let items;
        let mut stm_en;

        let nxt = self.peek_kind();
        if nxt == Some(Token::Extern) {
            self.advance();

            stm_en = self.peek_span().en.clone();

            let abi_st = self.peek_span().st.clone();
            let expr = self.parse_expr(0, false);
            let abi_en = self.peek_span().en.clone();
            let abi; // ABI -> Application Binary Interface

            match expr {
                Expr {
                    kind: ExprKind::Str(ident),
                    span,
                } => {
                    abi = Ident { ident, span };
                }
                _ => {
                    abi = Ident {
                        ident: String::new(),
                        span: Span {
                            // This spans the whole statement
                            st: Pos { column: 0, line: 0 },
                            en: Pos { column: 0, line: 0 },
                            file: self.file.clone(),
                        },
                    };

                    self.errs.push(Error {
                        message: format!("A string is expected"),
                        span: Span {
                            // This spans the whole statement
                            st: abi_st,
                            en: abi_en,
                            file: self.file.clone(),
                        },
                    })
                }
            }

            is_extern = Some((abi, variadic));

            let nxt = self.peek_kind();
            if nxt != Some(Token::RBrace) {
                stm_en = self.expect_newline().st;
            }

            items = Expr {
                kind: ExprKind::Foo,
                span: Span {
                    st: Pos { column: 0, line: 0 },
                    en: Pos { column: 0, line: 0 },
                    file: self.file.clone(),
                },
            };
        } else {
            if variadic.is_some() {
                self.errs.push(Error {
                    message: format!(
                        "Kui functions cannot be variadic only an extern function can"
                    ),
                    span: variadic.unwrap(),
                })
            }

            is_extern = None;
            self.expect(&Token::Colon);
            items = self.parse_expr(0, false);
            stm_en = self.peek_span().en.clone();
        }

        // Flags End

        BlockLevel::FuncDecl {
            expect,
            ident,
            generics,
            is_extern,
            par: params,
            worker_usage: vec![],
            info: StmtInfo {
                span: Span {
                    st: func_init,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(self.is_expr_block(&items)),
            },
            block: items,
        }
    }

    fn parse_func_call(&mut self) -> BlockLevel {
        let stm_st = self.peek_span().st.clone();

        self.expect(&Token::Tilde);

        let expr = self.parse_expr(0, false);

        match expr {
            Expr {
                kind:
                    ExprKind::ArgBuffer {
                        asy,
                        generics,
                        args,
                        name,
                    },
                ..
            } => {
                let stm_en = self.peek_span().en.clone();

                BlockLevel::FuncCall {
                    func: Expr {
                        kind: ExprKind::ArgBuffer {
                            asy,
                            generics,
                            args,
                            name: name.clone(),
                        },
                        span: name.span,
                    },
                    info: StmtInfo {
                        span: Span {
                            st: stm_st,
                            en: stm_en,
                            file: self.file.clone(),
                        },
                        docs: self.end_stmt_parse(false),
                    },
                }
            }
            Expr {
                kind: ExprKind::ObjAssess { obj, fields },
                span: _,
            } => {
                // if its a field assees then it most end with a function; the semantic analyser will check if it returns void
                let stm_en = self.peek_span().en.clone();
                let i = fields.len() - 1;
                match &fields[i] {
                    Expr {
                        kind:
                            ExprKind::ArgBuffer {
                                asy: _,
                                generics: _,
                                args: _,
                                name: _,
                            },
                        span: _,
                    } => BlockLevel::FuncCall {
                        func: Expr {
                            kind: ExprKind::ObjAssess {
                                obj: obj.clone(),
                                fields: fields.clone(),
                            },
                            span: Span {
                                st: obj.clone().span.st,
                                en: if fields.len() == 0 {
                                    obj.span.en
                                } else {
                                    fields[fields.len() - 1].span.en
                                },
                                file: self.file.clone(),
                            },
                        },
                        info: StmtInfo {
                            span: Span {
                                st: stm_st,
                                en: stm_en,
                                file: self.file.clone(),
                            },
                            docs: self.end_stmt_parse(false),
                        },
                    },
                    _ => {
                        self.errs.push(Error {
                            message: format!("The statement does not expect a value"),
                            span: Span {
                                st: stm_st,
                                en: stm_en,
                                file: self.file.clone(),
                            },
                        });

                        BlockLevel::Foo
                    }
                }
            }
            Expr {
                kind: ExprKind::NamespaceAccess { ident, access },
                span,
            } => {
                let stm_en = self.peek_span().en.clone();

                BlockLevel::FuncCall {
                    func: Expr {
                        kind: ExprKind::NamespaceAccess { ident, access },
                        span,
                    },
                    info: StmtInfo {
                        span: Span {
                            st: stm_st,
                            en: stm_en,
                            file: self.file.clone(),
                        },
                        docs: self.end_stmt_parse(false),
                    },
                }
            }
            _ => {
                let stm_en = self.peek_span().en.clone();
                self.errs.push(Error {
                    message: format!("The statement does not expect a value"),
                    span: Span {
                        st: stm_st,
                        en: stm_en,
                        file: self.file.clone(),
                    },
                });

                BlockLevel::Foo
            }
        }
    }

    fn parse_block(&mut self) -> BlockLevel {
        let st = self.expect(&Token::LBrace).st.clone();
        let stms = self.parse_stmts();

        BlockLevel::Block {
            stms: stms.0,
            info: StmtInfo {
                span: Span {
                    st,
                    en: stms.1,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    fn parse_var_init(&mut self) -> BlockLevel {
        let stm_st = self.peek_span().st.clone();
        let ident = self.expect_ident();

        self.expect(&Token::Colon);

        let expr = self.parse_expr(0, false);
        let stm_en = self.peek_span().en.clone();

        BlockLevel::VarInit {
            ident,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(self.is_expr_block(&expr)),
            },
            expr,
        }
    }

    fn parse_expr_call_or_var_init(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::Colon).st.clone();

        let expr = self.parse_expr(0, false);

        let stm_en = self.peek_span().en.clone();

        BlockLevel::BindedVal {
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(self.is_expr_block(&expr)),
            },
            expr,
        }
    }

    fn parse_if(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::If).st.clone();
        let bool_expr;
        let mut else_ifs = vec![];
        let mut else_exe = None;

        bool_expr = self.parse_expr(0, false);
        self.expect(&Token::Colon);

        // Parse Function-Level

        let exe = self.parse_expr(0, false);
        // self.expect(&Token::LBrace);
        // let from_block = self.parse_stmts()?;
        // let items = from_block.0;
        // let mut stm_en = from_block.1;

        while self.peek_kind() == Some(Token::Else) {
            self.advance();
            if self.peek_kind() == Some(Token::If) {
                let else_if_stm_st = self.peek_span().st.clone();
                self.advance();
                let else_if_expr = self.parse_expr(0, false);
                // Parse Function-Level
                self.expect(&Token::Colon);

                let else_if_exe = self.parse_expr(0, false);
                let else_if_stm_en = self.peek_span().en.clone();

                else_ifs.push(ElseIf {
                    bool_expr: else_if_expr,
                    exe: else_if_exe,
                    span: Span {
                        st: else_if_stm_st,
                        en: else_if_stm_en.clone(),
                        file: self.file.clone(),
                    },
                });
            } else {
                // should be else_block then
                // Parse Function-Level
                self.expect(&Token::Colon);

                else_exe = Some(self.parse_expr(0, false));
                break;
            }
        }

        let stm_en = self.peek_span().en.clone();

        BlockLevel::IfStmt {
            bool_expr,
            exe: exe,
            else_ifs,
            else_exe,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(false), // we need to find out though
            },
        }
    }

    fn parse_do_while(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::Do).st.clone();

        self.expect(&Token::LBrace);
        let exe = self.parse_stmts().0;

        self.expect(&Token::While);
        self.expect(&Token::LParen);
        let bool_expr = self.parse_expr(0, false);
        self.expect(&Token::RParen);

        let stm_en = self.peek_span().en.clone();

        BlockLevel::DoWhileStmt {
            exe,
            expr: bool_expr,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(false),
            },
        }
    }

    fn parse_while(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::While).st.clone();

        let bool_expr = self.parse_expr(0, false);
        self.expect(&Token::Colon);

        self.expect(&Token::LBrace); // loops most be blocked
        let exe = self.parse_stmts().0;

        let stm_en = self.peek_span().en.clone();

        BlockLevel::WhileStmt {
            expr: bool_expr,
            exe,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    fn parse_loop(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::Loop).st.clone();

        self.expect(&Token::LBrace);
        let exe = self.parse_stmts().0;

        let stm_en = self.peek_span().en.clone();

        BlockLevel::LoopStmt {
            exe,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    fn parse_peeking(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::Peeking).st.clone();

        // fún 8p i nínú 1..10: {...}
        let param = self.expect_param();

        self.expect(&Token::From);

        let expr = self.parse_expr(0, false);
        self.expect(&Token::Colon);

        self.expect(&Token::LBrace);
        let block = self.parse_stmts();

        let exe = block.0;

        let stm_en = block.1;

        BlockLevel::PeekingStmt {
            param,
            expr,
            exe,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    fn parse_for(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::For).st.clone();

        // self.expect(&Token::LParen);
        let assign_st = self.peek_span().st.clone();
        let ty = self.expect_typespec();

        let name = self.expect_ident(); // Check if there is an identifier, if there is, that is the name and there isn't, return an error

        let nxt = self.peek_kind();
        if nxt == Some(Token::Mut) {
            self.advance();
        }

        self.expect(&Token::Colon);
        let value = Box::new(self.parse_expr(0, false));
        let assign_en = self.peek_span().en.clone();
        // assign
        let assign = ForAssign {
            ty,
            name,
            value,
            span: Span {
                st: assign_st,
                en: assign_en,
                file: self.file.clone(),
            },
        };

        self.expect(&Token::Comma);

        let op_option = self.advance();
        let expr = Box::new(self.parse_expr(0, false));

        let op;
        match op_option {
            Some(tok) => {
                match tok {
                    FullTok {
                        kind: Token::Less,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Greater,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::LessEq,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::GreaterEq,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::NotEqual,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Equal,
                        span: _,
                    } => (),
                    _ => {
                        self.errs.push(Error {
                            message: format!("Expected conditional expressing"),
                            span: tok.span.clone(),
                        });
                    }
                }
                op = tok;
            }
            None => {
                op = FullTok {
                    kind: Token::EOF,
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: self.file.clone(),
                    },
                }
            }
        }

        let cond = ForCond { op, expr };

        self.expect(&Token::Comma);

        let op_option = self.advance(); // ... + ...
        let expr = Box::new(self.parse_expr(0, false));

        let incr_op;
        match op_option {
            Some(tok) => {
                match tok {
                    FullTok {
                        kind: Token::Plus,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Minus,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Div,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::BackSlash,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Star,
                        span: _,
                    }
                    | FullTok {
                        kind: Token::Percent,
                        span: _,
                    } => (),
                    _ => {
                        self.errs.push(Error {
                            message: format!("Expected arithmetic operator"),
                            span: tok.span.clone(),
                        });
                    }
                }
                incr_op = tok;
            }
            None => {
                incr_op = FullTok {
                    kind: Token::EOF,
                    span: Span {
                        st: Pos { column: 0, line: 0 },
                        en: Pos { column: 0, line: 0 },
                        file: self.file.clone(),
                    },
                }
            }
        }

        let incr = ForIncr { incr_op, expr };
        self.expect(&Token::Colon);

        self.expect(&Token::LBrace);
        let block = self.parse_stmts();
        let exe = block.0;

        let stm_en = block.1;

        BlockLevel::ForStmt {
            assign,
            cond,
            incr,
            exe,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }

    fn parse_match(&mut self) -> BlockLevel {
        let stm_st = self.expect(&Token::Match).st.clone();
        let mut arms = vec![];

        let var = self.expect_ident();
        self.expect(&Token::LBrace);

        // let default_string = String::from("_");

        while self.peek_kind() != Some(Token::RBrace) {
            // for each cases
            let expr;
            let mut has_default = false;

            match self.peek() {
                Some(FullTok {
                    kind: Token::Identifier(ident),
                    span,
                }) => match self.peek() {
                    Some(FullTok {
                        kind: Token::Colon,
                        span: _,
                    }) => {
                        if ident == format!("_") {
                            self.advance();
                            if has_default && ident == format!("_") {
                                self.errs.push(Error {
                                    message: format!("You can't have more than one default block"),
                                    span: Span {
                                        st: self.peek_span().st.clone(),
                                        en: self.peek_span().en.clone(),
                                        file: self.file.clone(),
                                    },
                                });
                            }
                            has_default = true;
                            let _ = has_default;
                            expr = Expr {
                                kind: ExprKind::Default,
                                span: span,
                            };
                        } else {
                            expr = Expr {
                                kind: ExprKind::Ident {
                                    affix: None,
                                    ident: Ident {
                                        ident,
                                        span: span.clone(),
                                    },
                                },
                                span: span,
                            };
                        }
                    }
                    _ => {
                        self.advance();
                        expr = Expr {
                            kind: ExprKind::Default,
                            span: Span {
                                st: Pos { column: 0, line: 0 },
                                en: Pos { column: 0, line: 0 },
                                file: self.file.clone(),
                            },
                        };
                        self.expect(&Token::Colon);
                        self.errs.push(Error {
                            message: format!("Rántí fi `:` síwájú rẹ̀"),
                            span: Span {
                                st: self.peek_span().st.clone(),
                                en: self.peek_span().en.clone(),
                                file: self.file.clone(),
                            },
                        });
                    }
                },
                _ => {
                    self.advance();
                    expr = self.parse_expr(0, true);
                    self.expect(&Token::Colon);
                }
            }

            let exe = self.parse_expr(0, false);

            arms.push(Case { expr, exe });
        }
        let stm_en = self.peek_span().en.clone();
        self.expect(&Token::RBrace);

        BlockLevel::MatchStmt {
            var,
            arms,
            info: StmtInfo {
                span: Span {
                    st: stm_st,
                    en: stm_en,
                    file: self.file.clone(),
                },
                docs: self.end_stmt_parse(true),
            },
        }
    }
}

pub fn get_typekey(spec: &TypeHint) -> Result<TypeKey, Error> {
    match spec {
        TypeHint::I8(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::I8(Some(IntRangeKey { st, en }))),
        },
        TypeHint::I16(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::I16(Some(IntRangeKey { st, en }))),
        },
        TypeHint::I32(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::I32(Some(IntRangeKey { st, en }))),
        },
        TypeHint::I64(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::I64(Some(IntRangeKey { st, en }))),
        },
        TypeHint::I128(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::I128(Some(IntRangeKey { st, en }))),
        },
        TypeHint::ISize(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::ISize(Some(IntRangeKey { st, en }))),
        },
        TypeHint::U8(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::U8(Some(IntRangeKey { st, en }))),
        },
        TypeHint::U16(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::U16(Some(IntRangeKey { st, en }))),
        },
        TypeHint::U32(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::U32(Some(IntRangeKey { st, en }))),
        },
        TypeHint::U64(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::U64(Some(IntRangeKey { st, en }))),
        },
        TypeHint::U128(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::U128(Some(IntRangeKey { st, en }))),
        },
        TypeHint::USize(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::USize(Some(IntRangeKey { st, en }))),
        },
        TypeHint::F32(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::F32(Some(IntRangeKey { st, en }))),
        },
        TypeHint::F64(Some(range)) => match range.clone() {
            IntRange { st, en, span: _ } => Ok(TypeKey::F64(Some(IntRangeKey { st, en }))),
        },
        TypeHint::Str => Ok(TypeKey::Str),
        TypeHint::Char => Ok(TypeKey::Char),
        TypeHint::Bool => Ok(TypeKey::Bool),
        TypeHint::Trinary => Ok(TypeKey::Trinary),
        TypeHint::Quaternary => Ok(TypeKey::Quaternary),
        TypeHint::End => Ok(TypeKey::End),
        TypeHint::Custom { get, access, .. } => {
            if access.is_empty() {
                Ok(TypeKey::Custom(get.ident.clone()))
            } else {
                Ok(TypeKey::Custom(access.last().unwrap().ident.clone()))
            }
        }
        TypeHint::Void => Ok(TypeKey::Void), // òfìfo for void functions
        TypeHint::SelfKw => Ok(TypeKey::SelfKw), // òwun
        TypeHint::RawPtr { tysp, mutb } => {
            let key = get_typekey(&tysp.hint)?;

            Ok(TypeKey::RawPtr {
                tysp: Box::new(key),
                mutb: *mutb,
            })
        }
        // Scaler(TypeHint), // Doesn't need span, type-hint has it
        TypeHint::MVSTy(key, num) => {
            let typekey = get_typekey(&*key)?;

            Ok(TypeKey::MVSTy(Box::new(typekey), num.clone()))
        } // Multiple Value - Same Types
        TypeHint::MVMTy(key) => {
            let mut new = vec![];

            for ky in key {
                let tykey = get_typekey(ky)?;
                new.push(tykey);
            }
            Ok(TypeKey::MVMTy(new))
        }
        _ => panic!("Unexpected type"),
    }
}
