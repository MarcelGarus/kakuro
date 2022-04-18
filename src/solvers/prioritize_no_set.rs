use crate::{
    game::{Input, Output, Solution, Value},
    log::log,
};
use itertools::Itertools;

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait InputExt {
    fn is_possible_solution(&self, attempt: &Game) -> bool;
}
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &Game) -> bool {
        for constraint in self.constraints.iter() {
            let cells = constraint
                .cells
                .iter()
                .map(|b| attempt[*b])
                .collect::<Vec<_>>();
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
                continue; // No cells filled in yet.
            }

            let unused_digits = (1..=9u8)
                .filter(|digit| !seen[(digit - 1) as usize])
                .collect_vec();
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
            .join("")
    ));

    if !input.is_possible_solution(attempt) {
        return;
    }

    // For each cell, save how many partially filled constraints contain it.
    let mut cell_priorities = vec![0; input.num_cells];
    for constraint in &input.constraints {
        let is_partially_filled = constraint
            .cells
            .iter()
            .any(|index| attempt[*index].is_some());
        if is_partially_filled {
            for i in &constraint.cells {
                if attempt[*i].is_none() {
                    cell_priorities[*i] += 1;
                }
            }
        }
    }

    let cell_to_fill = cell_priorities
        .into_iter()
        .enumerate()
        .max_by_key(|(_, priority)| *priority)
        .and_then(|(cell, priority)| {
            if priority > 0 {
                // The cell is guaranteed to be empty because only the priority
                // of empty cells can be non-zero.
                Some(cell)
            } else {
                // No constraint contains a digit _and_ an empty cell. Just fill
                // the first empty cell.
                attempt.iter().position(|it| it.is_none())
            }
        });

    if let Some(cell) = cell_to_fill {
        for i in 1..=9 {
            attempt[cell] = Some(i);
            solve_rec(input, attempt, solutions);
        }
        attempt[cell] = None;
    } else {
        // This is a solution.
        solutions.push(attempt.iter().map(|it| it.unwrap()).collect());
    }
}
