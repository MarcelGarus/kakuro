use crate::board::{Board, Cell, Value};
use svg::{
    node::{
        self,
        element::{Rectangle, Text, SVG},
    },
    Document, Node,
};

const PADDING: isize = 5;
const GRID_SIZE: isize = 100;
const TEXT_SIZE: isize = 30;

const HEAD: &str = "\
<defs>
    <filter id='shadow' x='0' y='0' width='200%' height='200%'>
        <feOffset result='offOut' in='SourceAlpha' dx='-50px' dy='-50px' />
        <feGaussianBlur result='blurOut' in='offOut' stdDeviation='10' />
        <feBlend in='SourceGraphic' in2='blurOut' mode='normal' />
    </filter>
</defs>
<style>
    .kakuro { background: rgba(0,0,0,0.2); }
    .empty-cell { fill: white; stroke: black; stroke-width: 2px; }\
    .sum { font-family: sans-serif; font-size: 24px; stroke: black; }\
</style>
";

pub fn svg(board: &Board) {
    let mut doc = Document::new()
        .set(
            "viewBox",
            (
                -PADDING,
                -PADDING,
                (board.cells[0].len() + 1) as isize * GRID_SIZE + PADDING,
                (board.cells.len() + 1) as isize * GRID_SIZE + PADDING,
            ),
        )
        .set("class", "kakuro");
    doc.append(node::Text::new(HEAD));

    for (y, row) in board.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            doc = match cell {
                Cell::Wall {
                    vertical_sum,
                    horizontal_sum,
                } => svg_wall_cell(doc, x, y, vertical_sum, horizontal_sum),
                Cell::Empty => svg_empty_cell(doc, x, y),
            };
        }
    }

    svg::save("kakuro.svg", &doc).unwrap();
}

fn svg_empty_cell(doc: SVG, x: usize, y: usize) -> SVG {
    let rect = Rectangle::new()
        .set("x", x as isize * GRID_SIZE)
        .set("y", y as isize * GRID_SIZE)
        .set("width", GRID_SIZE)
        .set("height", GRID_SIZE)
        .set("class", "empty-cell");
    doc.add(rect)
}

fn svg_wall_cell(
    mut doc: SVG,
    x: usize,
    y: usize,
    vertical_sum: &Option<Value>,
    horizontal_sum: &Option<Value>,
) -> SVG {
    if let Some(vertical_sum) = vertical_sum {
        let mut text = Text::new()
            .set("x", x as isize * GRID_SIZE + GRID_SIZE / 2 - 10)
            .set("y", (y + 1) as isize * GRID_SIZE - TEXT_SIZE / 2 + 5)
            .set("text-anchor", "middle")
            .set("class", "sum");
        text.append(node::Text::new(format!("{}", vertical_sum)));
        doc = doc.add(text);
    }
    if let Some(horizontal_sum) = horizontal_sum {
        let mut text = Text::new()
            .set("x", (x + 1) as isize * GRID_SIZE - 5)
            .set(
                "y",
                y as isize * GRID_SIZE + GRID_SIZE / 2 + TEXT_SIZE / 2 - 15,
            )
            .set("text-anchor", "end")
            .set("class", "sum");
        text.append(node::Text::new(format!("{}", horizontal_sum)));
        doc = doc.add(text);
    }
    doc
}
