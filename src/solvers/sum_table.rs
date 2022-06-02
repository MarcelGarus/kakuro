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

        let mut unused_digits = 0u16;
        for i in 0..9 {
            if seen[i] {
                unused_digits &= 1 << i;
            }
        }
        log!(
            "Checking if any combination of {:?} with length {} plus existing sum {} yields total sum {}.",
            unused_digits,
            self.cells.len() - digits.len(),
            sum,
            self.sum
        );
        if sum > self.sum {
            return false;
        } else {
            return is_sum_reachable(
                self.cells.len() - digits.len(),
                unused_digits,
                self.sum - sum,
            );
        }
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

fn is_sum_reachable(num_total_digits: usize, existing_digits: u16, sum: u8) -> bool {
    SUM_TABLE[num_total_digits][existing_digits as usize] & 1u64 << sum != 0
}

lazy_static! {
    static ref SUM_TABLE: [[u64; 1 << 9]; 10] = calculate_sum_table();
}

fn calculate_sum_table() -> [[u64; 1 << 9]; 10] {
    let mut table = [[0; 1 << 9]; 10];

    for total_digits in 0..=9 {
        for digits in 0b000_000_000..=0b111_111_111 {
            let mut reachable_sums = [false; 46];
            fill_reachable_sums(
                digits,
                (total_digits - digits.count_ones()) as u8,
                &mut reachable_sums,
            );
            let mut reachable = 0u64;
            for (i, is_reachable) in reachable_sums.into_iter().enumerate() {
                if *is_reachable {
                    reachable |= 1 << i;
                }
            }
            table[total_digits as usize][digits as usize] = reachable;
        }
    }

    table
}
fn fill_reachable_sums(unused_digits: u16, num_to_use: u8, reachable_sums: &mut [bool; 46]) {
    fill_reachable_sums_rec(unused_digits, num_to_use, 0, reachable_sums);
}
fn fill_reachable_sums_rec(
    unused_digits: u16,
    num_to_use: u8,
    sum_so_far: u8,
    reachable_sums: &mut [bool; 46],
) {
    println!("unused digits: {:b}", &unused_digits);
    println!("num to use:    {}", &num_to_use);
    println!("sum so far:    {}", &sum_so_far);
    println!("reachables:    {:?}", &reachable_sums);

    if num_to_use == 0 {
        reachable_sums[sum_so_far as usize] = true;
        return;
    }
    for (digit, mask) in (0..9).map(|it| (it as u8 + 1, 1 << it)) {
        if unused_digits & mask != 1 {
            continue;
        }
        let inner_unused_digits = unused_digits & ((1 << digit) - 1);
        fill_reachable_sums_rec(
            inner_unused_digits,
            num_to_use - 1,
            sum_so_far + digit,
            reachable_sums,
        );
    }
}
