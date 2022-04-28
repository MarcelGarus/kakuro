use crate::{
    game::{self, Output, Solution, Value},
    log,
};
use arrayvec::ArrayVec;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{
    cmp::{max, min},
    fmt::{self, Display},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    cells: ArrayVec<usize, 9>,
    min: Value,
    max: Value,
}
impl From<game::Constraint> for Constraint {
    fn from(constraint: game::Constraint) -> Self {
        Self {
            cells: constraint.cells.into_iter().collect(),
            min: constraint.sum,
            max: constraint.sum,
        }
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
    // The colors vector has the length of the original input and assigns each
    // cell a color.
    colors: Vec<Color>,

    // A vector that maps cell indizes from the original input to the cell
    // indizes in the smaller parts.
    index_mapping: Vec<usize>,

    red_constraints: Vec<Constraint>,
    blue_constraints: Vec<Constraint>,
    connections: Vec<Constraint>,
}
impl SplitInput {
    fn flip(self) -> Self {
        Self {
            colors: self.colors.into_iter().map(|color| color.flip()).collect(),
            red_constraints: self.blue_constraints,
            blue_constraints: self.red_constraints,
            ..self
        }
    }
}

fn split(num_cells: usize, constraints: &[Constraint]) -> Option<SplitInput> {
    for num_connections in 0..constraints.len() {
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

            // Start with all cells blue, then flood fill from the first cell,
            // following constraints.
            let mut colors = vec![Color::Blue; num_cells];

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
                for i in 0..num_cells {
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
            let all_constraints =
                add_slices_to_vec(&connecting_constraints, &remaining_constraints);
            return Some(SplitInput {
                colors: colors.clone(),
                index_mapping: index_mapping.clone(),
                red_constraints: constraints_for_color(
                    Color::Red,
                    &all_constraints,
                    &colors,
                    &index_mapping,
                ),
                blue_constraints: constraints_for_color(
                    Color::Blue,
                    &all_constraints,
                    &colors,
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
    Plus(Vec<Rc<QuasiSolution>>),
    Product {
        colors: Vec<Color>,
        red: Rc<QuasiSolution>,
        blue: Rc<QuasiSolution>,
    },
}
impl QuasiSolution {
    fn size(&self) -> usize {
        match self {
            QuasiSolution::Concrete(_) => 1,
            QuasiSolution::Plus(others) => others.iter().map(|it| it.size()).sum(),
            QuasiSolution::Product { red, blue, .. } => red.size() * blue.size(),
        }
    }
    fn simplify(&self) -> Self {
        match self {
            QuasiSolution::Concrete(concrete) => QuasiSolution::Concrete(concrete.clone()),
            QuasiSolution::Plus(children) => {
                let mut children = children.into_iter().map(|it| it.simplify()).collect_vec();
                if children.len() == 1 {
                    children.pop().unwrap()
                } else {
                    QuasiSolution::Plus(children.into_iter().map(Rc::new).collect())
                }
            }
            QuasiSolution::Product { colors, red, blue } => {
                let red = Rc::new(red.simplify());
                let blue = Rc::new(blue.simplify());
                if red.size() == 1 || blue.size() == 1 {
                    let mut red = red.build().pop().unwrap();
                    let mut blue = blue.build().pop().unwrap();
                    let mut solution = vec![];
                    for color in colors {
                        solution.push(match color {
                            Color::Red => red.remove(0),
                            Color::Blue => blue.remove(0),
                        });
                    }
                    QuasiSolution::Concrete(solution)
                } else {
                    QuasiSolution::Product {
                        colors: colors.clone(),
                        red: red.clone(),
                        blue: blue.clone(),
                    }
                }
            }
        }
    }
    fn build(&self) -> Vec<Solution> {
        match self {
            QuasiSolution::Concrete(concrete) => vec![concrete.clone()],
            QuasiSolution::Plus(children) => {
                Iterator::flatten(children.into_iter().map(|it| it.build())).collect_vec()
            }
            QuasiSolution::Product { colors, red, blue } => {
                let all_red = red.build();
                let all_blue = blue.build();
                let mut solutions = vec![];
                for red in all_red.clone() {
                    for blue in all_blue.clone() {
                        let mut solution = vec![];
                        let mut red = red.clone();
                        let mut blue = blue.clone();
                        for color in colors {
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
impl Display for QuasiSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, "")
    }
}
impl QuasiSolution {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indentation: &str) -> fmt::Result {
        match self {
            QuasiSolution::Concrete(concrete) => {
                writeln!(
                    f,
                    "{}{}",
                    indentation,
                    concrete.iter().map(|digit| format!("{}", digit)).join("")
                )?;
            }
            QuasiSolution::Plus(plus) => {
                writeln!(f, "{}+ -> {}", indentation, self.size())?;
                for solution in plus {
                    solution.fmt_indented(f, &format!(" {}", indentation))?;
                }
            }
            QuasiSolution::Product { colors, red, blue } => {
                writeln!(
                    f,
                    "{}*{} -> {}",
                    indentation,
                    colors
                        .iter()
                        .map(|color| match color {
                            Color::Red => "r",
                            Color::Blue => "b",
                        })
                        .join(""),
                    self.size()
                )?;
                red.fmt_indented(f, &format!(" {}", indentation))?;
                blue.fmt_indented(f, &format!(" {}", indentation))?;
            }
        }
        Ok(())
    }
}

fn add_slices_to_vec<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut v = vec![];
    for t in a {
        v.push(t.clone());
    }
    for t in b {
        v.push(t.clone());
    }
    v
}
fn add_slices_to_array_vec<T: Clone>(a: &[T], b: &[T]) -> ArrayVec<T, 9> {
    let mut v = ArrayVec::<T, 9>::new();
    for t in a {
        v.push(t.clone());
    }
    for t in b {
        v.push(t.clone());
    }
    v
}

pub fn solve_one_cell(constraints: &[Constraint]) -> Output {
    let mut min_digit: Value = 1;
    let mut max_digit: Value = 9;
    for constraint in constraints {
        min_digit = max(constraint.min, min_digit);
        max_digit = min(constraint.max, max_digit);
    }
    (min_digit..=max_digit).map(|it| vec![it]).collect()
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

fn do_digits_satisfy_sum(digits: &[Value], min: Value, max: Value) -> bool {
    let mut seen = [false; 9];
    for digit in digits {
        if seen[(digit - 1) as usize] {
            return false; // A digit appears twice.
        } else {
            seen[(digit - 1) as usize] = true;
        }
    }

    let sum = digits.into_iter().sum::<Value>();
    min <= sum && sum <= max
}

fn translate_constraint(
    constraint: &Constraint,
    color: Color,
    colors: &[Color],
    mapping: &[usize],
) -> Constraint {
    let own_cells: ArrayVec<usize, 9> = constraint
        .cells
        .iter()
        .filter(|it| colors[**it] == color)
        .map(|cell| mapping[*cell])
        .collect();
    let num_other_cells = constraint.cells.len() - own_cells.len();
    Constraint {
        cells: own_cells,
        min: max(0, constraint.min as i8 - MAXES[num_other_cells] as i8) as Value,
        max: max(0, constraint.max as i8 - MINS[num_other_cells] as i8) as Value,
    }
}

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
    // log!("Solutions:\n{}", solutions));
    // log!("There are {} solutions.", solutions.size()));
    // log!("Simplified:"));
    let solutions = solutions.simplify();
    // log!("There are {} simple solutions.", solutions.size()));
    // log!("{}", &solutions));
    solutions.build()
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
) -> FxHashMap<Vec<ArrayVec<Value, 9>>, Rc<QuasiSolution>> {
    log!(
        "{}Solving input with {} cells and {} constraints to pay attention to: {:?}",
        log_prefix,
        num_cells,
        all_constraints.len(),
        all_constraints,
    );
    let split = split(num_cells, all_constraints);

    if split.is_none() {
        log!("{}Solving with simple algorithm.", log_prefix);
        debug_assert_eq!(num_cells, 1);
        let solutions = solve_one_cell(all_constraints);
        log!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        let mut grouped = FxHashMap::<Vec<ArrayVec<Value, 9>>, Rc<QuasiSolution>>::default();
        for solution in solutions {
            let key = connecting_constraints
                .iter()
                .map(|connection| {
                    let mut cells: ArrayVec<Value, 9> =
                        connection.cells.iter().map(|i| solution[*i]).collect();
                    cells.sort();
                    cells
                })
                .collect_vec();
            grouped
                .entry(key)
                .and_modify(|existing_solution| {
                    *existing_solution = Rc::new(QuasiSolution::Plus(vec![
                        Rc::new(QuasiSolution::Concrete(solution.clone())),
                        existing_solution.clone(),
                    ]));
                })
                .or_insert(Rc::new(QuasiSolution::Concrete(solution)));
        }
        return grouped;
    }

    let mut split = split.unwrap();
    log!(
        "{}Split with connections: {:?}",
        log_prefix,
        split.connections
    );
    let more_red_than_blue =
        split.colors.iter().filter(|it| **it == Color::Red).count() > num_cells / 2;
    if more_red_than_blue {
        split = split.flip();
    }
    log!("{}Mask: {:?}", log_prefix, split.colors);
    let SplitInput {
        colors,
        index_mapping,
        red_constraints,
        blue_constraints,
        connections: split_connecting_constraints,
    } = split;

    // Mappings from part cell indizes to the cell indizes in the combined game.
    let mut red_mapping = vec![];
    let mut blue_mapping = vec![];
    for i in 0..num_cells {
        match colors[i] {
            Color::Red => red_mapping.push(i),
            Color::Blue => blue_mapping.push(i),
        }
    }

    // Solve parts.
    let inner_log_prefix = format!("{}  ", log_prefix);
    let red_solutions = solve_rec(
        red_mapping.len(),
        &red_constraints,
        &add_slices_to_vec(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| {
                translate_constraint(&constraint, Color::Red, &colors, &index_mapping)
            })
            .collect_vec(),
        &inner_log_prefix,
    );
    let blue_solutions = solve_rec(
        blue_mapping.len(),
        &blue_constraints,
        &add_slices_to_vec(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| {
                translate_constraint(&constraint, Color::Blue, &colors, &index_mapping)
            })
            .collect_vec(),
        &inner_log_prefix,
    );

    // Combine results.
    log!(
        "{}Combining {}x{} solutions with {} connections.",
        log_prefix,
        red_solutions.len(),
        blue_solutions.len(),
        split_connecting_constraints.len(),
    );
    log!(
        "{}Naively joining solutions would require checking {} candidates.",
        log_prefix,
        red_solutions.len() * blue_solutions.len()
    );
    let mut solutions = vec![];
    for (red_connecting_values, red_solution) in &red_solutions {
        'solutions: for (blue_connecting_values, blue_solution) in &blue_solutions {
            for ((constraint, red_values), blue_values) in split_connecting_constraints
                .iter()
                .zip(
                    red_connecting_values
                        .iter()
                        .skip(connecting_constraints.len()),
                )
                .zip(
                    blue_connecting_values
                        .iter()
                        .skip(connecting_constraints.len()),
                )
            {
                let values = add_slices_to_array_vec(red_values, blue_values);
                if !do_digits_satisfy_sum(&values, constraint.min, constraint.max) {
                    continue 'solutions;
                }
            }
            log!(
                "{}  Combining red {:?} and blue {:?} works.",
                log_prefix,
                red_connecting_values,
                blue_connecting_values,
            );
            let key = connecting_constraints
                .iter()
                .zip(red_connecting_values)
                .zip(blue_connecting_values)
                .map(|((_, red_values), blue_values)| {
                    add_slices_to_array_vec(red_values, blue_values)
                })
                .collect_vec();
            let value = Rc::new(QuasiSolution::Product {
                colors: colors.clone(),
                red: red_solution.clone(),
                blue: blue_solution.clone(),
            });
            solutions.push((key, value));
        }
    }

    let mut grouped = FxHashMap::<Vec<ArrayVec<Value, 9>>, Rc<QuasiSolution>>::default();
    for (key, solution) in solutions {
        grouped
            .entry(key)
            .and_modify(|existing_solution| {
                *existing_solution = Rc::new(QuasiSolution::Plus(vec![
                    solution.clone(),
                    existing_solution.clone(),
                ]));
            })
            .or_insert(solution);
    }

    log!(
        "{}Done. Found {} solutions.",
        log_prefix,
        grouped
            .iter()
            .map(|(_, solution)| solution.size())
            .sum::<usize>()
    );
    grouped
}
