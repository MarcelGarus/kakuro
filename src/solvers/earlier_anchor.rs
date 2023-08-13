use crate::{
    game::{self, Output, Solution, Value},
    log,
};
use arrayvec::ArrayVec;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::{
    cmp::max,
    fmt::{self, Display},
    rc::Rc,
};

type Vec9<T> = ArrayVec<T, 9>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    cells: Vec9<usize>,
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
    for num_connections in 0..2 {
        for connecting_constraints in constraints.iter().combinations(num_connections) {
            let connecting_constraints: Vec<_> = connecting_constraints
                .into_iter()
                .map(|it| it.clone())
                .collect();
            let remaining_constraints: Vec<_> = constraints
                .iter()
                .filter(|it| !connecting_constraints.contains(it))
                .map(|it| it.clone())
                .collect();

            // Start with all cells blue, then flood fill from the first cell,
            // following constraints.
            let mut colors: Vec<Color> = vec![Color::Blue; num_cells];

            let mut dirty_queue: Vec<usize> = vec![0];
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

            {
                // Abort if this is a stupid split.
                let num_red = colors.iter().filter(|color| **color == Color::Red).count();
                let num_blue = colors.len() - num_red;
                if num_red < 3 || num_blue < 3 {
                    continue;
                }
            }

            // A vector that maps cell indizes from the original input to the
            // cell indizes in the smaller parts.
            let index_mapping = {
                let mut red_counter = 0;
                let mut blue_counter = 0;
                let mut mapping: Vec<usize> = Default::default();
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
                add_slices_to_small_vec(&connecting_constraints, &remaining_constraints);
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
    Concrete(Vec<Value>),
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
                let mut children: Vec<_> = children.into_iter().map(|it| it.simplify()).collect();
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
                    let mut solution: Vec<Value> = Default::default();
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
            QuasiSolution::Concrete(concrete) => vec![concrete.to_vec()],
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

fn add_slices_to_vec9<T: Clone>(a: &[T], b: &[T]) -> Vec9<T> {
    let mut v: Vec9<_> = Default::default();
    for t in a {
        v.push(t.clone());
    }
    for t in b {
        v.push(t.clone());
    }
    v
}
fn add_slices_to_small_vec<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut v: Vec<T> = Default::default();
    for t in a {
        v.push(t.clone());
    }
    for t in b {
        v.push(t.clone());
    }
    v
}

mod simple_solver {
    use super::*;

    type Game = Vec<Cell>;
    type Cell = Option<Value>;

    trait ConstraintExt {
        fn is_satisfied_by(&self, attempt: &Game) -> bool;
    }
    impl ConstraintExt for Constraint {
        fn is_satisfied_by(&self, attempt: &Game) -> bool {
            let cells = self.cells.iter().map(|b| attempt[*b]).collect_vec();
            let digits = cells.into_iter().filter_map(|it| it).collect_vec();

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

            let unused_digits = (1..=9u8)
                .filter(|digit| !seen[(digit - 1) as usize])
                .collect_vec();
            let is_sum_reachable = unused_digits
                .into_iter()
                .combinations(self.cells.len() - digits.len())
                .map(|additional_digits| sum + additional_digits.into_iter().sum::<Value>())
                .any(|possible_sum| possible_sum >= self.min && possible_sum <= self.max);
            return is_sum_reachable;
        }
    }

    pub fn solve(num_cells: usize, constraints: &[Constraint]) -> Output {
        let mut attempt = vec![None; num_cells];
        let mut solutions = vec![];
        let mut affected_constraints = FxHashMap::default();
        for (i, constraint) in constraints.iter().enumerate() {
            for cell in &constraint.cells {
                affected_constraints.entry(*cell).or_insert(vec![]).push(i);
            }
        }
        solve_rec(
            num_cells,
            constraints,
            &affected_constraints,
            &mut attempt,
            &mut solutions,
        );
        solutions
    }

    fn solve_rec(
        num_cells: usize,
        constraints: &[Constraint],
        affected_constraints: &FxHashMap<usize, Vec<usize>>,
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
                    let constraint = &constraints[*constraint_index];
                    if !constraint.is_satisfied_by(attempt) {
                        continue 'candidates;
                    }
                }
                solve_rec(
                    num_cells,
                    constraints,
                    affected_constraints,
                    attempt,
                    solutions,
                );
            }
            attempt[index] = None;
        } else {
            solutions.push(attempt.iter().map(|cell| cell.unwrap()).collect());
        }
    }
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
    let own_cells: Vec9<usize> = constraint
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
    let constraints: Vec<Constraint> = input
        .constraints
        .iter()
        .map(|it| it.clone().into())
        .collect();
    let mut solutions = solve_rec(input.num_cells, &constraints, &[], "");
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
) -> FxHashMap<Vec<Vec9<Value>>, Rc<QuasiSolution>> {
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
        let solutions = simple_solver::solve(num_cells, all_constraints);
        log!("{}Done. Found {} solutions.", log_prefix, solutions.len());
        let mut grouped = FxHashMap::<Vec<Vec9<Value>>, Rc<QuasiSolution>>::default();
        for solution in solutions {
            let key = connecting_constraints
                .iter()
                .map(|connection| {
                    let mut cells: Vec9<Value> =
                        connection.cells.iter().map(|i| solution[*i]).collect();
                    cells.sort();
                    cells
                })
                .collect();
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
    let mut red_mapping: Vec<_> = Default::default();
    let mut blue_mapping: Vec<_> = Default::default();
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
        &add_slices_to_small_vec(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| {
                translate_constraint(&constraint, Color::Red, &colors, &index_mapping)
            })
            .collect::<Vec<_>>(),
        &inner_log_prefix,
    );
    let blue_solutions = solve_rec(
        blue_mapping.len(),
        &blue_constraints,
        &add_slices_to_small_vec(&connecting_constraints, &split_connecting_constraints)
            .into_iter()
            .map(|constraint| {
                translate_constraint(&constraint, Color::Blue, &colors, &index_mapping)
            })
            .collect::<Vec<_>>(),
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
    let mut solutions: Vec<(Vec<Vec9<Value>>, Rc<QuasiSolution>)> = Default::default();
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
                let values = add_slices_to_small_vec(red_values, blue_values);
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
            let key: Vec<Vec9<Value>> = connecting_constraints
                .iter()
                .zip(red_connecting_values)
                .zip(blue_connecting_values)
                .map(|((_, red_values), blue_values)| add_slices_to_vec9(red_values, blue_values))
                .collect();
            let value = Rc::new(QuasiSolution::Product {
                colors: colors.clone(),
                red: red_solution.clone(),
                blue: blue_solution.clone(),
            });
            solutions.push((key, value));
        }
    }

    let mut grouped = FxHashMap::<Vec<Vec9<Value>>, Rc<QuasiSolution>>::default();
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
