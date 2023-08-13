use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use extension_trait::extension_trait;
use itertools::Itertools;

macro_rules! check_additional {
    ($nesting:expr, $digits_so_far:expr, $target_digits:expr, $sum_so_far:expr, $target_sum:expr, $seen:expr, $block:block) => {
        if $sum_so_far <= $target_sum {
            if $digits_so_far == $target_digits {
                if $sum_so_far == $target_sum {
                    return true;
                }
            } else {
                for a in ($nesting + 1)..=9 {
                    if $seen[a - 1] {
                        continue;
                    }
                    $digits_so_far += 1;
                    $sum_so_far += a;
                    $seen[a - 1] = true;
                    $block;
                    $digits_so_far -= 1;
                    $sum_so_far -= a;
                    $seen[a - 1] = false;
                }
            }
        }
    };
}

#[extension_trait]
impl ConstraintExt5 for Constraint {
    fn is_satisfied_by(&self, attempt: &Vec<Option<Value>>) -> bool {
        let mut seen = [false; 9];
        let mut sum = 0usize;

        for digit in self.cells.iter().filter_map(|b| attempt[*b]) {
            if seen[(digit - 1) as usize] {
                return false; // A digit appears twice.
            }
            seen[(digit - 1) as usize] = true;
            sum += digit as usize;
        }

        let mut num_digits = seen.iter().filter(|it| **it).count();
        let target_digits  = self.cells.len();
        let target_sum = self.sum as usize;

        check_additional!(0, num_digits, target_digits, sum, target_sum, seen, {
            check_additional!(1, num_digits, target_digits, sum, target_sum, seen, {
                check_additional!(2, num_digits, target_digits, sum, target_sum, seen, {
                    check_additional!(3, num_digits, target_digits, sum, target_sum, seen, {
                        check_additional!(4, num_digits, target_digits, sum, target_sum, seen, {
                            check_additional!(5, num_digits, target_digits, sum, target_sum, seen, {
                                check_additional!(6, num_digits, target_digits, sum, target_sum, seen, {
                                    check_additional!(7, num_digits, target_digits, sum, target_sum, seen, {
                                        // Constraints are only checked if at
                                        // least one cell is filled out. 8 cells
                                        // later, everything is filled out.
                                    });
                                });
                            });
                        });
                    });
                });
            });
        });
        false
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
    solve_rec(
        input,
        &affected_constraints,
        0,
        &mut attempt,
        &mut solutions,
    );
    solutions
}

fn solve_rec(
    input: &Input,
    affected_constraints: &[Vec<usize>],
    first_empty: usize,
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

    if first_empty < attempt.len() {
        'candidates: for i in 1..=9 {
            attempt[first_empty] = Some(i);

            for constraint_index in &affected_constraints[first_empty] {
                let constraint = &input.constraints[*constraint_index];
                if !constraint.is_satisfied_by(attempt) {
                    continue 'candidates;
                }
            }
            solve_rec(
                input,
                affected_constraints,
                first_empty + 1,
                attempt,
                solutions,
            );
        }
        attempt[first_empty] = None;
    } else {
        solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
    }
}
