mod board;
mod game;
mod generate;
mod import;
mod log;
mod solvers;
mod svg;

use crate::{board::*, game::Input};
use import::ImportJsonBoard;
use itertools::Itertools;
use std::{fs, io::Write, path::PathBuf, time::Instant};
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
    Import {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Solve {
        solver: String,

        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Bench {
        solver: String,

        #[structopt(parse(from_os_str))]
        file: Option<PathBuf>,
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
        KakuroOptions::Import { file } => import(file),
        KakuroOptions::Solve { solver, file } => solve(solver, file),
        KakuroOptions::Bench { solver, file } => benchmark(solver, file),
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

fn import(file: PathBuf) {
    let input = fs::read(file.clone()).expect(&format!("Couldn't read file: {:?}", file));
    let input =
        String::from_utf8(input).expect(&format!("The file {:?} contains non-UTF8 chars.", file));
    let board = input.import_json().expect(&format!(
        "The file {:?} doesn't contain a valid JSON Kakuro.",
        file
    ));
    let mut out = file;
    assert!(out.set_extension("kakuro"));
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
        "connecting_cells" => solvers::connecting_cells::solve(&input),
        "lazy" => solvers::lazy::solve(&input),
        "propagate_constraints" => solvers::propagate_constraints::solve(&input),
        _ => panic!("Unknown solver {}.", solver),
    }
}

fn benchmark(solver: String, file: Option<PathBuf>) {
    fn debug_warning() -> bool {
        println!("WARNING: You are running this binary in debug mode.");
        println!("Compile with `cargo build --release` to get a binary actually worth measuring.");
        println!();
        true
    }
    debug_assert!(debug_warning());

    const BENCHMARK_SUITE: [&'static str; 7] = [
        "examples/mini.kakuro",
        "examples/small.kakuro",
        "examples/wikipedia.kakuro",
        "examples/15x15.kakuro",
        "examples/20x20.kakuro",
        "examples/30x30.kakuro",
        "examples/book.kakuro",
    ];
    const NUM_RUNS: usize = 10;

    let inputs = if let Some(file) = file {
        vec![file]
    } else {
        BENCHMARK_SUITE
            .iter()
            .map(|path| PathBuf::from(path))
            .collect_vec()
    }
    .into_iter()
    .map(|file| read_kakuro(&file).to_input())
    .collect_vec();

    // Warm up for 10 seconds.
    println!("Warming up.");
    let warmup_start = chrono::Utc::now();
    while chrono::Utc::now() < warmup_start + chrono::Duration::seconds(10) {
        let input = &inputs[0];
        raw_solve(&solver, &input);
    }
    println!();

    for (i, input) in inputs.iter().enumerate() {
        println!("Input {}.", BENCHMARK_SUITE[i]);
        let mut durations = vec![];
        for i in 0..NUM_RUNS {
            print!(
                "Solving run {}/{} started at {}.",
                i,
                NUM_RUNS,
                chrono::Local::now()
            );
            std::io::stdout().flush().expect("Couldn't flush stdout.");
            let before = Instant::now();
            raw_solve(&solver, &input);
            let after = Instant::now();
            let runtime = after - before;
            println!(" It took {} seconds.", runtime.as_secs_f64());
            durations.push(after - before);
        }
        let durations = durations
            .into_iter()
            .map(|duration| duration.as_nanos())
            .collect_vec();
        println!(
            "{} +- {} % nano seconds; {} - {} nano seconds",
            mean(&durations).unwrap(),
            std_deviation(&durations).unwrap() / mean(&durations).unwrap() * 100.0,
            durations.iter().min().unwrap(),
            durations.iter().max().unwrap(),
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
