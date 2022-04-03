//! This module contains different solvers to solve Kakuros:
//!
//! - naive: This solver considers all combinations of filling out cells. For
//!   each combination, it checks if it's a valid solution.
//! - gradual: This solver can evaluate partially filled out Kakuros. It fills
//!   cells with numbers one at a time and tries to find a valid combination by
//!   filling the cells one at a time and early-aborting if the Kakuro becomes
//!   invalid.
//! - gradual_early_abort: Like gradual, but also aborts if a sum cannot
//!   possibly be reached any longer. For example, if there's a constraint with
//!   the sum for the cells 4 _ 9, then the partially filled out game is already
//!   considered invalid.

pub mod gradual;
pub mod gradual_early_abort;
pub mod naive;
