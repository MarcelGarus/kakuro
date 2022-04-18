use crate::game::{self, Output, Solution, Value};
use itertools::Itertools;
use std::{cmp::max, collections::HashMap};

pub fn solve(input: &game::Input) -> Output {
    let mut solutions = solve_rec(
        input.num_cells,
        &input
            .constraints
            .iter()
            .map(|it| it.clone().into())
            .collect_vec(),
        &[],
        "",
    );
    let solutions = solutions.remove(&vec![]).unwrap();
    // dbg!(&solutions);
    // println!("That are {} solutions.", solutions.size());
    solutions.build()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    cells: Vec<usize>,
    min: Value,
    max: Value,
}

impl From<game::Constraint> for Constraint {
    fn from(constraint: game::Constraint) -> Self {
        Self {
            cells: constraint.cells,
            min: constraint.sum,
            max: constraint.sum,
        }
    }
}
#[derive(Debug, Clone)]
enum QuasiSolution {
    Concrete(Solution),
    Plus(Vec<QuasiSolution>),
    Product {
        colors: Vec<Color>,
        white: Box<QuasiSolution>,
        black: Box<QuasiSolution>,
    },
}
impl QuasiSolution {
    fn build(self) -> Vec<Solution> {
        match self {
            QuasiSolution::Concrete(concrete) => vec![concrete],
            QuasiSolution::Plus(children) => {
                Iterator::flatten(children.into_iter().map(|it| it.build())).collect_vec()
            }
            QuasiSolution::Product {
                colors,
                white,
                black,
            } => {
                let mut white = white.build();
                let mut black = black.build();
                let mut solutions = vec![];
                for white in &mut white {
                    for black in &mut black {
                        let mut solution = vec![];
                        for color in &colors {
                            solution.push(match color {
                                Color::White => white.remove(0),
                                Color::Black => black.remove(0),
                            });
                        }
                        solutions.push(solution);
                    }
                }
                solutions
            }
        }
    }
}

// Takes a number of cells and all constraints. The additional information of
// connecting constraints is a subset of all constraints and is used to group
// the solutions in the return value:
// For each connecting constraint, there's a vec containing a set of possible
// numbers.
fn solve_rec(
    num_cells: usize,
    all_constraints: &[Constraint],
    connecting_constraints: &[Constraint],
    log_prefix: &str,
) -> HashMap<Vec<Vec<Value>>, QuasiSolution> {
    // println!(
    //     "{}Solving input with {} cells and {} constraints to pay attention to.",
    //     log_prefix,
    //     num_cells,
    //     all_constraints.len(),
    // );
    let splitted = split(num_cells, all_constraints);

    if splitted.is_none() {
        // println!("{}Solving with early abort.", log_prefix);
        let solutions = early_abort::solve(num_cells, all_constraints);
        // println!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        let grouped = solutions.into_iter().group_by(|solution| {
            connecting_constraints
                .iter()
                .map(|connection| {
                    let mut cells = connection.cells.iter().map(|i| solution[*i]).collect_vec();
                    cells.sort();
                    cells
                })
                .collect_vec()
        });
        return grouped
            .into_iter()
            .map(|(key, group)| {
                let solution =
                    QuasiSolution::Plus(group.map(QuasiSolution::Concrete).collect_vec());
                (key, solution)
            })
            .collect();
    }

    let mut splitted = splitted.unwrap();
    // println!(
    //     "{}Split with connections: {:?}",
    //     log_prefix, splitted.connecting_constraints
    // );
    let more_white_than_black = splitted
        .colors
        .iter()
        .filter(|it| **it == Color::White)
        .count()
        > num_cells / 2;
    if more_white_than_black {
        splitted = splitted.flip();
    }
    // println!("{}Mask: {:?}", log_prefix, splitted.colors);
    let SplittedKakuro {
        colors,
        white_constraints,
        black_constraints,
        connecting_constraints: split_connecting_constraints,
    } = splitted;

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut white_mapping = vec![];
    let mut black_mapping = vec![];
    for i in 0..num_cells {
        match colors[i] {
            Color::White => white_mapping.push(i),
            Color::Black => black_mapping.push(i),
        }
    }

    // The mapping in the other direction. Copied from below.
    let mapping = {
        let mut white_counter = 0;
        let mut black_counter = 0;
        let mut mapping = vec![];
        for i in 0..num_cells {
            match colors[i] {
                Color::White => {
                    mapping.push(white_counter);
                    white_counter += 1;
                }
                Color::Black => {
                    mapping.push(black_counter);
                    black_counter += 1;
                }
            }
        }
        mapping
    };

    // Solve parts.
    let inner_log_prefix = format!("{}  ", log_prefix);
    let white_solutions = solve_rec(
        white_mapping.len(),
        &white_constraints,
        &add_slices(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| translate_constraint(&constraint, Color::White, &colors, &mapping))
            .collect_vec(),
        &inner_log_prefix,
    );
    let black_solutions = solve_rec(
        black_mapping.len(),
        &black_constraints,
        &add_slices(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| translate_constraint(&constraint, Color::Black, &colors, &mapping))
            .collect_vec(),
        &inner_log_prefix,
    );

    // Combine results.
    // println!(
    //     "{}Combining {}x{} solutions with {} connections.",
    //     log_prefix,
    //     white_solutions.len(),
    //     black_solutions.len(),
    //     split_connecting_constraints.len(),
    // );
    // println!(
    //     "{}Naively joining solutions would require checking {} candidates.",
    //     log_prefix,
    //     white_solutions.len() * black_solutions.len()
    // );
    let mut solutions = vec![];
    for (white_connecting_values, white_solution) in &white_solutions {
        'solutions: for (black_connecting_values, black_solution) in &black_solutions {
            for ((constraint, white_values), black_values) in split_connecting_constraints
                .iter()
                .zip(
                    white_connecting_values
                        .iter()
                        .skip(connecting_constraints.len()),
                )
                .zip(
                    black_connecting_values
                        .iter()
                        .skip(connecting_constraints.len()),
                )
            {
                let values = add_slices(white_values, black_values);
                if !do_values_satisfy_sum(&values, constraint.min, constraint.max) {
                    continue 'solutions;
                }
            }
            // println!(
            //     "{}  Combining white {:?} and black {:?} works.",
            //     log_prefix, white_connecting_values, black_connecting_values,
            // );
            let key = connecting_constraints
                .iter()
                .zip(white_connecting_values)
                .zip(black_connecting_values)
                .map(|((_, white_values), black_values)| add_slices(white_values, black_values))
                .collect_vec();
            let value = QuasiSolution::Product {
                colors: colors.clone(),
                white: Box::new(white_solution.clone()),
                black: Box::new(black_solution.clone()),
            };
            solutions.push((key, value));
        }
    }

    let solutions = solutions
        .into_iter()
        .group_by(|it| it.0.clone())
        .into_iter()
        .map(|(key, values)| {
            let values = values.map(|it| it.1).collect_vec();
            let solution = if values.is_empty() {
                values[0].clone()
            } else {
                QuasiSolution::Plus(values)
            };
            (key, solution)
        })
        .collect();

    // println!("{}Done. Found some solutions.", log_prefix);
    solutions
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Color {
    White,
    Black,
}

