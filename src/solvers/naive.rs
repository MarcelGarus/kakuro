use crate::{
    game::{Input, Output},
    log::log,
};
use itertools::Itertools;

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![1; input.num_cells];
    let mut solutions = vec![];

    'search: loop {
        log(format!(
            "Evaluating attempt {}",
            attempt.iter().map(|digit| format!("{}", digit)).join("")
        ));

        if input.is_solution(&attempt) {
            solutions.push(attempt.clone());
        }

        // Increase attempt by one, interpreted as a single number.
        let mut i = attempt.len() - 1;
        loop {
            attempt[i] += 1;
            if attempt[i] == 10 {
                attempt[i] = 1;
                if i == 0 {
                    break 'search;
                }
                i -= 1;
            } else {
                break;
            }
        }
    }

    solutions
}
