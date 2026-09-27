// src/cfg.rs additions

use std::collections::HashMap;

use crate::parser::{Expr, Ident, TypeSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub struct BlockId(pub usize);

/// The ownership state of a variable at a program point
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum OwnershipState {
    Owned,               // value is owned, not borrowed
    Moved,               // value has been moved out — invalid
    MovedInBranch,       // moved in some branch — maybe invalid
    BorrowedShared(u32), // borrowed — u32 = borrow count
    BorrowedMut,         // mutably borrowed — exclusive
    Dropped,             // explicitly dropped
    Uninitialized,       // declared but not yet assigned
}

/// A single borrow — a loan from lender to borrower
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Loan {
    pub id: LoanId,
    pub lender: Ident,   // variable being borrowed
    pub borrower: Ident, // variable holding the borrow
    pub kind: BorrowKind,
    pub created: ProgramPoint, // where borrow is created
    pub expires: ProgramPoint, // where borrow must end
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum BorrowKind {
    Shared,  // &T
    Mutable, // &mut T
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct LoanId(pub usize);

/// A specific point in the program — block + instruction index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub struct ProgramPoint {
    pub block: BlockId,
    pub inst: usize,
}

/// Liveness information for a variable
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LiveRange {
    pub name: String,
    pub defined: ProgramPoint,   // where variable is first assigned
    pub uses: Vec<ProgramPoint>, // all use sites
    pub last_use: ProgramPoint,  // last use — drop point
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Terminator {
    /// Unconditional jump
    Jump(BlockId),

    /// Conditional branch — true_bb if cond, else false_bb
    Branch {
        cond: Expr,
        true_bb: BlockId,
        false_bb: BlockId,
    },

    /// Function return
    Return(Option<Expr>),

    /// Unreachable — block has no exit (after panic etc)
    Unreachable,
}

/// Extended basic block with borrow checker information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BasicBlock {
    pub id: BlockId,
    pub label: String,
    pub insts: Vec<Inst>, // instructions with ownership info
    pub terminator: Option<Terminator>,
    pub preds: Vec<BlockId>,
    pub succs: Vec<BlockId>,

    // borrow checker state at entry and exit of this block
    pub state_in: OwnershipMap,  // ownership state when entering
    pub state_out: OwnershipMap, // ownership state when exiting
    pub loans_in: Vec<LoanId>,   // active loans when entering
    pub loans_out: Vec<LoanId>,  // active loans when exiting

    // which group does each variable belong to at block entry
    // pub group_in:   HashMap<String, GroupPath>,
    // // which group does each variable belong to at block exit
    // pub group_out:  HashMap<String, GroupPath>,
    // // which group paths are invalidated at each program point
    // pub invalidated: HashMap<ProgramPoint, Vec<GroupPath>>,
}

impl BasicBlock {
    fn new(id: BlockId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            insts: vec![],
            terminator: None,
            preds: vec![],
            succs: vec![],

            state_in: HashMap::new(),
            state_out: HashMap::new(),
            loans_in: vec![],
            loans_out: vec![],

            // group_in: HashMap::new(),
            // group_out: HashMap::new(),
            // invalidated: HashMap::new()
        }
    }

    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }
}

/// Ownership state of all variables at a program point
#[allow(dead_code)]
pub type OwnershipMap = std::collections::HashMap<String, OwnershipState>;

// /// A group — a set of values that can be mutably aliased
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct GroupId(pub usize);

