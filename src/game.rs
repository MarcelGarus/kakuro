use crate::board::{Board, Cell, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Constraint {
    pub cells: Vec<usize>,
    pub sum: Value,
}

#[derive(Debug)]
pub struct Input {
    pub num_cells: usize,
    pub constraints: Vec<Constraint>,
}

type Solution = Vec<Value>;
pub type Output = Vec<Solution>;

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
                constraints.push(Constraint {
                    sum: h,
                    cells: cells,
                });
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
                constraints.push(Constraint {
                    sum: v,
                    cells: cells,
                });
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
                return false;
            }
            let sum = cell_values.into_iter().reduce(|a, b| a + b).unwrap();
            if sum != constraint.sum {
                return false;
            }
        }
        return true;
    }
}

// . 1 9
// 3 _ _
// . . _
