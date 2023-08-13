use crate::board::{Board, Cell, Value};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct JsonBoard {
    board: Vec<Vec<i32>>,
}

pub trait ImportJsonBoard {
    fn import_json(&self) -> Result<Board, String>;
}
impl ImportJsonBoard for str {
    fn import_json(&self) -> Result<Board, String> {
        let json: JsonBoard = serde_json::from_str(self).map_err(|err| format!("{:?}", err))?;
        println!("Parsed into JSON board.");
        let cells = json
            .board
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| match cell {
                        -1 => Cell::Wall {
                            vertical_sum: None,
                            horizontal_sum: None,
                        },
                        0 => Cell::Empty,
                        _ => {
                            let vertical_sum = cell / 10 % 100;
                            let horizontal_sum = cell / 1000;
                            Cell::Wall {
                                vertical_sum: if vertical_sum == 0 {
                                    None
                                } else {
                                    Some(vertical_sum as Value)
                                },
                                horizontal_sum: if horizontal_sum == 0 {
                                    None
                                } else {
                                    Some(horizontal_sum as Value)
                                },
                            }
                        }
                    })
                    .collect_vec()
            })
            .collect_vec();
        Ok(Board { cells })
    }
}
