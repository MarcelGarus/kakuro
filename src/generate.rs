use itertools::Itertools;
use rand::Rng;

use crate::board::{self, Value};

#[derive(Debug, Clone, Copy)]
enum Cell {
    Wall,
    Value(Value),
}

fn is_board_valid(board: &[Vec<Cell>]) -> bool {
    for row in board {
        if !is_valid(&row) {
            return false;
        }
    }
    for i in 0..board[0].len() {
        let column = board.iter().map(|row| row[i]).collect_vec();
        if !is_valid(&column) {
            return false;
        }
    }
    return true;
}
fn is_valid(cells: &[Cell]) -> bool {
    let mut run = vec![];
    for cell in cells {
        match cell {
            Cell::Wall => {
                if !run.is_empty() {
                    if !is_run_valid(&run) {
                        return false;
                    }
                    run.clear();
                }
            }
            Cell::Value(digit) => run.push(*digit),
        }
    }
    if !run.is_empty() {
        if !is_run_valid(&run) {
            return false;
        }
    }
    return true;
}
fn is_run_valid(run: &[Value]) -> bool {
    let mut seen = [false; 9];
    for digit in run {
        if seen[(digit - 1) as usize] {
            return false; // A digit appears twice.
        } else {
            seen[(digit - 1) as usize] = true;
        }
    }
    return true;
}

struct Wall {
    horizontal_sum: Option<Value>,
    vertical_sum: Option<Value>,
}

pub fn generate(width: usize, height: usize, numbers: usize) -> board::Board {
    let mut board = {
        let mut cells = vec![];
        for _y in 0..height {
            let mut row = vec![];
            for _x in 0..width {
                row.push(Cell::Wall);
            }
            cells.push(row);
        }
        cells
    };

    let mut rand = rand::thread_rng();
    while Itertools::flatten(board.iter())
        .filter(|cell| matches!(cell, Cell::Value(_)))
        .count()
        < numbers
    {
        let x = rand.gen_range(0..width);
        let y = rand.gen_range(0..height);
        let digit: Value = rand.gen_range(1..=9);

        if !matches!(board[y][x], Cell::Wall) {
            continue;
        }
        board[y][x] = Cell::Value(digit);
        if !is_board_valid(&board) {
            println!("Board invalid: {:?}", board);
            board[y][x] = Cell::Wall;
        }
    }

    // Turn board into wall grid.
    let mut walls = vec![];
    for _y in 0..=height {
        let mut row = vec![];
        for _x in 0..=width {
            row.push(Wall {
                vertical_sum: None,
                horizontal_sum: None,
            });
        }
        walls.push(row);
    }
    for (y, row) in board.iter().enumerate() {
        let mut run_start_x = 0;
        let mut run = vec![];
        for (x, cell) in row.iter().enumerate() {
            match cell {
                Cell::Wall => {
                    if !run.is_empty() {
                        walls[y + 1][run_start_x].horizontal_sum = Some(run.iter().sum());
                        run.clear();
                    }
                    run_start_x = x + 1;
                }
                Cell::Value(digit) => run.push(*digit),
            }
        }
        if !run.is_empty() {
            walls[y + 1][run_start_x].horizontal_sum = Some(run.iter().sum());
        }
    }
    for x in 0..board[0].len() {
        let column = board.iter().map(|row| row[x]).collect_vec();
        let mut run_start_y = 0;
        let mut run = vec![];
        for (y, cell) in column.iter().enumerate() {
            match cell {
                Cell::Wall => {
                    if !run.is_empty() {
                        walls[run_start_y][x + 1].vertical_sum = Some(run.iter().sum());
                        run.clear();
                    }
                    run_start_y = y + 1;
                }
                Cell::Value(digit) => run.push(*digit),
            }
        }
        if !run.is_empty() {
            walls[run_start_y][x + 1].vertical_sum = Some(run.iter().sum());
        }
    }

    // Build new board from walls and cells.
    let mut cells = vec![];
    for y in 0..=height {
        let mut row = vec![];
        for x in 0..=width {
            row.push(
                if x == 0 || y == 0 || matches!(board[y - 1][x - 1], Cell::Wall) {
                    let wall = &walls[y][x];
                    board::Cell::Wall {
                        horizontal_sum: wall.horizontal_sum,
                        vertical_sum: wall.vertical_sum,
                    }
                } else {
                    board::Cell::Empty
                },
            );
        }
        cells.push(row);
    }
    board::Board { cells }
}
