//! This module contains different solvers to solve Kakuros:
//!
//! - naive: This solver considers all combinations of filling out cells. For
//!   each combination, it checks if it's a valid solution.
//! - gradual: This solver can evaluate partially filled out Kakuros. It fills
//!   cells with numbers one at a time and tries to find a valid combination by
//!   filling the cells one at a time and early-aborting if the Kakuro becomes
//!   invalid.
//! - early_abort: Like gradual, but also aborts if a sum cannot possibly be
//!   reached any longer. For example, if there's a constraint with the sum for
//!   the cells 4 _ 9, then the partially filled out game is already considered
//!   invalid.
//! - prioritize: Like early_abort, but doesn't just fill the cells in the
//!   arbitrary order that they were numbered in. Instead, it fills the first
//!   cell and then cells in rows and columns that already contain numbers.
//! - divide: On a large board, it doesn't make sense to even attempt to look at
//!   all combinations. Instead, this solver divides a big Kakuro into two
//!   smaller ones that are only connected with few constraints. For those
//!   smaller games, all constraints still apply, except those spanning both.
//!   Both smaller Kakuros are solved in isolation and the product of both
//!   result sets is filtered to the solutions also fulfilling the connecting
//!   constraints.
//! - lazy: Like divide, but the recursive solving communicates which parts of
//!   the board play a role in connecting constraints later on. This allows
//!   inner calls to not actually construct solutions that are equivalent in
//!   this regard.
//! - connections: Like lazy, but instead of communicating which cells connect
//!   the parts, the recursive solving communicates the connecting constraints
//!   themselves and information about the minimum/maximum sum that one part
//!   should have. For example, if a connecting constraint with one cell in
//!   either part has a sum of 6, then the values in each part can only be
//!   between 1 and 5.
//!
//! Ideas:
//! - first go by sum
//! - track the possibility wave like when solving by hand

pub mod connections;
pub mod divide;
pub mod gradual;
pub mod lazy;
pub mod naive;
pub mod prioritize;
pub mod sum_reachable;
