use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use extension_trait::extension_trait;
use itertools::Itertools;

#[extension_trait]
impl ConstraintExt4 for Constraint {
    fn is_satisfied_by(&self, attempt: &Vec<Option<Value>>) -> bool {
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
    let mut affected_constraints = vec![vec![]; input.num_cells];
    for (i, constraint) in input.constraints.iter().enumerate() {
        for cell in &constraint.cells {
            affected_constraints[*cell].push(i);
        }
    }
    solve_rec(input, &affected_constraints, &mut attempt, &mut solutions);
    solutions
}

fn solve_rec(
    input: &Input,
    affected_constraints: &[Vec<usize>],
    attempt: &mut Vec<Option<Value>>,
    solutions: &mut Vec<Solution>,
) {
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

    let first_empty_cell_index = attempt.iter().position(|it| it.is_none());
    if let Some(index) = first_empty_cell_index {
        'candidates: for i in 1..=9 {
            attempt[index] = Some(i);
            for constraint_index in &affected_constraints[index] {
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
