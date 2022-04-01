use crate::board::{Board, Box, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Constraint {
    pub boxes: Vec<usize>,
    pub sum: Value,
}

#[derive(Debug)]
pub struct Input {
    pub num_boxes: usize,
    pub constraints: Vec<Constraint>,
}

type Solution = Vec<Value>;
pub type Output = Vec<Solution>;

impl Board {
    pub fn to_input(&self) -> Input {
        let mut mapping: HashMap<(usize, usize), usize> = HashMap::new();
        let mut empty_box_index = 0;
        let mut wall_boxes = vec![];

        for (y, line) in self.boxes.iter().enumerate() {
            for (x, b) in line.iter().enumerate() {
                match b {
                    Box::Empty => {
                        mapping.insert((x, y), empty_box_index);
                        empty_box_index += 1;
                    }
                    Box::Wall { .. } => wall_boxes.push((x, y)),
                }
            }
        }

        let mut constraints = vec![];

        for (x, y) in wall_boxes {
            let (vertical, horizontal) = match self.boxes[y][x] {
                Box::Empty => panic!(),
                Box::Wall {
                    vertical_sum: v,
                    horizontal_sum: h,
                } => (v, h),
            };
            if let Some(h) = horizontal {
                let mut boxes = vec![];
                let mut x = x + 1;
                while x < self.boxes[0].len() {
                    if let Box::Wall { .. } = self.boxes[y][x] {
                        break;
                    }
                    boxes.push(mapping[&(x, y)]);
                    x += 1;
                }
                constraints.push(Constraint {
                    sum: h,
                    boxes: boxes,
                });
            }
            if let Some(v) = vertical {
                let mut boxes = vec![];
                let mut y = y + 1;
                while y < self.boxes.len() {
                    if let Box::Wall { .. } = self.boxes[y][x] {
                        break;
                    }
                    boxes.push(mapping[&(x, y)]);
                    y += 1;
                }
                constraints.push(Constraint {
                    sum: v,
                    boxes: boxes,
                });
            }
        }

        Input {
            num_boxes: mapping.len(),
            constraints,
        }
    }
}

impl Input {
    pub fn is_solution(&self, attempt: &Solution) -> bool {
        if attempt.len() != self.num_boxes {
            return false;
        }
        if !attempt.iter().all(|i| *i >= 1 && *i <= 9) {
            return false;
        }
        for constraint in self.constraints.iter() {
            let box_values = constraint
                .boxes
                .iter()
                .map(|b| attempt[*b])
                .collect::<HashSet<_>>();
            if box_values.len() < constraint.boxes.len() {
                return false;
            }
            let sum = box_values.into_iter().reduce(|a, b| a + b).unwrap();
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
