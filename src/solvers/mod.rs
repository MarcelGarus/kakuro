//! This module contains different solvers to solve Kakuros:
//!
//! - naive: This solver constructs all combinations of filling out cells. For
//!   each combination, it checks if it's a valid solution.
//! - gradual: This solver can evaluate partially filled out Kakuros. It tries
//!   to find a valid combination by filling the cells one at a time and
//!   aborting if the Kakuro becomes invalid by having the same digit in a
//!   constraint twice or a filled constraint not having the right sum.
//! - sum_reachable: Like gradual, but also aborts if a constraints sum cannot
//!   possibly be reached any longer by filling out unused numbers.
//! - prioritize: Like sum_reachable, but doesn't just fill the cells in the
//!   arbitrary order that they were numbered in. Instead, it fills the first
//!   cell and then cells in rows and columns that already contain numbers.
//! - sum_reachable_no_set: Like sum_reachable, but without using HashSets to
//!   check for uniqueness.
//! - only_check_changes: Like sum_reachable_no_set, but it doesn't check the
//!   entire Kakuro after a digit was filled in. Instead, it only checks the
//!   affected constraints.
//! - divide: This solver divides a big Kakuro into two smaller ones that are
//!   only connected with few constraints. All constraints except those
//!   connecting both parts still apply. Both smaller Kakuros are solved in
//!   isolation and the product of both result sets is filtered to the solutions
//!   also fulfilling the connecting constraints.
//! - connecting_cells: Like divide, but to merge partial solutions, instead of
//!   considering the whole product of all solutions, we first group them by the
//!   connecting cells. We only look at all combinations of connecting cells. If
//!   we find a match, we then take the product of the actual solutions.
//! - lazy: Like divide, but the recursive solving communicates which parts of
//!   the board play a role in connecting constraints later on. This allows
//!   inner calls to not actually construct solutions that are equivalent in
//!   this regard.
//! - propagate_constraints: Like lazy, but instead of communicating to the
//!   partial solvers which connecting cells are important later on, we
//!   communicate the connecting constraints themselves and information about
//!   the minimum/maximum sum that one part should have. For example, if a
//!   connecting constraint with one cell in either part has a sum of 6, then
//!   the values in each part can only be between 1 and 5.
//! - solution_in_rc: Like propagate_constraints, but the quasi solutions are
//!   wrapped in a reference counter. This allows common subtrees of quasi
//!   solutions to share their memory, reducing allocations.
//! - fxhashmap: Like solution_in_rc, but change usages of `HashMap` to
//!   `FxHashMap` from the `rustc-hash` crate.
//!
//! Ideas:
//! - combine first by sum, only then by actual numbers
//! - track the possibility wave like when solving by hand

pub mod array_vec;
pub mod better_vecs;
pub mod connecting_cells;
pub mod divide;
pub mod earlier_anchor;
pub mod fxhashmap;
pub mod gradual;
pub mod iterative;
pub mod lazy;
pub mod naive;
pub mod only_check_changes;
pub mod prioritize;
pub mod propagate_constraints;
pub mod simpler_recursion_anchor;
pub mod solution_in_rc;
pub mod sum_reachable;
pub mod sum_reachable_no_set;
pub mod sum_table;
