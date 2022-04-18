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
//! - prioritize_no_set: Like prioritize, but without using HashSets to check
//!   for uniqueness.
//! - sum_reachable_no_set: Like sum_reachable, but without using HashSets to
//!   check for uniqueness.
//! - divide: This solver divides a big Kakuro into two smaller ones that are
//!   only connected with few constraints. All constraints except those
//!   connecting both parts still apply. Both smaller Kakuros are solved in
//!   isolation and the product of both result sets is filtered to the solutions
//!   also fulfilling the connecting constraints.
//! - combine_by_connecting_cells: Like divide, but to merge partial solutions,
//!   instead of considering the whole product of all solutions, we first group
//!   them by the connecting cells. We only look at all combinations of
//!   connecting cells. If we find a match, we then take the product of the
//!   actual solutions.
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
//! - separate divide and combine_by_sum
//! - combine first by sum, only then by actual numbers
//! - track the possibility wave like when solving by hand

pub mod combine_by_connecting_cells;
pub mod connections;
pub mod divide;
pub mod gradual;
pub mod lazy;
pub mod naive;
pub mod prioritize;
pub mod prioritize_no_set;
pub mod sum_reachable;
pub mod sum_reachable_no_set;
