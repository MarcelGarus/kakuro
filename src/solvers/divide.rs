use crate::{
    game::{Constraint, Input, Output, Solution, Value},
    log,
};
use itertools::Itertools;

trait InputExt {
    fn is_possible_solution(&self, attempt: &Solution) -> bool;
}
impl InputExt for Input {
    fn is_possible_solution(&self, attempt: &Solution) -> bool {
        for constraint in self.constraints.iter() {
            let digits = constraint.cells.iter().map(|b| attempt[*b]).collect_vec();

            let mut seen = [false; 9];
            for digit in &digits {
                if seen[(digit - 1) as usize] {
                    return false; // A digit appears twice.
                } else {
                    seen[(digit - 1) as usize] = true;
                }
            }

            let sum: Value = digits.iter().sum();
            if sum != constraint.sum {
                return false;
            }
        }
        return true;
    }
}

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

pub fn solve(input: &Input) -> Output {
    solve_rec(input, "")
}

fn solve_rec(input: &Input, log_prefix: &str) -> Vec<Solution> {
    log!(
        "{}Solving input with {} cells and {} constraints.",
        log_prefix,
        input.num_cells,
        input.constraints.len(),
    );
    let split = split(input);

    if split.is_none() {
        log!("{}Solving with simple solver.", log_prefix);
        let solutions = super::sum_reachable_no_set::solve(input);
        log!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        return solutions;
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
    let SplitInput {
        colors,
        red: red_input,
        blue: blue_input,
        connections,
    } = split;

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut red_to_original_mapping = vec![];
    let mut blue_to_original_mapping = vec![];
    for i in 0..input.num_cells {
        match colors[i] {
            Color::Red => red_to_original_mapping.push(i),
            Color::Blue => blue_to_original_mapping.push(i),
        }
    }

    // Solve parts.
    let inner_log_prefix = format!("{}  ", log_prefix);
    let red_solutions = solve_rec(&red_input, &inner_log_prefix);
    let blue_solutions = solve_rec(&blue_input, &inner_log_prefix);

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
    for red_solution in &red_solutions {
        for blue_solution in &blue_solutions {
            let mut attempt = vec![0; input.num_cells];
            for (i, value) in red_solution.iter().enumerate() {
                attempt[red_to_original_mapping[i]] = *value;
            }
            for (i, value) in blue_solution.iter().enumerate() {
                attempt[blue_to_original_mapping[i]] = *value;
            }
            if input.is_possible_solution(&attempt) {
                solutions.push(attempt);
            }
        }
    }

    log!("{}Done. Found {} solutions.", log_prefix, solutions.len());
    solutions
}
