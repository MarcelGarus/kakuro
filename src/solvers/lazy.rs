use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log::log,
};
use itertools::Itertools;
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
    fn size(&self) -> usize {
        match self {
            QuasiSolution::Concrete(_) => 1,
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
                let mut red = red.build();
                let mut blue = blue.build();
                let mut solutions = vec![];
                for red in &mut red {
                    for blue in &mut blue {
                        let mut solution = vec![];
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

pub fn solve(input: &Input) -> Output {
    let mut solutions = solve_rec(input, &vec![], "");
    let solutions = solutions.remove(&vec![]).unwrap();
    log(format!("That are {} solutions.", solutions.size()));
    solutions.build()
}

fn solve_rec(
    input: &Input,
    connecting_cells: &[usize],
    log_prefix: &str,
) -> HashMap<Vec<Value>, QuasiSolution> {
    log(format!(
        "{}Solving input with {} cells, {} constraints, and {} connecting cells {:?} to pay attention to.",
        log_prefix,
        input.num_cells,
        input.constraints.len(),
        connecting_cells.len(),
        connecting_cells,
    ));
    let split = split(input);

    if matches!(split, None) {
        log(format!("{}Solving with early abort.", log_prefix));
        let solutions = super::sum_reachable::solve(input);
        log(format!(
            "{}Done. Found {} solutions.",
            log_prefix,
            solutions.len()
        ));
        let grouped = solutions
            .into_iter()
            .group_by(|cells| connecting_cells.iter().map(|i| cells[*i]).collect_vec());
        return grouped
            .into_iter()
            .map(|(key, group)| {
                let solution =
                    QuasiSolution::Plus(group.map(QuasiSolution::Concrete).collect_vec());
                (key, solution)
            })
            .collect();
    }

    let mut split = split.unwrap();
    log(format!(
        "{}Split with connections: {:?}",
        log_prefix, split.connections
    ));
    if split.red.num_cells > split.blue.num_cells {
        split = split.flip();
    }
    log(format!("{}Mask: {:?}", log_prefix, split.colors));
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
    println!(
        "{}Combining {}x{} solutions with {} connections.",
        log_prefix,
        red_solutions.len(),
        blue_solutions.len(),
        connections.len(),
    );
    println!(
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
                if !do_values_satisfy_sum(&values, constraint.sum) {
                    continue 'solutions;
                }
            }
            println!(
                "{}  Combining red {:?} and blue {:?} works.",
                log_prefix, red_connecting_values, blue_connecting_values,
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
