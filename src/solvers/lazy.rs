use crate::game::{Constraint, Input, Output, Solution, Value};
use itertools::Itertools;
use std::collections::HashMap;

pub fn solve(input: &Input) -> Output {
    let mut solutions = solve_rec(input, &vec![], "");
    let solutions = solutions.remove(&vec![]).unwrap();
    // dbg!(&solutions);
    println!("That are {} solutions.", solutions.size());
    solutions.build()
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
    fn size(&self) -> usize {
        match self {
            QuasiSolution::Concrete(_) => 1,
            QuasiSolution::Plus(children) => children.iter().map(|it| it.size()).sum(),
            QuasiSolution::Product { white, black, .. } => white.size() * black.size(),
        }
    }
    fn simplify(self) -> Self {
        match self {
            QuasiSolution::Concrete(concrete) => QuasiSolution::Concrete(concrete),
            QuasiSolution::Plus(children) => {
                let mut children = children
                    .into_iter()
                    .map(|it| it.simplify())
                    .filter(|it| it.size() == 0)
                    .collect_vec();
                if children.len() == 1 {
                    children.pop().unwrap()
                } else {
                    QuasiSolution::Plus(children)
                }
            }
            QuasiSolution::Product {
                colors,
                white,
                black,
            } => {
                let white = white.simplify();
                let black = black.simplify();
                if white.size() == 1 || black.size() == 1 {
                    // Merge.
                }

                QuasiSolution::Product {
                    colors,
                    white: Box::new(white),
                    black: Box::new(black),
                }
            }
        }
    }
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

fn solve_rec(
    input: &Input,
    connecting_cells: &[usize],
    log_prefix: &str,
) -> HashMap<Vec<Value>, QuasiSolution> {
    println!(
        "{}Solving input with {} cells, {} constraints, and {} connecting cells {:?} to pay attention to.",
        log_prefix,
        input.num_cells,
        input.constraints.len(),
        connecting_cells.len(),
        connecting_cells,
    );
    let splitted = split(input);

    if matches!(splitted, None) {
        println!("{}Solving with early abort.", log_prefix);
        let solutions = super::early_abort::solve(input);
        println!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        let grouped = solutions
            .into_iter()
            .group_by(|cells| connecting_cells.iter().map(|i| cells[*i]).collect_vec());
        return grouped
            .into_iter()
            .map(|(key, group)| {
                (
                    key,
                    QuasiSolution::Plus(group.map(QuasiSolution::Concrete).collect_vec()),
                )
            })
            .collect();
    }

    let mut splitted = splitted.unwrap();
    println!(
        "{}Split with connections: {:?}",
        log_prefix, splitted.connections
    );
    if splitted.white.num_cells > splitted.black.num_cells {
        splitted = splitted.flip();
    }
    println!("{}Mask: {:?}", log_prefix, splitted.colors);
    let SplittedKakuro {
        colors,
        white,
        black,
        connections,
    } = splitted;

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut white_mapping = vec![];
    let mut black_mapping = vec![];
    for i in 0..input.num_cells {
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
        for i in 0..input.num_cells {
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
    let white_connecting_cells = connecting_cells
        .iter()
        .map(|it| *it)
        .chain(
            Iterator::flatten(connections.iter().map(|constraint| &constraint.cells)).map(|it| *it),
        )
        .filter(|it| colors[*it] == Color::White)
        .map(|it| mapping[it])
        .collect_vec();
    let white_solutions = solve_rec(&white, &white_connecting_cells, &inner_log_prefix);
    let black_connecting_cells = connecting_cells
        .iter()
        .map(|it| *it)
        .chain(
            Iterator::flatten(connections.iter().map(|constraint| &constraint.cells)).map(|it| *it),
        )
        .filter(|it| colors[*it] == Color::Black)
        .map(|it| mapping[it])
        .collect_vec();
    let black_solutions = solve_rec(&black, &black_connecting_cells, &inner_log_prefix);

    // Combine results.
    println!(
        "{}Combining {}x{} solutions with {} connections.",
        log_prefix,
        white_solutions.len(),
        black_solutions.len(),
        connections.len(),
    );
    println!(
        "{}Naively joining solutions would require checking {} candidates.",
        log_prefix,
        white_solutions.len() * black_solutions.len()
    );
    let mut solutions = vec![];
    for (white_connecting_values, white_solution) in &white_solutions {
        'solutions: for (black_connecting_values, black_solution) in &black_solutions {
            for constraint in &connections {
                let values = constraint
                    .cells
                    .iter()
                    .map(|i| match colors[*i] {
                        Color::White => {
                            let index = mapping[*i];
                            let connecting_index = white_connecting_cells
                                .iter()
                                .position(|it| *it == index)
                                .unwrap();
                            white_connecting_values[connecting_index]
                        }
                        Color::Black => {
                            let index = mapping[*i];
                            let connecting_index = black_connecting_cells
                                .iter()
                                .position(|it| *it == index)
                                .unwrap();
                            black_connecting_values[connecting_index]
                        }
                    })
                    .collect_vec();
                if !do_values_satisfy_sum(&values, constraint.sum) {
                    continue 'solutions;
                }
            }
            println!(
                "{}  Combining white {:?} and black {:?} works.",
                log_prefix, white_connecting_values, black_connecting_values,
            );
            let key = connecting_cells
                .iter()
                .map(|i| match colors[*i] {
                    Color::White => {
                        let index = mapping[*i];
                        let connecting_index = white_connecting_cells
                            .iter()
                            .position(|it| *it == index)
                            .unwrap();
                        white_connecting_values[connecting_index]
                    }
                    Color::Black => {
                        let index = mapping[*i];
                        let connecting_index = black_connecting_cells
                            .iter()
                            .position(|it| *it == index)
                            .unwrap();
                        black_connecting_values[connecting_index]
                    }
                })
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
            (
                key,
                if values.len() == 0 {
                    values[0].clone()
                } else {
                    QuasiSolution::Plus(values)
                },
            )
        })
        .collect();

    println!("{}Done. Found some solutions.", log_prefix);
    solutions
}

fn do_values_satisfy_sum(values: &[Value], sum: Value) -> bool {
    for (i, a) in values.iter().enumerate() {
        for (j, b) in values.iter().enumerate() {
            if a == b && i != j {
                return false; // Duplicate value.
            }
        }
    }
    values.into_iter().sum::<Value>() == sum
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

#[derive(Debug)]
struct SplittedKakuro {
    colors: Vec<Color>,
    white: Input,
    black: Input,
    connections: Vec<Constraint>,
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
            white: self.black,
            black: self.white,
            connections: self.connections,
        }
    }
}

fn split(input: &Input) -> Option<SplittedKakuro> {
    // println!("Splitting.");
    for num_connections in 0..input.constraints.len() {
        // println!("Maybe we can split with {} connections?", num_connections);
        for connecting_constraints in input.constraints.iter().combinations(num_connections) {
            let remaining_constraints = input
                .constraints
                .iter()
                .filter(|it| !connecting_constraints.contains(it))
                .collect_vec();
            // println!(
            //     "How about connections {:?} and remaining constraints {:?}?",
            //     connecting_constraints, remaining_constraints
            // );

            let mut colors = vec![Color::Black; input.num_cells];
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

            if colors.iter().any(|color| *color == Color::Black) {
                let mapping = {
                    let mut white_counter = 0;
                    let mut black_counter = 0;
                    let mut mapping = vec![];
                    for i in 0..input.num_cells {
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

                return Some(SplittedKakuro {
                    colors: colors.clone(),
                    white: Input {
                        num_cells: colors
                            .iter()
                            .filter(|color| **color == Color::White)
                            .count(),
                        constraints: remaining_constraints
                            .iter()
                            .filter(|constraint| colors[constraint.cells[0]] == Color::White)
                            .map(|constraint| Constraint {
                                cells: constraint.cells.iter().map(|cell| mapping[*cell]).collect(),
                                sum: constraint.sum,
                            })
                            .collect(),
                    },
                    black: Input {
                        num_cells: colors
                            .iter()
                            .filter(|color| **color == Color::Black)
                            .count(),
                        constraints: remaining_constraints
                            .iter()
                            .filter(|constraint| colors[constraint.cells[0]] == Color::Black)
                            .map(|constraint| Constraint {
                                cells: constraint.cells.iter().map(|cell| mapping[*cell]).collect(),
                                sum: constraint.sum,
                            })
                            .collect(),
                    },
                    connections: connecting_constraints
                        .iter()
                        .map(|it| (*it).clone())
                        .collect(),
                });
            }
        }
    }
    None
}
