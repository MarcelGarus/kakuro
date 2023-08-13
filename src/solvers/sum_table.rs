use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::collections::HashMap;

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait ConstraintExt {
    fn is_satisfied_by(&self, attempt: &Game) -> bool;
}
impl ConstraintExt for Constraint {
    fn is_satisfied_by(&self, attempt: &Game) -> bool {
        let cells: ArrayVec<Option<Value>, 9> = self.cells.iter().map(|b| attempt[*b]).collect();
        let digits: ArrayVec<Value, 9> = cells.into_iter().filter_map(|it| it).collect();

        let mut used_digits_bitmask = 0u16;
        for digit in &digits {
            if used_digits_bitmask & (1 << (digit - 1)) == 1 {
                return false; // A digit appears twice.
            } else {
                used_digits_bitmask |= 1 << (digit - 1);
            }
        }

        return is_sum_reachable(
            used_digits_bitmask,
            self.cells.len() - digits.len(),
            self.sum,
        );
    }
}

pub fn solve(input: &Input) -> Output {
    init_sum_table();

    let mut attempt = vec![None; input.num_cells];
    let mut solutions = vec![];
    let mut affected_constraints = HashMap::new();
    for (i, constraint) in input.constraints.iter().enumerate() {
        for cell in &constraint.cells {
            affected_constraints.entry(*cell).or_insert(vec![]).push(i);
        }
    }
    solve_rec(input, &affected_constraints, &mut attempt, &mut solutions);
    solutions
}

fn solve_rec(
    input: &Input,
    affected_constraints: &HashMap<usize, Vec<usize>>,
    attempt: &mut Game,
    solutions: &mut Vec<Solution>,
) {
    log!("Evaluating attempt {}", format_game(attempt));

    let first_empty_cell_index = attempt.iter().position(|it| it.is_none());
    if let Some(index) = first_empty_cell_index {
        'candidates: for i in 1..=9 {
            attempt[index] = Some(i);
            for constraint_index in &affected_constraints[&index] {
                let constraint = &input.constraints[*constraint_index];
                if !constraint.is_satisfied_by(attempt) {
                    continue 'candidates;
                }
            }
            solve_rec(input, affected_constraints, attempt, solutions);
        }
        attempt[index] = None;
    } else {
        solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
    }
}

// A lookup table where you can look up the total number of digits as well as
// the existing digits to find the minimum and maximum reachable sum.

fn init_sum_table() {
    SUM_TABLE[0][0][0];
}

fn is_sum_reachable(
    existing_digits_bitmask: u16,
    num_additional_digits: usize,
    sum: Value,
) -> bool {
    SUM_TABLE[existing_digits_bitmask as usize][num_additional_digits][sum as usize]
}

lazy_static! {
    // first, outer array: which digits have already been used
    // second array: how many digits still need to be filled in
    // third, inner array: the total sum to reach
    // value: whether the sum is reachable
    static ref SUM_TABLE: [[[bool; 46]; 10]; 1 << 9] = calculate_sum_table();
}

fn calculate_sum_table() -> [[[bool; 46]; 10]; 1 << 9] {
    log!("Calculating sum table.");
    let mut table = [[[false; 46]; 10]; 1 << 9];

    for existing_digits_bitmask in 0b000_000_000_u16..=0b111_111_111_u16 {
        let existing_digits = (0u8..9)
            .filter_map(|i| {
                if existing_digits_bitmask & (1 << i as u16) == 0 {
                    None
                } else {
                    Some(i + 1)
                }
            })
            .collect_vec();
        let existing_sum: Value = existing_digits.iter().sum();

        for num_additional_digits in 0..=(9 - existing_digits.len()) {
            let unused_digits = (1u8..=9)
                .filter(|it| !existing_digits.contains(it))
                .collect_vec();
            let reachable_sums = unused_digits
                .iter()
                .combinations(num_additional_digits)
                .map(|additional_digits| {
                    existing_sum + additional_digits.into_iter().sum::<Value>()
                })
                .collect_vec();

            // log!("With {:b} in use ({:?}, sum {}) and {} more to fill in ({} out of {:?}), we can reach ({:?})",
            //     existing_digits_bitmask, existing_digits, existing_sum, num_additional_digits, num_additional_digits, unused_digits, reachable_sums);
            for target_sum in 0..=45 {
                let is_reachable = reachable_sums.contains(&target_sum);
                table[existing_digits_bitmask as usize][num_additional_digits]
                    [target_sum as usize] = is_reachable;
            }
        }
    }

    log!("Sum table calculated.");
    table
}

fn format_game(game: &Game) -> String {
    game.iter()
        .map(|cell| match cell {
            Some(digit) => format!("{}", digit),
            None => "-".to_string(),
        })
        .join("")
}
