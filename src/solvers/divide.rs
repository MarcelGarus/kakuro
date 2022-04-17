use crate::game::{Constraint, Input, Output, Solution, Value};
use itertools::Itertools;
use std::collections::HashMap;

pub fn solve(input: &Input) -> Output {
    solve_rec(input, "")
}

fn solve_rec(input: &Input, log_prefix: &str) -> Vec<Solution> {
    println!(
        "{}Solving input with {} cells and {} constraints.",
        log_prefix,
        input.num_cells,
        input.constraints.len(),
    );
    let splitted = split(input);

    if matches!(splitted, None) {
        // || input.num_cells < 10 {
        println!("{}Solving with early abort.", log_prefix);
        let solutions = super::early_abort::solve(input);
        println!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        return solutions;
    }

    let mut splitted = splitted.unwrap();
    println!(
        "{}Split with connections: {:?}",
        log_prefix, splitted.connections
    );
    if splitted.white.num_cells > splitted.black.num_cells {
        splitted = splitted.flip();
    }

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut white_mapping = vec![];
    let mut black_mapping = vec![];
    for i in 0..input.num_cells {
        match splitted.colors[i] {
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
            match splitted.colors[i] {
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
    let white_solutions = solve_rec(&splitted.white, &inner_log_prefix);
    let black_solutions = solve_rec(&splitted.black, &inner_log_prefix);

    // Combine results.
    println!(
        "{}Combining {}x{} solutions with {} connections.",
        log_prefix,
        white_solutions.len(),
        black_solutions.len(),
        splitted.connections.len(),
    );
    println!(
        "{}Naively joining solutions would require checking {} candidates.",
        log_prefix,
        white_solutions.len() * black_solutions.len()
    );
    let mut white_solutions_by_connection_cells = HashMap::new();
    for solution in white_solutions {
        let key = splitted
            .connections
            .iter()
            .map(|constraint| {
                constraint
                    .cells
                    .iter()
                    .filter(|cell| splitted.colors[**cell] == Color::White)
                    .map(|cell| solution[mapping[*cell]])
                    .collect_vec()
            })
            .collect_vec();
        white_solutions_by_connection_cells
            .entry(key)
            .or_insert_with_key(|_| vec![])
            .push(solution);
    }
    let mut black_solutions_by_sums = HashMap::new();
    for solution in black_solutions {
        let key = splitted
            .connections
            .iter()
            .map(|constraint| {
                constraint
                    .cells
                    .iter()
                    .filter(|cell| splitted.colors[**cell] == Color::Black)
                    .map(|cell| solution[mapping[*cell]])
                    .collect_vec()
            })
            .collect_vec();
        black_solutions_by_sums
            .entry(key)
            .or_insert_with_key(|_| vec![])
            .push(solution);
    }
    let mut solutions = vec![];
    for (white_connecting_values, white_solutions) in &white_solutions_by_connection_cells {
        'solutions: for (black_connecting_values, black_solutions) in &black_solutions_by_sums {
            for ((white, black), constraint) in white_connecting_values
                .iter()
                .zip(black_connecting_values)
                .zip(splitted.connections.iter())
            {
                let mut values = vec![];
                values.append(&mut white.clone());
                values.append(&mut black.clone());
                if !do_values_satisfy_sum(&values, constraint.sum) {
                    continue 'solutions;
                }
            }
            println!(
                "{}  Combining white {:?} and black {:?} works and yields {}x{} = {} candidates.",
                log_prefix,
                white_connecting_values,
                black_connecting_values,
                white_solutions.len(),
                black_solutions.len(),
                white_solutions.len() * black_solutions.len()
            );
            for white_solution in white_solutions {
                for black_solution in black_solutions {
                    let mut attempt = vec![0; input.num_cells];
                    for (i, value) in white_solution.iter().enumerate() {
                        attempt[white_mapping[i]] = *value;
                    }
                    for (i, value) in black_solution.iter().enumerate() {
                        attempt[black_mapping[i]] = *value;
                    }
                    solutions.push(attempt);
                }
            }
        }
    }

    println!("{}Done. Found {} solutions.", log_prefix, solutions.len());
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
