use crate::board::{Board, Cell};

// Note: The grid size is 100.

pub fn svg(board: &Board) -> String {
    let height = board.cells.len();
    let width = board.cells[0].len();

    let mut lines = vec![
        // We assume that the left and top row contain no empty cells, only
        // clues. That's why we start at 50 50.
        format!(
            "<svg class=\"board\" viewBox=\"50 50 {} {}\" xmlns=\"http://www.w3.org/2000/svg\" >",
            100 * width,
            100 * height
        ),
        "<style>".to_string(),
        format!("svg {{ max-height: {}em; }}", 3 * height),
        ".cell { fill: white; stroke: black; stroke-width: 2px; }".to_string(),
        ".clue { fill: black; font-family: sans-serif; font-size: 24px; }".to_string(),
        "</style>".to_string(),
    ];

    for (y, row) in board.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            match cell {
                Cell::Wall {
                    vertical_sum,
                    horizontal_sum,
                } => {
                    if let Some(vertical_sum) = vertical_sum {
                        lines.push(format!("<text class=\"clue\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
                            x * 100 + 100 / 2 - 10,
                            (y + 1) * 100 - 30 / 2 + 5,
                            vertical_sum
                        ));
                    }
                    if let Some(horizontal_sum) = horizontal_sum {
                        lines.push(format!(
                            "<text class=\"clue\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
                            (x + 1) as isize * 100 - 5,
                            y as isize * 100 + 100 / 2 + 30 / 2 - 15,
                            horizontal_sum
                        ));
                    }
                }
                Cell::Empty => lines.push(format!(
                    "<rect class=\"cell\" x=\"{}\" y=\"{}\" width=\"100\" height=\"100\" />",
                    100 * x,
                    100 * y
                )),
            };
        }
    }

    lines.push("</svg>".to_string());

    lines.join("\n")
}
