mod board;
mod game;
mod solvers;

use crate::board::*;
use std::{env::args, fs};

fn main() {
    let input = args().nth(1).expect("Give the input file as an argument.");
    let input = fs::read(input).expect("Couldn't read from input file.");
    let input = String::from_utf8(input).expect("File contains non-utf8 chars.");
    let input = input.parse_board();
    println!("Whole board:");
    println!("{}", input);
    println!();
    let input = input.to_input();
    println!("Abstracted to this:");
    println!("{}", input);
    println!();
    println!("Solving.");
    let solutions: Vec<Vec<u8>> = solvers::gradual_early_abort::solve(&input);
    println!("Done.");
    println!("Solutions:");
    for solution in solutions {
        println!(
            "{}",
            solution
                .iter()
                .map(|number| format!("{}", number))
                .collect::<String>()
        );
    }
}
