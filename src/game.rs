//! A game is a more abstracted version of a board. The concrete arrangements of
//! cells on a 2D grid doesn't matter. Instead, it only contains constraints
//! that are imposed on subsets of the cells.

use crate::board::{self, Board, Cell};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
};

pub type Value = board::Value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Constraint {
    pub cells: Vec<usize>,
    pub sum: Value,
}

#[derive(Debug)]
pub struct Input {
    pub num_cells: usize,
    pub constraints: Vec<Constraint>,
}

pub type Solution = Vec<Value>;
pub type Output = Vec<Solution>;

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} cells", self.num_cells)?;
        for (i, constraint) in self.constraints.iter().enumerate() {
            write!(
                f,
                "{} should be sum from {:?}{}",
                constraint.sum,
                constraint.cells,
                if i == self.constraints.len() - 1 {
                    ""
                } else {
                    "\n"
                },
            )?;
        }
        Ok(())
    }
}

impl Board {
    pub fn to_input(&self) -> Input {
        let mut mapping: HashMap<(usize, usize), usize> = HashMap::new();
        let mut empty_cell_index = 0;
        let mut wall_cells = vec![];

        for (y, line) in self.cells.iter().enumerate() {
            for (x, b) in line.iter().enumerate() {
                match b {
                    Cell::Empty => {
                        mapping.insert((x, y), empty_cell_index);
                        empty_cell_index += 1;
                    }
                    Cell::Wall { .. } => wall_cells.push((x, y)),
                }
            }
        }

        let mut constraints = vec![];

        for (x, y) in wall_cells {
            let (vertical, horizontal) = match self.cells[y][x] {
                Cell::Empty => panic!(),
                Cell::Wall {
                    vertical_sum: v,
                    horizontal_sum: h,
                } => (v, h),
            };
            if let Some(h) = horizontal {
                let mut cells = vec![];
                let mut x = x + 1;
                while x < self.cells[0].len() {
                    if let Cell::Wall { .. } = self.cells[y][x] {
                        break;
                    }
                    cells.push(mapping[&(x, y)]);
                    x += 1;
                }
                if cells.is_empty() {
                    panic!(
                        "Constraint with sum {} starting at {}, {} has no cells.",
                        h,
                        x - 1,
                        y
                    );
                }
                constraints.push(Constraint { sum: h, cells });
            }
            if let Some(v) = vertical {
                let mut cells = vec![];
                let mut y = y + 1;
                while y < self.cells.len() {
                    if let Cell::Wall { .. } = self.cells[y][x] {
                        break;
                    }
                    cells.push(mapping[&(x, y)]);
                    y += 1;
                }
                if cells.is_empty() {
                    panic!(
                        "Constraint with sum {} starting at {}, {} has no cells.",
                        v,
                        x,
                        y - 1
                    );
                }
                constraints.push(Constraint { sum: v, cells });
            }
        }

        Input {
            num_cells: mapping.len(),
            constraints,
        }
    }
}

impl Input {
    pub fn is_solution(&self, solution: &Solution) -> bool {
        solution.len() == self.num_cells
            && solution.iter().all(|number| (1..=9).contains(number))
            && self
                .constraints
                .iter()
                .all(|constraint| constraint.is_solution(solution))
    }
}
impl Constraint {
    pub fn is_solution(&self, solution: &Solution) -> bool {
        let digits = self.cells.iter().map(|i| solution[*i]).collect_vec();
        let unique_digits = digits.iter().collect::<HashSet<_>>();

        if unique_digits.len() < digits.len() {
            false // A digit appears twice.
        } else {
            digits.iter().sum::<Value>() == self.sum
        }
    }
}
