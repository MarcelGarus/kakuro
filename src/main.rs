mod board;
mod game;
mod generate;
mod log;
mod solvers;
mod svg;

use crate::board::*;
use std::{env::args, fs};

fn main() {
    let solver = args().nth(1).expect("Give a solver as the first argument.");
    let input = args()
        .nth(2)
        .expect("Give the input file as the second argument.");

    if solver == "generate" {
        let width: usize = args().nth(3).expect("Give the width.").parse().unwrap();
        let height: usize = args().nth(4).expect("Give the height.").parse().unwrap();
        let fill: f64 = args().nth(5).expect("Give the fill.").parse().unwrap();
        let board = generate::generate(
            width,
            height,
            (width as f64 * height as f64 * fill) as usize,
        );
        fs::write(input, format!("{}", board).as_bytes()).unwrap();
        return;
    }

    println!("Reading from file.");
    let input = fs::read(input).expect("Couldn't read from input file.");
    let input = String::from_utf8(input).expect("File contains non-utf8 chars.");
    println!("Done.");
    println!();

    let input = input.parse_board();
    println!("Whole board:");
    println!("{}", input);
    println!();

    if solver == "svg" {
        svg::svg(&input);
        return;
    }

    let input = input.to_input();
    println!("Abstracted to this:");
    println!("{}", input);
    println!();

    println!("Solving.");
    let solutions: Vec<Vec<u8>> = match solver.as_str() {
        "naive" => solvers::naive::solve(&input),
        "gradual" => solvers::gradual::solve(&input),
        "sum_reachable" => solvers::sum_reachable::solve(&input),
        "prioritize" => solvers::prioritize::solve(&input),
        "prioritize_no_set" => solvers::prioritize_no_set::solve(&input),
        "sum_reachable_no_set" => solvers::sum_reachable_no_set::solve(&input),
        "divide" => solvers::divide::solve(&input),
        "combine_by_connecting_cells" => solvers::combine_by_connecting_cells::solve(&input),
        "lazy" => solvers::lazy::solve(&input),
        "propagate_constraints" => solvers::propagate_constraints::solve(&input),
        _ => panic!("Unknown solver {}.", solver),
    };
    println!("Done.");
    println!();

    if solutions.len() == 1 {
        println!("One solution:");
    } else {
        println!("{} solutions:", solutions.len());
    }
    for solution in solutions {
        print!(
            "{}",
            solution
                .iter()
                .map(|number| format!("{}", number))
                .collect::<String>()
        );
        if !input.is_solution(&solution) {
            print!(" (invalid)");
        }
        println!();
    }
}
