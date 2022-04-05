//! A game is a more abstracted version of a board. The concrete arrangements of
//! cells on a 2D grid doesn't matter. Instead, it only contains constraints
//! that are imposed on subsets of the cells.

use crate::board::{self, Board, Cell};
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
    pub fn is_solution(&self, attempt: &Solution) -> bool {
        if attempt.len() != self.num_cells {
            return false;
        }
        if !attempt.iter().all(|i| *i >= 1 && *i <= 9) {
            return false;
        }
        for constraint in self.constraints.iter() {
            let cell_values = constraint
                .cells
                .iter()
                .map(|b| attempt[*b])
                .collect::<HashSet<_>>();
            if cell_values.len() < constraint.cells.len() {
                return false; // A number appears twice.
            }
            let sum = cell_values.into_iter().reduce(|a, b| a + b).unwrap();
            if sum != constraint.sum {
                return false;
            }
        }
        return true;
    }
}
