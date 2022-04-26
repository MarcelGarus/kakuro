use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use itertools::Itertools;
use num_bigint::{BigUint, ToBigUint};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Color {
    Red,
    Blue,
}
impl Color {
    fn flip(self) -> Self {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
        }
    }
}

#[derive(Debug)]
struct SplitInput {
    // The colors vector has the length of the original input, an assigns each
    // cell a color.
    colors: Vec<Color>,

    // A vector that maps cell indizes from the original input to the cell
    // indizes in the smaller parts.
    index_mapping: Vec<usize>,

    red: Input,
    blue: Input,
    connections: Vec<Constraint>,
}
impl SplitInput {
    fn flip(self) -> Self {
        Self {
            colors: self.colors.into_iter().map(|color| color.flip()).collect(),
            red: self.blue,
            blue: self.red,
            ..self
        }
    }
}

fn split(input: &Input) -> Option<SplitInput> {
    for num_connections in 0..input.constraints.len() {
        for connecting_constraints in input.constraints.iter().combinations(num_connections) {
            let remaining_constraints = input
                .constraints
                .iter()
                .filter(|it| !connecting_constraints.contains(it))
                .map(|it| it.clone())
                .collect_vec();

            // Start with all cells blue, then flood fill from the first cell,
            // following constraints.
            let mut colors = vec![Color::Blue; input.num_cells];

            let mut dirty_queue = vec![0];
            while let Some(current) = dirty_queue.pop() {
                colors[current] = Color::Red;
                for constraint in &remaining_constraints {
                    if constraint.cells.contains(&current) {
                        for cell in &constraint.cells {
                            if colors[*cell] == Color::Blue {
                                colors[*cell] = Color::Red;
                                dirty_queue.push(*cell);
                            }
                        }
                    }
                }
            }

            if colors.iter().all(|color| *color == Color::Red) {
                // The whole input was filled red, which means it was not split.
                continue;
            }

            // A vector that maps cell indizes from the original input to the
            // cell indizes in the smaller parts.
            let index_mapping = {
                let mut red_counter = 0;
                let mut blue_counter = 0;
                let mut mapping = vec![];
                for i in 0..input.num_cells {
                    match colors[i] {
                        Color::Red => {
                            mapping.push(red_counter);
                            red_counter += 1;
                        }
                        Color::Blue => {
                            mapping.push(blue_counter);
                            blue_counter += 1;
                        }
                    }
                }
                mapping
            };

            fn create_sub_input(
                color: Color,
                colors: &[Color],
                constraints: &[Constraint],
                index_mapping: &[usize],
            ) -> Input {
                Input {
                    num_cells: colors.iter().filter(|it| **it == color).count(),
                    constraints: constraints
                        .iter()
                        .filter(|constraint| colors[constraint.cells[0]] == color)
                        .map(|constraint| Constraint {
                            cells: constraint
                                .cells
                                .iter()
                                .map(|cell| index_mapping[*cell])
                                .collect(),
                            sum: constraint.sum,
                        })
                        .collect(),
                }
            }
            return Some(SplitInput {
                colors: colors.clone(),
                index_mapping: index_mapping.clone(),
                red: create_sub_input(Color::Red, &colors, &remaining_constraints, &index_mapping),
                blue: create_sub_input(
                    Color::Blue,
                    &colors,
                    &remaining_constraints,
                    &index_mapping,
                ),
                connections: connecting_constraints
                    .iter()
                    .map(|it| (*it).clone())
                    .collect(),
            });
        }
    }
    None
}

