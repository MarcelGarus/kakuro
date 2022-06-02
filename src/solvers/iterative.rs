use crate::{
    game::{Constraint, Input, Output, Value},
    log,
};
use itertools::Itertools;
use std::collections::HashMap;

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait ConstraintExt {
    fn is_satisfied_by(&self, attempt: &Game) -> bool;
}
impl ConstraintExt for Constraint {
    fn is_satisfied_by(&self, attempt: &Game) -> bool {
        let cells = self.cells.iter().map(|b| attempt[*b]).collect_vec();
        let digits = cells.into_iter().filter_map(|it| it).collect_vec();

        let mut seen = [false; 9];
        for digit in &digits {
            if seen[(digit - 1) as usize] {
                return false; // A digit appears twice.
            } else {
                seen[(digit - 1) as usize] = true;
            }
        }

        let sum: Value = digits.iter().sum();
        if sum == 0 {
            return true; // No cells filled in yet.
        }

        let unused_digits = (1..=9u8)
            .filter(|digit| !seen[(digit - 1) as usize])
            .collect_vec();
        let is_sum_reachable = unused_digits
            .into_iter()
            .combinations(self.cells.len() - digits.len())
            .map(|additional_digits| sum + additional_digits.into_iter().sum::<Value>())
            .any(|possible_sum| possible_sum == self.sum);
        return is_sum_reachable;
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![None; input.num_cells];
    let mut solutions = vec![];
    let mut affected_constraints = HashMap::new();
    for (i, constraint) in input.constraints.iter().enumerate() {
        for cell in &constraint.cells {
            affected_constraints.entry(*cell).or_insert(vec![]).push(i);
        }
    }

    let mut current_cell = 0;
    'outer: loop {
        if current_cell == input.num_cells {
            // No cell is free anymore. We have a solution.
            solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
            current_cell -= 1;
        } else {
            attempt[current_cell] = match attempt[current_cell] {
                None => Some(1),
                Some(9) => None,
                Some(i) => Some(i + 1),
            };

            log!(
                "Evaluating attempt {} (current is {})",
                attempt
                    .iter()
                    .map(|cell| {
                        match cell {
                            Some(digit) => format!("{}", digit),
                            None => "-".to_string(),
                        }
                    })
                    .join(""),
                current_cell
            );

            if attempt[current_cell].is_none() {
                if current_cell == 0 {
                    break;
                } else {
                    current_cell -= 1;
                    continue;
                }
            }

            for constraint_index in &affected_constraints[&current_cell] {
                let constraint = &input.constraints[*constraint_index];
                if !constraint.is_satisfied_by(&attempt) {
                    continue 'outer;
                }
            }
            current_cell += 1;
        }
    }

    solutions
}
