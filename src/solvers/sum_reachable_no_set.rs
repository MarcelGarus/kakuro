use crate::{
    game::{Input, Output, Solution, Value, Constraint},
    log,
};
use extension_trait::extension_trait;
use itertools::Itertools;


#[extension_trait]
impl InputExt3 for Input {
    fn is_possible_solution(&self, attempt: &[Option<Value>]) -> bool {
        self.constraints.iter().all(|constraint| constraint.is_possible_solution(attempt))
    }
}
#[extension_trait]
impl ConstraintExt3 for Constraint {
    fn is_possible_solution(&self, attempt: &[Option<Value>]) -> bool {
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
        is_sum_reachable
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![None; input.num_cells];
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
        solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
    }
}
