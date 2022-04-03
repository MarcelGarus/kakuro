//! This module contains different solvers to solve Kakuros:
//!
//! - naive: This solver considers all combinations of filling out cells. For
//!   each combination, it checks if it's a valid solution.
//! - gradual: This solver can evaluate partially filled out Kakuros. It fills
//!   cells with numbers one at a time and tries to find a valid combination by
//!   filling the cells one at a time and early-aborting if the Kakuro becomes
//!   invalid.

pub mod gradual;
pub mod naive;
