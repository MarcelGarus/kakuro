mod board;
mod game;
mod generate;
mod log;
mod solvers;
mod svg;

use crate::{board::*, game::Input};
use itertools::Itertools;
use std::{fs, path::PathBuf, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kakuro", about = "A Kakuro tool.")]
enum KakuroOptions {
    Generate {
        width: usize,
        height: usize,
        fill: f64,

        #[structopt(parse(from_os_str))]
        out: PathBuf,
    },
    Solve {
        solver: String,

        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Bench {
        solver: String,
    },
    Svg {
        #[structopt(parse(from_os_str))]
        file: PathBuf,

        #[structopt(parse(from_os_str))]
        out: PathBuf,
    },
}

fn main() {
    match KakuroOptions::from_args() {
        KakuroOptions::Generate {
            width,
            height,
            fill,
            out,
        } => generate(width, height, fill, out),
        KakuroOptions::Solve { solver, file } => solve(solver, file),
        KakuroOptions::Bench { solver } => benchmark(solver),
        KakuroOptions::Svg { file, out } => svg(&file, &out),
    }
}

fn generate(width: usize, height: usize, fill: f64, out: PathBuf) {
    let board = generate::generate(
        width,
        height,
        (width as f64 * height as f64 * fill) as usize,
    );
    fs::write(out, format!("{}", board).as_bytes()).unwrap();
}

fn solve(solver: String, file: PathBuf) {
    let input = read_kakuro(&file).to_input();
    println!("Input board abstracted to this:");
    println!("{}", input);
    println!();

    println!("Solving.");
    let solutions = raw_solve(&solver, &input);
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
fn raw_solve(solver: &str, input: &Input) -> Vec<Vec<u8>> {
    match solver {
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
    }
}

fn benchmark(solver: String) {
    fn debug_warning() -> bool {
        println!("WARNING: You are running this binary in debug mode.");
        println!("Compile with `cargo build --release` to get a binary actually worth measuring.");
        println!();
        true
    }
    debug_assert!(debug_warning());

    const BENCHMARK_SUITE: [&'static str; 4] = [
        "examples/mini.kakuro",
        "examples/small.kakuro",
        "examples/wikipedia.kakuro",
        "examples/book.kakuro",
    ];
    const NUM_RUNS: usize = 10;

    let inputs = BENCHMARK_SUITE
        .iter()
        .map(|path| read_kakuro(&PathBuf::from(path)).to_input())
        .collect_vec();

    for (i, input) in inputs.iter().enumerate() {
        println!("Input {}.", BENCHMARK_SUITE[i]);
        let mut durations = vec![];
        for i in 0..NUM_RUNS {
            print!("Run {}/{}", i, NUM_RUNS);
            let before = Instant::now();
            raw_solve(&solver, &input);
            let after = Instant::now();
            let runtime = after - before;
            println!(" took {} seconds.", runtime.as_secs_f64());
            durations.push(after - before);
        }
        let durations = durations
            .into_iter()
            .map(|duration| duration.as_micros())
            .collect_vec();
        println!(
            "{} +- {} micro seconds",
            mean(&durations).unwrap(),
            std_deviation(&durations).unwrap()
        );
        println!();
    }
}
fn mean(data: &[u128]) -> Option<f32> {
    let sum = data.iter().sum::<u128>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}
fn std_deviation(data: &[u128]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);

                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

fn svg(file: &PathBuf, out: &PathBuf) {
    let board = read_kakuro(file);
    let svg = svg::svg(&board);
    fs::write(out, svg.as_bytes()).expect(&format!("Couldn't write to {:?}.", out));
}

fn read_kakuro(file: &PathBuf) -> Board {
    let input = fs::read(file).expect(&format!("Couldn't read file: {:?}", file));
    let input =
        String::from_utf8(input).expect(&format!("The file {:?} contains non-UTF8 chars.", file));
    let board = input.parse_board().expect(&format!(
        "The file {:?} doesn't contain a valid Kakuro.",
        file
    ));
    println!("Whole board:");
    println!("{}", board);
    println!();
    board
}
