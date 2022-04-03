use std::collections::HashSet;

use crate::game::{Input, Output, Solution, Value};

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait InputExt {
    fn is_possible_solution(&self, attempt: &Game) -> bool;
}
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &Game) -> bool {
        for constraint in self.constraints.iter() {
            let cell_values = constraint
                .cells
                .iter()
                .map(|b| attempt[*b])
                .collect::<HashSet<_>>();
            if cell_values.iter().any(|value| value.is_none()) {
                continue;
            } else if cell_values.len() != constraint.cells.len() {
                return false; // A number appears twice.
            } else {
                let sum = cell_values
                    .into_iter()
                    .map(|a| a.unwrap())
                    .reduce(|a, b| a + b)
                    .unwrap();
                if sum != constraint.sum {
                    return false;
                }
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
    println!(
        "Evaluating attempt {}",
        attempt
            .iter()
            .map(|cell| {
                match cell {
                    Some(number) => format!("{}", number),
                    None => "-".to_string(),
                }
            })
            .collect::<String>()
    );
    if !input.is_possible_solution(attempt) {
        return;
    }
    let first_free_index = attempt.iter().position(|it| it.is_none());
    if let Some(index) = first_free_index {
        for i in 1..=9 {
            attempt[index] = Some(i);
            solve_rec(input, attempt, solutions);
        }
        attempt[index] = None;
    } else {
        // This is a solution.
        solutions.push(attempt.iter().map(|it| it.unwrap()).collect());
    }
}
