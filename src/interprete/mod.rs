// use std::collections::HashMap;

// use crate::{
//     parser::{BlockLevel, Error, Expr, TopLevel, TypeHint, TypeSpec},
//     scanner::{Span, Token, TruthVal},
// };

// #[derive(Debug)]
// #[allow(dead_code)]
// pub struct InterpretesResults {
//     pub pos: usize,
//     pub interprs: Vec<TopLevel>,
//     pub globals: HashMap<String, BlockLevel>,
//     pub in_scope: Vec<Scope>,
//     pub errs: Vec<Error>,
// }

// #[derive(Debug, PartialEq, PartialOrd)]
// #[allow(dead_code)]
// pub struct Determ {
//     ty: TypeSpec,
//     value: Expr,
// }

// #[derive(Debug, Clone)]
// #[allow(dead_code)]
// pub struct Scope {
//     pub globals: HashMap<String, BlockLevel>,
//     pub vars: HashMap<String, BlockLevel>,
// }

// impl InterpretesResults {
//     // pub fn new(interprs: Vec<TopLevel>) -> Self {
//     //     Self {
//     //         pos: 0,
//     //         interprs,
//     //         globals: HashMap::new(),
//     //         in_scope: vec![Scope {
//     //             globals: HashMap::new(),
//     //             vars: HashMap::new(),
//     //         }],
//     //         errs: vec![],
//     //     }
//     // }
// }