#[derive(Debug)]
struct SplittedKakuro {
    colors: Vec<Color>,
    white_constraints: Vec<Constraint>,
    black_constraints: Vec<Constraint>,
    connecting_constraints: Vec<Constraint>,
}
impl SplittedKakuro {
    fn flip(self) -> Self {
        Self {
            colors: self
                .colors
                .into_iter()
                .map(|color| match color {
                    Color::White => Color::Black,
                    Color::Black => Color::White,
                })
                .collect(),
            white_constraints: self.black_constraints,
            black_constraints: self.white_constraints,
            connecting_constraints: self.connecting_constraints,
        }
    }
}

fn split(num_cells: usize, constraints: &[Constraint]) -> Option<SplittedKakuro> {
    // println!("Splitting.");
    for num_connections in 0..constraints.len() {
        // println!("Maybe we can split with {} connections?", num_connections);
        for connecting_constraints in constraints.iter().combinations(num_connections) {
            let connecting_constraints = connecting_constraints
                .into_iter()
                .map(|it| it.clone())
                .collect_vec();
            let remaining_constraints = constraints
                .iter()
                .filter(|it| !connecting_constraints.contains(it))
                .map(|it| it.clone())
                .collect_vec();
            // println!(
            //     "How about connections {:?} and remaining constraints {:?}?",
            //     connecting_constraints, remaining_constraints
            // );

            let mut colors = vec![Color::Black; num_cells];
            let mut dirty_queue = vec![0];

            while let Some(current) = dirty_queue.pop() {
                colors[current] = Color::White;
                let connections = remaining_constraints
                    .iter()
                    .filter(|it| it.cells.contains(&current))
                    .collect_vec();
                for connection in connections {
                    for cell in &connection.cells {
                        if colors[*cell] == Color::Black {
                            colors[*cell] = Color::White;
                            dirty_queue.push(*cell);
                        }
                    }
                }
            }

            if !colors.iter().any(|color| *color == Color::Black) {
                continue;
            }

            let mapping = {
                let mut white_counter = 0;
                let mut black_counter = 0;
                let mut mapping = vec![];
                for i in 0..num_cells {
                    match colors[i] {
                        Color::White => {
                            mapping.push(white_counter);
                            white_counter += 1;
                        }
                        Color::Black => {
                            mapping.push(black_counter);
                            black_counter += 1;
                        }
                    }
                }
                mapping
            };

            fn constraints_for_color(
                color: Color,
                constraints: &[Constraint],
                colors: &[Color],
                mapping: &[usize],
            ) -> Vec<Constraint> {
                constraints
                    .iter()
                    .map(|constraint| translate_constraint(constraint, color, &colors, &mapping))
                    .filter(|constraint| !constraint.cells.is_empty())
                    .collect()
            }
            let all_constraints = add_slices(&connecting_constraints, &remaining_constraints);
            return Some(SplittedKakuro {
                colors: colors.clone(),
                white_constraints: constraints_for_color(
                    Color::White,
                    &all_constraints,
                    &colors,
                    &mapping,
                ),
                black_constraints: constraints_for_color(
                    Color::Black,
                    &all_constraints,
                    &colors,
                    &mapping,
                ),
                connecting_constraints: connecting_constraints
                    .iter()
                    .map(|it| (*it).clone())
                    .collect(),
            });
        }
    }
    None
}