#[derive(Debug, Clone)]
enum QuasiSolution {
    Concrete(Solution),
    Plus(Vec<QuasiSolution>),
    Product {
        colors: Vec<Color>,
        red: Box<QuasiSolution>,
        blue: Box<QuasiSolution>,
    },
}
impl QuasiSolution {
    fn size(&self) -> BigUint {
        match self {
            QuasiSolution::Concrete(_) => 1u8.to_biguint().unwrap(),
            QuasiSolution::Plus(children) => children.iter().map(|it| it.size()).sum(),
            QuasiSolution::Product { red, blue, .. } => red.size() * blue.size(),
        }
    }
    fn build(self) -> Vec<Solution> {
        match self {
            QuasiSolution::Concrete(concrete) => vec![concrete],
            QuasiSolution::Plus(children) => {
                Iterator::flatten(children.into_iter().map(|it| it.build())).collect_vec()
            }
            QuasiSolution::Product { colors, red, blue } => {
                let red = red.build();
                let blue = blue.build();
                let mut solutions = vec![];
                for red in &red {
                    for blue in &blue {
                        let mut solution = vec![];
                        let mut red = red.clone();
                        let mut blue = blue.clone();
                        for color in &colors {
                            solution.push(match color {
                                Color::Red => red.remove(0),
                                Color::Blue => blue.remove(0),
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

fn do_digits_satisfy_sum(digits: &[Value], sum: Value) -> bool {
    let mut seen = [false; 9];
    for digit in digits {
        if seen[(digit - 1) as usize] {
            return false; // A digit appears twice.
        } else {
            seen[(digit - 1) as usize] = true;
        }
    }

    digits.into_iter().sum::<Value>() == sum
}

pub fn solve(input: &Input) -> Output {
    let mut solutions = solve_rec(input, &vec![], "");
    let solutions = solutions.remove(&vec![]).unwrap();
    log!("That are {} solutions.", solutions.size());
    // log!("{}", &solutions);
    solutions.build()
}

fn solve_rec(
    input: &Input,
    connecting_cells: &[usize],
    log_prefix: &str,
) -> HashMap<Vec<Value>, QuasiSolution> {
    log!(
        "{}Solving input with {} cells, {} constraints, and {} connecting cells {:?} to pay attention to.",
        log_prefix,
        input.num_cells,
        input.constraints.len(),
        connecting_cells.len(),
        connecting_cells,
    );
    let split = split(input);

    if matches!(split, None) {
        log!("{}Solving with early abort.", log_prefix);
        let solutions = super::sum_reachable_no_set::solve(input);
        log!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        let mut grouped = HashMap::<Vec<Value>, QuasiSolution>::new();
        for solution in solutions {
            let key = connecting_cells.iter().map(|i| solution[*i]).collect_vec();
            grouped
                .entry(key)
                .and_modify(|existing_solution| {
                    *existing_solution = QuasiSolution::Plus(vec![
                        QuasiSolution::Concrete(solution.clone()),
                        existing_solution.clone(),
                    ]);
                })
                .or_insert(QuasiSolution::Concrete(solution));
        }
        return grouped;
    }

    let mut split = split.unwrap();
    log!(
        "{}Split with connections: {:?}",
        log_prefix,
        split.connections
    );
    if split.red.num_cells > split.blue.num_cells {
        split = split.flip();
    }
    log!("{}Mask: {:?}", log_prefix, split.colors);
    let SplitInput {
        colors,
        index_mapping,
        red,
        blue,
        connections,
    } = split;

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut red_mapping = vec![];
    let mut blue_mapping = vec![];
    for i in 0..input.num_cells {
        match colors[i] {
            Color::Red => red_mapping.push(i),
            Color::Blue => blue_mapping.push(i),
        }
    }

    // Solve parts.
    let inner_log_prefix = format!("{}  ", log_prefix);
    let red_connecting_cells = connecting_cells
        .iter()
        .map(|it| *it)
        .chain(
            Iterator::flatten(connections.iter().map(|constraint| &constraint.cells)).map(|it| *it),
        )
        .filter(|it| colors[*it] == Color::Red)
        .map(|it| index_mapping[it])
        .collect_vec();
    let red_solutions = solve_rec(&red, &red_connecting_cells, &inner_log_prefix);
    let blue_connecting_cells = connecting_cells
        .iter()
        .map(|it| *it)
        .chain(
            Iterator::flatten(connections.iter().map(|constraint| &constraint.cells)).map(|it| *it),
        )
        .filter(|it| colors[*it] == Color::Blue)
        .map(|it| index_mapping[it])
        .collect_vec();
    let blue_solutions = solve_rec(&blue, &blue_connecting_cells, &inner_log_prefix);

    // Combine results.
    log!(
        "{}Combining {}x{} solutions with {} connections.",
        log_prefix,
        red_solutions.len(),
        blue_solutions.len(),
        connections.len(),
    );
    log!(
        "{}Naively joining solutions would require checking {} candidates.",
        log_prefix,
        red_solutions.len() * blue_solutions.len()
    );
    let mut solutions = vec![];
    for (red_connecting_values, red_solution) in &red_solutions {
        'solutions: for (blue_connecting_values, blue_solution) in &blue_solutions {
            for constraint in &connections {
                let values = constraint
                    .cells
                    .iter()
                    .map(|i| match colors[*i] {
                        Color::Red => {
                            let index = index_mapping[*i];
                            let connecting_index = red_connecting_cells
                                .iter()
                                .position(|it| *it == index)
                                .unwrap();
                            red_connecting_values[connecting_index]
                        }
                        Color::Blue => {
                            let index = index_mapping[*i];
                            let connecting_index = blue_connecting_cells
                                .iter()
                                .position(|it| *it == index)
                                .unwrap();
                            blue_connecting_values[connecting_index]
                        }
                    })
                    .collect_vec();
                if !do_digits_satisfy_sum(&values, constraint.sum) {
                    continue 'solutions;
                }
            }
            log!(
                "{}  Combining red {:?} and blue {:?} works.",
                log_prefix,
                red_connecting_values,
                blue_connecting_values,
            );
            let key = connecting_cells
                .iter()
                .map(|i| match colors[*i] {
                    Color::Red => {
                        let index = index_mapping[*i];
                        let connecting_index = red_connecting_cells
                            .iter()
                            .position(|it| *it == index)
                            .unwrap();
                        red_connecting_values[connecting_index]
                    }
                    Color::Blue => {
                        let index = index_mapping[*i];
                        let connecting_index = blue_connecting_cells
                            .iter()
                            .position(|it| *it == index)
                            .unwrap();
                        blue_connecting_values[connecting_index]
                    }
                })
                .collect_vec();
            let value = QuasiSolution::Product {
                colors: colors.clone(),
                red: Box::new(red_solution.clone()),
                blue: Box::new(blue_solution.clone()),
            };
            solutions.push((key, value));
        }
    }

    let mut grouped = HashMap::<Vec<Value>, QuasiSolution>::new();
    for (key, solution) in solutions {
        grouped
            .entry(key)
            .and_modify(|existing_solution| {
                *existing_solution =
                    QuasiSolution::Plus(vec![solution.clone(), existing_solution.clone()]);
            })
            .or_insert(solution);
    }

    log!("{}Done. Found some solutions.", log_prefix);
    grouped
}
