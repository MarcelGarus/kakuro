use crate::{
    game::{Input, Output, Solution, Value},
    log::log,
};
use itertools::Itertools;
use std::collections::HashSet;

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait InputExt {
    fn is_possible_solution(&self, attempt: &Game) -> bool;
}
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &Game) -> bool {
        for constraint in self.constraints.iter() {
            let cells = constraint.cells.iter().map(|b| attempt[*b]).collect_vec();
            let digits = cells.into_iter().filter_map(|it| it).collect_vec();
            let len = digits.len();

            let digits = digits.into_iter().collect::<HashSet<_>>();
            if digits.len() < len {
                return false; // A digit appears twice.
            }

            let sum: Value = digits.iter().sum();
            if sum == 0 {
                continue; // No cells filled in yet; we assume the attempt is possible.
            }

            let unused_digits = (1..=9u8)
                .collect::<HashSet<_>>()
                .difference(&digits)
                .map(|it| *it)
                .collect::<HashSet<Value>>();
            let is_sum_reachable = unused_digits
                .into_iter()
                .combinations(constraint.cells.len() - digits.len())
                .map(|additional_digits| sum + additional_digits.into_iter().sum::<Value>())
                .any(|possible_sum| possible_sum == constraint.sum);
            if !is_sum_reachable {
                return false;
            }
        }
        return true;
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![None; input.num_cells];
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
            .collect::<String>()
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
