use crate::{
    game::{Input, Output, Solution, Value},
    log::log,
};
use itertools::Itertools;
use std::collections::HashSet;

// A partially filled out Kakuro.
type Game = Vec<Option<Value>>;

trait InputExt {
    fn is_possible_solution(&self, attempt: &Game) -> bool;
}
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &Game) -> bool {
        for constraint in &self.constraints {
            let cells = constraint.cells.iter().map(|i| attempt[*i]).collect_vec();
            let digits = cells.into_iter().filter_map(|it| it).collect_vec();
            let unique_digits = digits.iter().collect::<HashSet<_>>();

            if unique_digits.len() < digits.len() {
                return false; // A digit appears twice.
            } else if digits.len() < constraint.cells.len() {
                continue; // Ignore partially filled out constraints.
            } else if digits.iter().sum::<Value>() != constraint.sum {
                return false;
            }
        }
        return true;
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt: Game = vec![None; input.num_cells];
    let mut solutions = vec![];
    solve_rec(input, &mut attempt, &mut solutions);
    solutions
}

fn solve_rec(input: &Input, attempt: &mut Game, solutions: &mut Vec<Solution>) {
    log(format!(
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
    ));

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