// /// The path to a child group
// /// e.g. r.rings.items[*] means "elements of rings field"
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum GroupPath {
//     /// The root group itself
//     Root(GroupId),
//     /// A field of the parent group that is a dynamic container
//     /// e.g. r.rings.items[*]
//     ChildCollection {
//         parent: Box<GroupPath>,
//         field:  String,        // field name e.g. "rings"
//     },
//     /// A pointer dereference — the heap object pointed to
//     /// e.g. r.armor[]
//     ChildPtr {
//         parent: Box<GroupPath>,
//         field:  String,
//     },
//     /// A variant payload
//     /// e.g. r.hand.Shield
//     ChildVariant {
//         parent:  Box<GroupPath>,
//         field:   String,
//         variant: String,
//     },
// }

// impl GroupPath {
//     /// Is this path a child of another path?
//     pub fn is_child_of(&self, other: &GroupPath) -> bool {
//         match self {
//             GroupPath::Root(_) => false,
//             GroupPath::ChildCollection { parent, .. } |
//             GroupPath::ChildPtr        { parent, .. } |
//             GroupPath::ChildVariant    { parent, .. } => {
//                 parent.as_ref() == other || parent.is_child_of(other)
//             }
//         }
//     }
// }

// /// What kind of type is a variable — determines child group rules
// #[derive(Debug, Clone, PartialEq)]
// pub enum ContainerKind {
//     /// Plain struct field — part of parent group (not a child)
//     InlineField,
//     /// Dynamic collection (Vec, List etc) — elements are in child group
//     Collection,
//     /// Owning pointer (Box, unique_ptr) — pointee is in child group
//     OwnedPtr,
//     /// Tagged union/enum — active variant payload is in child group
//     Variant,
// }

// /// Describes what a function call does to groups
// #[derive(Debug, Clone)]
// pub struct GroupEffect {
//     /// which argument index provides the group
//     pub arg_index: usize,
//     /// the path within that argument's group that gets mutated
//     pub path:      GroupPath,
//     /// is it a mutation (invalidates child groups)?
//     pub mutates:   bool,
// }

/// An instruction with ownership/borrow information attached
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Inst {
    pub point: ProgramPoint,
    pub kind: InstKind,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InstKind {
    /// let x: T = expr
    Assign {
        ty: TypeSpec,
        name: Ident,
        value: Expr,
        is_move: bool, // is value being moved in?

        // group:   Option<GroupPath>, // which group this variable belongs to
    },
    // RawValue {
    //     name: String,
    //     value: Expr,
    //     is_move: bool
    // },
    /// x = expr (reassign)
    Store {
        name: Ident,
        value: Expr,
        is_move: bool,
        is_init: bool
    },
    /// x = expr (reassign)
    ReStore {
        name: Ident,
        to_mut: Expr,
        val: Expr,
        is_move: bool,
        is_init: bool
    },
    /// &x — create shared borrow
    BorrowShared {
        result: Ident, // variable holding the borrow
        lender: Ident, // variable being borrowed
        loan_id: LoanId,

        // group:   GroupPath, // which group the borrow is into
    },
    /// &mut x — create mutable borrow
    BorrowMut {
        result: Ident,
        lender: Ident,
        loan_id: LoanId,

        // group:       GroupPath,   // which group the borrow is into
        // mutates_group: GroupPath, // which group gets mutated (may differ)
    },
    /// use of a variable — read
    Use { used: Ident, by: Ident },
    /// move of a variable — invalidates it
    Move { from: Ident, to: Ident },
    /// explicit drop
    Drop { name: Ident },
    /// function call with ownership info per arg
    Call {
        func: Ident,
        args: Vec<(Expr, ArgOwnership)>,
        result: Option<Ident>,

        // group_effects: Vec<GroupEffect>, // what groups this call mutates
    },
}

/// How a function argument is passed
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ArgOwnership {
    Owned,     // value is moved into function
    SharedRef, // &T — shared borrow passed
    MutRef,    // &mut T — mutable borrow passed
    Copy,      // T: Copy — value is copied
}

/// Borrow checker errors
#[derive(Debug)]
#[allow(dead_code)]
pub enum BorrowError {
    UseAfterMove {
        var: Ident,
        moved_at: ProgramPoint,
        used_at: ProgramPoint,
    },
    MutBorrowWhileBorrowed {
        var: Ident,
        at: ProgramPoint,
    },
    SharedBorrowWhileMutablyBorrowed {
        var: Ident,
        at: ProgramPoint,
    },
    BorrowOutlivesLender {
        lender: Ident,
        borrower: Ident,
        loan_id: LoanId,
    },
    UseOfUninitialized {
        var: Ident,
        at: ProgramPoint,
    },

    // /// Used a reference into a child group after the parent group was mutated
    // UseOfInvalidatedGroup {
    //     path: GroupPath,
    //     at:   ProgramPoint,
    // },
    // /// Tried to use aliased references that violate isolation
    // IsolationViolation {
    //     var_a: String,
    //     var_b: String,
    //     at:    ProgramPoint,
    // },
}

/// CFG extended with borrow checking information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CFG {
    pub name: String,
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub exits: Vec<BlockId>,

    // all loans in the function
    pub loans: Vec<Loan>,

    // live ranges for all variables
    pub live_ranges: HashMap<String, LiveRange>,
}

// add to cfg_resourses.rs
// pub fn expr_uses(expr: &Expr) -> Vec<String> {
//     use crate::parser::ExprKind;
//     match &expr.kind {
//         ExprKind::Ident { ident, .. } => vec![ident.ident.clone()],
//         ExprKind::Binary { lhs, rhs, .. } => {
//             let mut uses = expr_uses(lhs);
//             uses.extend(expr_uses(rhs));
//             uses
//         }
//         ExprKind::Unary { rhs, .. } => expr_uses(rhs),
//         ExprKind::ArgBuffer { args, .. } => args.iter().flat_map(expr_uses).collect(),
//         ExprKind::ObjAssess { obj, fields } => {
//             let mut uses = expr_uses(obj);
//             for f in fields {
//                 uses.extend(expr_uses(f));
//             }
//             uses
//         }
//         _ => vec![],
//     }
// }

impl CFG {
    pub fn new(name: String) -> Self {
        Self {
            name,
            blocks: vec![],
            entry: BlockId(0),
            exits: vec![],
            loans: vec![],
            live_ranges: HashMap::new(),
        }
    }

    /// allocate a new basic block and return its id
    pub fn new_block(&mut self, label: impl Into<String>) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(BasicBlock::new(id, label));
        id
    }

    /// get a block by id
    pub fn block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id.0]
    }

    pub fn _block_mut(&mut self, id: BlockId) -> &mut BasicBlock {
        &mut self.blocks[id.0]
    }

    /// add an instruction to a block
    pub fn push_inst(&mut self, block: BlockId, inst: Inst) {
        self.blocks[block.0].insts.push(inst);
    }

    /// set the terminator of a block and wire up edges
    pub fn terminate(&mut self, block: BlockId, term: Terminator) {
        // wire successor edges
        match &term {
            Terminator::Jump(succ) => {
                let succ = *succ;
                self.blocks[block.0].succs.push(succ);
                self.blocks[succ.0].preds.push(block);
            }
            Terminator::Branch {
                true_bb, false_bb, ..
            } => {
                let (t, f) = (*true_bb, *false_bb);
                self.blocks[block.0].succs.push(t);
                self.blocks[block.0].succs.push(f);
                self.blocks[t.0].preds.push(block);
                self.blocks[f.0].preds.push(block);
            }
            Terminator::Return(_) | Terminator::Unreachable => {
                self.exits.push(block);
            }
        }
        self.blocks[block.0].terminator = Some(term);
    }

    pub fn reachable(&self) -> Vec<BlockId> {
        let mut visited = vec![false; self.blocks.len()];
        let mut queue = vec![self.entry];
        let mut result = vec![];

        while let Some(id) = queue.pop() {
            if visited[id.0] {
                continue;
            }
            visited[id.0] = true;
            result.push(id);
            for &succ in &self.block(id).succs {
                queue.push(succ);
            }
        }
        result
    }

    /// compute live ranges for all variables
    // pub fn compute_liveness(&mut self) {
    //     // compute use_set and def_set per block
    //     let mut use_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();
    //     let mut def_sets: HashMap<BlockId, HashSet<String>> = HashMap::new();

    //     for block in &self.blocks {
    //         let mut use_set = HashSet::new();
    //         let mut def_set = HashSet::new();

    //         for inst in &block.insts {
    //             match &inst.kind {
    //                 InstKind::Assign { name, value, .. } | InstKind::Store { name, value, .. } => {
    //                     for used in expr_uses(value) {
    //                         if !def_set.contains(&used) {
    //                             use_set.insert(used);
    //                         }
    //                     }
    //                     def_set.insert(name.clone());
    //                 }
    //                 InstKind::Use { used: name, .. } | InstKind::Move { from: name, .. } => {
    //                     if !def_set.contains(name) {
    //                         use_set.insert(name.clone());
    //                     }
    //                 }
    //                 InstKind::BorrowShared { lender, result, .. }
    //                 | InstKind::BorrowMut { lender, result, .. } => {
    //                     if !def_set.contains(lender) {
    //                         use_set.insert(lender.clone());
    //                         println!("Hey...{}", lender);
    //                     }
    //                     def_set.insert(result.clone());
    //                 }
    //                 InstKind::Drop { name } => {
    //                     def_set.insert(name.clone());
    //                 }
    //                 InstKind::Call { args, result, .. } => {
    //                     for (expr, _) in args {
    //                         for used in expr_uses(expr) {
    //                             if !def_set.contains(&used) {
    //                                 use_set.insert(used);
    //                             }
    //                         }
    //                     }
    //                     if let Some(r) = result {
    //                         def_set.insert(r.clone());
    //                     }
    //                 }
    //             }
    //         }

    //         use_sets.insert(block.id, use_set);
    //         def_sets.insert(block.id, def_set);
    //     }

    //     // fixed-point backward iteration
    //     let mut live_in: HashMap<BlockId, HashSet<String>> = HashMap::new();
    //     let mut live_out: HashMap<BlockId, HashSet<String>> = HashMap::new();

    //     loop {
    //         let mut changed = false;
    //         for block in self.blocks.iter().rev() {
    //             // live_out = union of live_in of successors
    //             let mut new_out = HashSet::new();
    //             for &succ in &block.succs {
    //                 if let Some(s) = live_in.get(&succ) {
    //                     new_out.extend(s.iter().cloned());
    //                 }
    //             }
    //             // live_in = use ∪ (live_out - def)
    //             let mut new_in = use_sets[&block.id].clone();
    //             for v in &new_out {
    //                 if !def_sets[&block.id].contains(v) {
    //                     new_in.insert(v.clone());
    //                 }
    //             }
    //             if new_in != *live_in.get(&block.id).unwrap_or(&Default::default())
    //                 || new_out != *live_out.get(&block.id).unwrap_or(&Default::default())
    //             {
    //                 changed = true;
    //             }
    //             live_in.insert(block.id, new_in);
    //             live_out.insert(block.id, new_out);
    //         }
    //         if !changed {
    //             break;
    //         }
    //     }

    //     // store live_in/live_out on blocks and build live ranges
    //     for block in &mut self.blocks {
    //         if let Some(li) = live_in.get(&block.id) {
    //             for v in li {
    //                 block
    //                     .state_in
    //                     .entry(v.clone())
    //                     .or_insert(OwnershipState::Owned);
    //             }
    //         }
    //         if let Some(lo) = live_out.get(&block.id) {
    //             for v in lo {
    //                 block
    //                     .state_out
    //                     .entry(v.clone())
    //                     .or_insert(OwnershipState::Owned);
    //             }
    //         }
    //     }

    //     // build live ranges from inst walk
    //     for block in &self.blocks {
    //         for inst in &block.insts {
    //             match &inst.kind {
    //                 InstKind::Assign { name, .. } | InstKind::Store { name, .. } => {
    //                     self.live_ranges.entry(name.clone()).or_insert(LiveRange {
    //                         name: name.clone(),
    //                         defined: inst.point,
    //                         uses: vec![],
    //                         last_use: inst.point,
    //                     });
    //                 }
    //                 InstKind::Use { used: name, .. } | InstKind::Move { from: name, .. } => {
    //                     if let Some(lr) = self.live_ranges.get_mut(name) {
    //                         lr.uses.push(inst.point);
    //                         if inst.point > lr.last_use {
    //                             lr.last_use = inst.point;
    //                         }
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    // // Group analysis
    // /// Assign group paths to all variables
    // pub fn compute_groups(&mut self, type_info: &TypeEnvironment) {
    //     let mut group_counter = 0;

    //     for block in &mut self.blocks {
    //         for inst in &block.insts {
    //             match &inst.kind {
    //                 InstKind::Assign { name, ty, group, .. } => {
    //                     if group.is_none() {
    //                         // each local variable gets its own root group
    //                         let gid = GroupId(group_counter);
    //                         group_counter += 1;
    //                         block.group_in.insert(
    //                             name.ident.clone(),
    //                             GroupPath::Root(gid),
    //                         );
    //                     }
    //                 }
    //                 InstKind::BorrowShared { result, lender, group, .. } |
    //                 InstKind::BorrowMut   { result, lender, group, .. } => {
    //                     // borrower gets the same group path as lender
    //                     block.group_in.insert(result.clone(), group.clone());
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    // /// Multiple mutable refs to same group = OK
    // /// Refs to child groups after mutation = ERROR
    // pub fn check_group_borrowing(&self) -> Vec<BorrowError> {
    //     let mut errors = vec![];

    //     for block_id in self.reachable() {
    //         let block = self.block(block_id);
    //         let mut invalidated_paths: Vec<GroupPath> = vec![];

    //         for inst in &block.insts {
    //             // check uses against invalidated paths
    //             match &inst.kind {
    //                 InstKind::Use { used, .. } => {
    //                     if let Some(path) = self.group_of(used, block_id) {
    //                         if Self::is_invalidated(&path, &invalidated_paths) {
    //                             errors.push(BorrowError::UseOfInvalidatedGroup {
    //                                 path,
    //                                 at: inst.point,
    //                             });
    //                         }
    //                     }
    //                 }

    //                 // mutable borrow — invalidates child groups of lender's group
    //                 // but NOT the lender's group itself (key difference from Rust)
    //                 InstKind::BorrowMut { lender, group, .. } => {
    //                     // invalidate all child groups of this group
    //                     // (not the group itself — multiple mut refs to same group OK)
    //                     self.invalidate_children(group, &mut invalidated_paths);
    //                 }

    //                 InstKind::Call { group_effects, .. } => {
    //                     for effect in group_effects {
    //                         if effect.mutates {
    //                             // mutation invalidates child groups
    //                             self.invalidate_children(
    //                                 &effect.path,
    //                                 &mut invalidated_paths,
    //                             );
    //                         }
    //                     }
    //                 }

    //                 _ => {}
    //             }

    //             // expire invalidations based on NLL liveness
    //             invalidated_paths.retain(|path| {
    //                 // keep if any variable still using this path is live
    //                 self.any_live_ref_to_path(path, inst.point)
    //             });
    //         }
    //     }

    //     errors
    // }

    // fn is_invalidated(path: &GroupPath, invalidated: &[GroupPath]) -> bool {
    //     invalidated.iter().any(|inv| path == inv || path.is_child_of(inv))
    // }

    // fn invalidate_children(&self, parent: &GroupPath, invalidated: &mut Vec<GroupPath>) {
    //     // add all known child group paths of parent to invalidated list
    //     for block in &self.blocks {
    //         for (_, path) in &block.group_in {
    //             if path.is_child_of(parent) && !invalidated.contains(path) {
    //                 invalidated.push(path.clone());
    //             }
    //         }
    //     }
    // }

    // fn any_live_ref_to_path(&self, path: &GroupPath, point: ProgramPoint) -> bool {
    //     // check if any variable pointing to this path is still live
    //     if let Some(live) = self.live_at.get(&point) {
    //         for var in live {
    //             if let Some(var_path) = self.group_of(var, point.block) {
    //                 if &var_path == path || var_path.is_child_of(path) {
    //                     return true;
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }

    // fn group_of(&self, var: &str, block: BlockId) -> Option<GroupPath> {
    //     self.blocks[block.0].group_in.get(var).cloned()
    // }

    // // Check group borrowing rules — replaces _check_borrow_conflicts
    // // pub fn check_group_borrowing(&self) -> Vec<BorrowError> {
    // //     let mut errors = vec![];

    // //     for block in &self.blocks {
    // //         // track which group paths are currently invalidated
    // //         let mut invalidated: Vec<GroupPath> = vec![];

    // //         for inst in &block.insts {
    // //             // check if this instruction uses an invalidated group
    // //             let used_groups = self.groups_used_by_inst(inst);
    // //             for used in &used_groups {
    // //                 if invalidated.iter().any(|inv| used == inv || used.is_child_of(inv)) {
    // //                     errors.push(BorrowError::UseOfInvalidatedGroup {
    // //                         path: used.clone(),
    // //                         at:   inst.point,
    // //                     });
    // //                 }
    // //             }

    // //             // check if this instruction mutates a group
    // //             // if so, invalidate all child groups of the mutated group
    // //             if let InstKind::Call { group_effects, .. } = &inst.kind {
    // //                 for effect in group_effects {
    // //                     if effect.mutates {
    // //                         // find all variables in child groups of effect.path
    // //                         // and add them to invalidated
    // //                         for (var, path) in &block.group_in {
    // //                             if path.is_child_of(&effect.path) {
    // //                                 invalidated.push(path.clone());
    // //                             }
    // //                         }
    // //                     }
    // //                 }
    // //             }
    // //         }
    // //     }

    // //     errors
    // // }

    // fn groups_used_by_inst(&self, inst: &Inst) -> Vec<GroupPath> {
    //     // extract which group paths are accessed by this instruction
    //     match &inst.kind {
    //         InstKind::Use { used, .. } => {
    //             // look up the group of `used`
    //             self.lookup_group(&used.ident, inst.point.block)
    //                 .into_iter().collect()
    //         }
    //         InstKind::BorrowShared { lender, group, .. } |
    //         InstKind::BorrowMut   { lender, group, .. } => {
    //             vec![group.clone()]
    //         }
    //         _ => vec![],
    //     }
    // }

    // fn lookup_group(&self, var: &str, block: BlockId) -> Option<GroupPath> {
    //     self.blocks[block.0].group_in.get(var).cloned()
    // }

    pub fn compute_liveness(&mut self) {
        for block in &self.blocks {
            for inst in &block.insts {
                match &inst.kind {
                    InstKind::Assign { name, .. }
                    | InstKind::Store { name, .. }
                    | InstKind::ReStore { name, .. } => {
                        self.live_ranges.entry(name.ident.clone()).or_insert(LiveRange {
                            name: name.ident.clone(),
                            defined: inst.point,
                            uses: vec![],
                            last_use: inst.point,
                        });
                    }
                    InstKind::Move { from: name, .. } => {
                        if let Some(lr) = self.live_ranges.get_mut(&name.ident) {
                            lr.uses.push(inst.point);
                            if inst.point > lr.last_use {
                                lr.last_use = inst.point;
                            }
                        }
                    }
                    InstKind::Use { used, by } => {
                        // `used` is read — update its liveness
                        if let Some(lr) = self.live_ranges.get_mut(&used.ident) {
                            lr.uses.push(inst.point);
                            if inst.point > lr.last_use {
                                lr.last_use = inst.point;
                            }
                        }
                        // `by` is defined here — register it
                        self.live_ranges.entry(by.ident.clone()).or_insert(LiveRange {
                            name: by.ident.clone(),
                            defined: inst.point,
                            uses: vec![],
                            last_use: inst.point,
                        });
                    }
                    // ← ADD THIS — borrow is a use of the lender
                    InstKind::BorrowShared { result, lender, .. }
                    | InstKind::BorrowMut { result, lender, .. } => {
                        // lender is used here
                        if let Some(lr) = self.live_ranges.get_mut(&lender.ident) {
                            lr.uses.push(inst.point);
                            if inst.point > lr.last_use {
                                lr.last_use = inst.point;
                            }
                        }
                        // borrower is defined here — register it
                        self.live_ranges.entry(result.ident.clone()).or_insert(LiveRange {
                            name: result.ident.clone(),
                            defined: inst.point,
                            uses: vec![],
                            last_use: inst.point,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// check for use after move
    pub fn check_use_after_move(&self) -> Vec<BorrowError> {
        let mut errors = vec![];
        // per-block move state — propagated across edges
        let mut block_moved: HashMap<BlockId, HashMap<String, ProgramPoint>> = HashMap::new();

        // process in topological order (BFS from entry)
        let order = self.reachable();

        for block_id in order {
            let block = self.block(block_id);

            // merge move state from all predecessors
            let mut moved: HashMap<String, ProgramPoint> = HashMap::new();
            for &pred in &block.preds {
                if let Some(pred_moved) = block_moved.get(&pred) {
                    for (var, point) in pred_moved {
                        // if moved in ANY predecessor — flag it
                        moved.insert(var.clone(), *point);
                    }
                }
            }

            for inst in &block.insts {
                match &inst.kind {
                    InstKind::Move { from, to } => {
                        moved.insert(from.ident.clone(), inst.point);
                        // `to` is now valid — remove from moved if it was there
                        moved.remove(&to.ident);
                    }
                    InstKind::Assign { name, .. } => {
                        // reassignment reinitializes — clears moved status
                        moved.remove(&name.ident);
                    }
                    // CORRECT — Use is a copy, source stays valid
                    InstKind::Use { used, .. } => {
                        if let Some(&move_point) = moved.get(&used.ident) {
                            errors.push(BorrowError::UseAfterMove {
                                var: used.clone(),
                                moved_at: move_point,
                                used_at: inst.point,
                            });
                        }
                        // no move tracking update — source remains valid after a copy
                    }
                    _ => {}
                }
            }

            block_moved.insert(block_id, moved);
        }
        errors
    }

    pub fn check_borrow_conflicts(&self) -> Vec<BorrowError> {
        let mut errors = vec![];
        let mut block_active: HashMap<BlockId, HashMap<String, Vec<(LoanId, BorrowKind)>>> =
            HashMap::new();

        for block_id in self.reachable() {
            let block = self.block(block_id);

            // merge active borrows from predecessors
            let mut active: HashMap<String, Vec<(LoanId, BorrowKind)>> = HashMap::new();
            for &pred in &block.preds {
                if let Some(pred_active) = block_active.get(&pred) {
                    for (var, borrows) in pred_active {
                        active
                            .entry(var.clone())
                            .or_default()
                            .extend(borrows.iter().cloned());
                    }
                }
            }

            for inst in &block.insts {
                // expire borrows whose loan has ended before this point
                active.retain(|_, borrows| {
                    borrows.retain(|(loan_id, _)| {
                        if let Some(loan) = self.loans.iter().find(|l| l.id == *loan_id) {
                            loan.expires >= inst.point // keep if not yet expired
                        } else {
                            false
                        }
                    });
                    !borrows.is_empty()
                });

                match &inst.kind {
                    InstKind::BorrowShared {
                        lender, loan_id, ..
                    } => {
                        let borrows = active.entry(lender.ident.clone()).or_default();

                        if borrows.iter().any(|(_, k)| *k == BorrowKind::Mutable) {
                            errors.push(BorrowError::SharedBorrowWhileMutablyBorrowed {
                                var: lender.clone(),
                                at: inst.point,
                            });
                        }
                        borrows.push((*loan_id, BorrowKind::Shared));
                    }
                    InstKind::BorrowMut {
                        lender, loan_id, ..
                    } => {
                        let borrows = active.entry(lender.ident.clone()).or_default();

                        if !borrows.is_empty() {
                            errors.push(BorrowError::MutBorrowWhileBorrowed {
                                var: lender.clone(),
                                at: inst.point,
                            });
                        }
                        borrows.push((*loan_id, BorrowKind::Mutable));
                    }
                    _ => {}
                }
            }

            block_active.insert(block_id, active);
        }
        errors
    }

    pub fn check_loan_lifetimes(&self) -> Vec<BorrowError> {
        let mut errors = vec![];

        for loan in &self.loans {
            if let Some(lender_range) = self.live_ranges.get(&loan.lender.ident) {
                // only error if borrower is used AFTER lender dies
                // i.e. loan.expires is strictly greater than lender's last_use
                // AND lender.last_use is before the borrow creation
                // (meaning lender died before it could even be borrowed)
                if loan.expires > lender_range.last_use && lender_range.last_use < loan.created {
                    errors.push(BorrowError::BorrowOutlivesLender {
                        lender: loan.lender.clone(),
                        borrower: loan.borrower.clone(),
                        loan_id: loan.id,
                    });
                }
            }
        }
        errors
    }

    // pub fn finalize_loan_lifetimes(&mut self) {
    //     // Step 1 — compute block end points upfront
    //     // no borrow of self.loans here, only self.blocks
    //     let block_ends: HashMap<BlockId, ProgramPoint> = self
    //         .blocks
    //         .iter()
    //         .map(|block| {
    //             let end = ProgramPoint {
    //                 block: block.id,
    //                 inst: block.insts.len().saturating_sub(1),
    //             };
    //             (block.id, end)
    //         })
    //         .collect();

    //     // Step 2 — now iterate loans mutably
    //     // block_ends is a separate owned value — no conflict
    //     for loan in &mut self.loans {
    //         let block_end = block_ends
    //             .get(&loan.created.block)
    //             .copied()
    //             .unwrap_or(loan.created);

    //         loan.expires = if let Some(lr) = self.live_ranges.get(&loan.borrower) {
    //             if lr.uses.is_empty() {
    //                 block_end
    //             } else {
    //                 lr.last_use.max(loan.created)
    //             }
    //         } else {
    //             block_end
    //         };
    //     }
    // }

    pub fn finalize_loan_lifetimes(&mut self) {
        let block_ends: HashMap<BlockId, ProgramPoint> = self
            .blocks
            .iter()
            .map(|block| {
                (
                    block.id,
                    ProgramPoint {
                        block: block.id,
                        inst: block.insts.len().saturating_sub(1),
                    },
                )
            })
            .collect();

        for loan in &mut self.loans {
            let block_end = block_ends
                .get(&loan.created.block)
                .copied()
                .unwrap_or(loan.created);

            loan.expires = if let Some(lr) = self.live_ranges.get(&loan.borrower.ident) {
                if lr.uses.is_empty() {
                    // NLL: never used after declaration
                    // expire at creation point — borrow ends immediately
                    loan.created // ← NLL: not block_end
                } else {
                    lr.last_use // ← NLL: expire at last use
                }
            } else {
                block_end
            };
        }
    }
}
