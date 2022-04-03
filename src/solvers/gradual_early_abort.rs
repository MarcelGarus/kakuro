use std::collections::HashSet;

use itertools::Itertools;

use crate::game::{Input, Output, Solution, Value};

type Game = Vec<Cell>;
type Cell = Option<Value>;

impl Input {
    fn is_possible_solution(&self, attempt: &Game) -> bool {
        for constraint in self.constraints.iter() {
            //dbg!(&constraint);
            let cells = constraint
                .cells
                .iter()
                .map(|b| attempt[*b])
                .collect::<Vec<_>>();
            //dbg!(&cells);
            let numbers = cells.into_iter().filter_map(|it| it).collect::<Vec<_>>();

            if numbers.iter().collect::<HashSet<_>>().len() < numbers.len() {
                return false; // A number appears twice.
            }

            let numbers = numbers.into_iter().collect::<HashSet<_>>();
            let sum: Value = numbers.iter().sum();
            if sum == 0 {
                continue; // No cells filled in yet; we assume the game itself is possible.
            }

            let possible_digits = (1..=9u8)
                .collect::<HashSet<_>>()
                .difference(&numbers)
                .map(|it| *it)
                .collect::<HashSet<Value>>();
            let is_possible_to_reach_sum = possible_digits
                .into_iter()
                .combinations(constraint.cells.len() - numbers.len())
                .map(|additional_numbers| sum + additional_numbers.into_iter().sum::<Value>())
                .any(|possible_sum| possible_sum == constraint.sum);
            if !is_possible_to_reach_sum {
                return false;
            }
        }
        return true;
    }
}

pub fn solve(input: &Input) -> Output {
    // let mut attempt: Game = vec![4, 1, 2, 5, 9, 2, 4, 3, 2, 1]
    //     .into_iter()
    //     .map(|it| Some(it))
    //     .collect::<Vec<_>>();
    // while attempt.len() < input.num_cells {
    //     attempt.push(None);
    // }

    // //dbg!(input.is_possible_solution(&attempt));

    let mut attempt = vec![];
    for _ in 0..input.num_cells {
        attempt.push(None);
    }

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
