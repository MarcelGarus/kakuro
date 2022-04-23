//! This module contains the definition of a board. This corresponds to a 2D
//! layout of cells, just like you would see them on paper.

use itertools::Itertools;
use std::fmt::{self, Display, Formatter};

pub type Value = u8;
pub enum Cell {
    Wall {
        vertical_sum: Option<Value>,
        horizontal_sum: Option<Value>,
    },
    Empty,
}
pub struct Board {
    pub cells: Vec<Vec<Cell>>, // Outer is vertical, inner horizontal.
}

pub trait ParseBoard {
    fn parse_board(&self) -> Result<Board, String>;
}
impl ParseBoard for str {
    fn parse_board(&self) -> Result<Board, String> {
        let cells = self
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                line.split(' ')
                    .filter(|word| !word.is_empty())
                    .map(|word| {
                        if word.chars().all(|c| c == '_') {
                            Cell::Empty
                        } else {
                            fn parse_sum(sum_str: &str) -> Option<Value> {
                                if sum_str.is_empty() {
                                    None
                                } else {
                                    Some(
                                        sum_str
                                            .parse()
                                            .expect(&format!("Invalid sum {:?}.", sum_str)),
                                    )
                                }
                            }
                            let parts = word.split('\\').collect_vec();
                            if parts.len() != 2 {
                                panic!("Unknown cell {:?}!", word);
                            }
                            Cell::Wall {
                                vertical_sum: parse_sum(parts[0]),
                                horizontal_sum: parse_sum(parts[1]),
                            }
                        }
                    })
                    .collect_vec()
            })
            .collect_vec();
        Ok(Board { cells })
    }
}

impl Display for Cell {
    // Always 5 chars.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Wall {
                vertical_sum,
                horizontal_sum,
            } => match (vertical_sum, horizontal_sum) {
                (vertical, horizontal) => {
                    fn fmt_sum(s: &Option<Value>) -> String {
                        match s {
                            None => "".to_string(),
                            Some(s) => format!("{}", s),
                        }
                    }
                    write!(f, "{:>2}\\{:2}", fmt_sum(vertical), fmt_sum(horizontal))
                }
            },
            Cell::Empty => "_____".fmt(f),
        }
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let num_lines = self.cells.len();
        for (i, line) in self.cells.iter().enumerate() {
            for cell in line {
                cell.fmt(f)?;
                ' '.fmt(f)?;
            }
            if i < num_lines - 1 {
                '\n'.fmt(f)?;
            }
        }
        Ok(())
    }
}