fn translate_constraint(
    constraint: &Constraint,
    color: Color,
    colors: &[Color],
    mapping: &[usize],
) -> Constraint {
    let own_cells = constraint
        .cells
        .iter()
        .filter(|it| colors[**it] == color)
        .map(|cell| mapping[*cell])
        .collect_vec();
    let num_other_cells = constraint.cells.len() - own_cells.len();
    Constraint {
        cells: own_cells,
        min: max(0, constraint.min as i8 - MAXES[num_other_cells] as i8) as Value,
        max: max(0, constraint.max as i8 - MINS[num_other_cells] as i8) as Value,
    }
}

fn do_values_satisfy_sum(values: &[Value], min: Value, max: Value) -> bool {
    for (i, a) in values.iter().enumerate() {
        for (j, b) in values.iter().enumerate() {
            if a == b && i != j {
                return false; // Duplicate value.
            }
        }
    }
    let sum = values.into_iter().sum::<Value>();
    min <= sum && sum <= max
}

const MINS: [Value; 10] = [
    0,
    1,
    1 + 2,
    1 + 2 + 3,
    1 + 2 + 3 + 4,
    1 + 2 + 3 + 4 + 5,
    1 + 2 + 3 + 4 + 5 + 6,
    1 + 2 + 3 + 4 + 5 + 6 + 7,
    1 + 2 + 3 + 4 + 5 + 6 + 7 + 8,
    1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9,
];
const MAXES: [Value; 10] = [
    0,
    9,
    8 + 9,
    7 + 8 + 9,
    6 + 7 + 8 + 9,
    5 + 6 + 7 + 8 + 9,
    4 + 5 + 6 + 7 + 8 + 9,
    3 + 4 + 5 + 6 + 7 + 8 + 9,
    2 + 3 + 4 + 5 + 6 + 7 + 8 + 9,
    1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9,
];

mod early_abort {
    use super::*;
    use std::collections::HashSet;

    type Game = Vec<Cell>;
    type Cell = Option<Value>;

    fn is_possible_solution(constraints: &[Constraint], attempt: &Game) -> bool {
        for constraint in constraints.iter() {
            let numbers = constraint
                .cells
                .iter()
                .filter_map(|b| attempt[*b])
                .collect_vec();

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
                .any(|possible_sum| {
                    constraint.min <= possible_sum && possible_sum <= constraint.max
                });
            if !is_possible_to_reach_sum {
                return false;
            }
        }
        return true;
    }

    pub fn solve(num_cells: usize, constraints: &[Constraint]) -> Output {
        let mut attempt = vec![None; num_cells];
        let mut solutions = vec![];
        solve_rec(num_cells, constraints, &mut attempt, &mut solutions);
        solutions
    }

    fn solve_rec(
        num_cells: usize,
        constraints: &[Constraint],
        attempt: &mut Game,
        solutions: &mut Vec<Solution>,
    ) {
        // println!(
        //     "Attempt {}",
        //     attempt
        //         .iter()
        //         .map(|cell| match cell {
        //             Some(number) => format!("{}", number),
        //             None => "-".to_string(),
        //         })
        //         .join("")
        // );
        if !is_possible_solution(constraints, attempt) {
            return;
        }
        let index_to_fill = attempt.iter().position(|it| it.is_none());
        if let Some(index) = index_to_fill {
            for i in 1..=9 {
                attempt[index] = Some(i);
                solve_rec(num_cells, constraints, attempt, solutions);
            }
            attempt[index] = None;
        } else {
            // This is a solution.
            solutions.push(attempt.iter().map(|it| it.unwrap()).collect());
        }
    }
}

fn add_slices<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut v = vec![];
    for t in a {
        v.push(t.clone());
    }
    for t in b {
        v.push(t.clone());
    }
    v
}
