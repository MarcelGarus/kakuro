use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::collections::HashMap;

type Game = Vec<Cell>;
type Cell = Option<Value>;

trait ConstraintExt {
    fn is_satisfied_by(&self, attempt: &Game) -> bool;
}
impl ConstraintExt for Constraint {
    fn is_satisfied_by(&self, attempt: &Game) -> bool {
        let cells: ArrayVec<Option<Value>, 9> = self.cells.iter().map(|b| attempt[*b]).collect();
        let digits: ArrayVec<Value, 9> = cells.into_iter().filter_map(|it| it).collect();

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

        let mut unused_digits = [false; 9];
        for i in 0..9 {
            unused_digits[i] = !seen[i];
        }
        log!(
            "Checking if any combination of {:?} with length {} plus existing sum {} yields total sum {}.",
            unused_digits,
            self.cells.len() - digits.len(),
            sum,
            self.sum
        );
        return is_sum_reachable(
            &mut unused_digits,
            self.cells.len() - digits.len(),
            (self.sum as i8) - (sum as i8),
        );
    }
}

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![None; input.num_cells];
    let mut solutions = vec![];
    let mut affected_constraints = HashMap::new();
    for (i, constraint) in input.constraints.iter().enumerate() {
        for cell in &constraint.cells {
            affected_constraints.entry(*cell).or_insert(vec![]).push(i);
        }
    }
    solve_rec(input, &affected_constraints, &mut attempt, &mut solutions);
    solutions
}

fn solve_rec(
    input: &Input,
    affected_constraints: &HashMap<usize, Vec<usize>>,
    attempt: &mut Game,
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
            for constraint_index in &affected_constraints[&index] {
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

// A lookup table where you can look up the total number of digits as well as
// the existing digits to find the minimum and maximum reachable sum.
// type ReachableSumsMap = [[(Value, Value); 1 << 9]; 10];

// const REACHABLE_SUMS: () = calculate_reachable_sums();

// const fn calculate_reachable_sums() {
//     let mut reachable_sums: [[(Value, Value); 1 << 9]; 10] = Default::default();
//     for total_digits in 0..9 {
//         for existing_digits in (1u8..=9).combinations(total_digits) {
//             let remaining_digits = (1u8..=9)
//                 .filter(|it| !existing_digits.contains(it))
//                 .collect_vec();
//             let (min, max) = if existing_digits.len() <= total_digits {
//                 remaining_digits
//                     .combinations(total_digits - existing_digits.len())
//                     .map(|it| it.sum())
//                     .map(|sum| (sum, sum))
//                     .reduce(|(min_a, max_a), (min_b, max_b)| {
//                         (cmp::min(min_a, min_b), cmp::max(max_a, max_b))
//                     })
//                     .unwrap();
//             } else {
//                 (0, 0)
//             };
//             reachable_sums[total_digits][existing_digits] = (min, max);
//         }
//     }
// }

fn is_sum_reachable(digits_left: &mut [bool; 9], allowed_to_use: usize, sum: i8) -> bool {
    if sum < 0 {
        return false;
    }
    if sum == 0 {
        return allowed_to_use == 0;
    }
    for digit in 1..=9 {
        if !digits_left[digit - 1] {
            continue;
        }
        digits_left[digit - 1] = false;
        if is_sum_reachable(digits_left, allowed_to_use - 1, sum - digit as i8) {
            return true;
        }
        digits_left[digit - 1] = true;
    }
    return false;
}

pub struct Combinations {
    values: ArrayVec<Value, 9>,
    n: u32,
    mask: u16, // The lower 9 bits are a mask for the chosen values.
}
impl Combinations {
    fn current_values(&self) -> ArrayVec<Value, 9> {
        log!("Getting current values for mask {:9b}", self.mask);
        let mut output = ArrayVec::<Value, 9>::new();
        for (i, bit_mask) in (0..self.values.len()).map(|it| 1 << it).enumerate() {
            if self.mask & bit_mask != 0 {
                output.push(self.values[i]);
            }
        }
        output
    }
}
impl Iterator for Combinations {
    type Item = ArrayVec<Value, 9>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.mask >= (1 << 9) {
                return None;
            }
            if self.mask.count_ones() != self.n {
                self.mask += 1;
                continue;
            }
            self.mask += 1;
            return Some(self.current_values());
        }
    }
}

// fn combinations(values: ArrayVec<Value, 9>, n: usize) {
//     let mut i = 0;
//     'outer: loop {
//         if i == input.num_cells {
//             // No cell is free anymore. We have a solution.
//             solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
//             i -= 1;
//         } else {
//             attempt[i] = match attempt[i] {
//                 None => Some(1),
//                 Some(9) => None,
//                 Some(i) => Some(i + 1),
//             };

//             if attempt[i].is_none() {
//                 if i == 0 {
//                     break;
//                 } else {
//                     i -= 1;
//                     continue;
//                 }
//             }

//             for constraint_index in &affected_constraints[&i] {
//                 let constraint = &input.constraints[*constraint_index];
//                 if !constraint.is_satisfied_by(&attempt) {
//                     continue 'outer;
//                 }
//             }
//             i += 1;
//         }
//     }
// }
