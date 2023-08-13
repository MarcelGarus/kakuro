use crate::{
    game::{Input, Output, Solution, Value, Constraint},
    log,
};
use extension_trait::extension_trait;
use itertools::Itertools;
use std::collections::HashSet;

#[extension_trait]
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &[Option<Value>]) -> bool {
        self.constraints.iter().all(|constraint| constraint.is_possible_solution(attempt))
    }
}
#[extension_trait]
impl ConstraintExt for Constraint {
    fn is_possible_solution(&self, attempt: &[Option<Value>]) -> bool {
        let cells = self.cells.iter().map(|i| attempt[*i]).collect_vec();
        let digits = cells.into_iter().filter_map(|it| it).collect_vec();
        let unique_digits = digits.iter().collect::<HashSet<_>>();

        if unique_digits.len() < digits.len() {
            false // A digit appears twice.
        } else if digits.len() < self.cells.len() {
            true // Ignore partially filled out constraints.
        } else {
            digits.iter().sum::<Value>() != self.sum
        }
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt: Vec<Option<Value>> = vec![None; input.num_cells];
    let mut solutions = vec![];
    solve_rec(input, &mut attempt, &mut solutions);
    solutions
}

fn solve_rec(input: &Input, attempt: &mut Vec<Option<Value>>, solutions: &mut Vec<Solution>) {
    log!(
        "Evaluating attempt {}",
        attempt
            .iter()
            .map(|cell| {
                match cell {
                    Some(digit) => format!("{}", digit),
                    None => "-".to_string(),
                }
            })
            .join("")
    );

    if !input.is_possible_solution(attempt) {
        return;
    }

    let first_empty_cell_index = attempt.iter().position(|it| it.is_none());
    if let Some(index) = first_empty_cell_index {
        for i in 1..=9 {
            attempt[index] = Some(i);
            solve_rec(input, attempt, solutions);
        }
        attempt[index] = None;
    } else {
        // This is a solution.
        solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
    }
}
