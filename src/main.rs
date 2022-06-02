#![feature(path_file_prefix)]
#![feature(const_for)]

#[macro_use]
extern crate lazy_static;

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
    /// Generates a new Kakuro with the given width and height. The fill
    /// indicates what percentage of the cells should be empty vs. walls. For
    /// example, a fill of 0.1 indicates that 10% of cells should be empty.
    Generate {
        width: usize,
        height: usize,
        fill: f64,

        #[structopt(parse(from_os_str))]
        out: PathBuf,
    },
    /// Imports a JSON Kakuro from kakuros.com, which you can get by looking at
    /// the source code. Compared to Kakuros generated using this tool, they are
    /// guaranteed to have a unique solution.
    Import {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    /// Solves a Kakuro with the given solver.
    Solve {
        solver: String,

        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    /// Benchmarks the given solver on a bunch of example Kakuros, or only the
    /// one given. Measures the runtime several times and prints information
    /// about the median and standard deviation.
    Bench {
        solver: String,

        #[structopt(parse(from_os_str))]
        file: Option<PathBuf>,

        #[structopt(long)]
        warm_up: bool,

        #[structopt(long)]
        num_runs: Option<usize>,
    },
    /// Converts a Kakuro to an SVG.
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
        KakuroOptions::Bench {
            solver,
            file,
            warm_up,
            num_runs,
        } => benchmark(solver, file, warm_up, num_runs.unwrap_or(10)),
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
    // println!("Input board abstracted to this:");
    // println!("{}", input);
    // println!();

    println!("Solving Kakuro.");
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
        "sum_reachable_no_set" => solvers::sum_reachable_no_set::solve(&input),
        "only_check_changes" => solvers::only_check_changes::solve(&input),
        "divide" => solvers::divide::solve(&input),
        "connecting_cells" => solvers::connecting_cells::solve(&input),
        "lazy" => solvers::lazy::solve(&input),
        "propagate_constraints" => solvers::propagate_constraints::solve(&input),
        "solution_in_rc" => solvers::solution_in_rc::solve(&input),
        "simpler_recursion_anchor" => solvers::simpler_recursion_anchor::solve(&input),
        "fxhashmap" => solvers::fxhashmap::solve(&input),
        "better_vecs" => solvers::better_vecs::solve(&input),
        "earlier_anchor" => solvers::earlier_anchor::solve(&input),
        "iterative" => solvers::iterative::solve(&input),
        "array_vec" => solvers::array_vec::solve(&input),
        "sum_table" => solvers::sum_table::solve(&input),
        _ => panic!("Unknown solver {}.", solver),
    }
}

fn benchmark(solver: String, file: Option<PathBuf>, warm_up: bool, num_runs: usize) {
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

    let inputs = if let Some(file) = file {
        vec![file]
    } else {
        BENCHMARK_SUITE
            .iter()
            .map(|path| PathBuf::from(path))
            .collect_vec()
    }
    .into_iter()
    .map(|file| (format!("{}", file.display()), read_kakuro(&file).to_input()))
    .collect_vec();

    // Warm up for 10 seconds.
    if warm_up {
        println!("Warming up.");
        let warmup_start = chrono::Utc::now();
        while chrono::Utc::now() < warmup_start + chrono::Duration::seconds(10) {
            let input = &inputs[0];
            raw_solve(&solver, &input.1);
        }
        println!();
    }

    let mut results = vec![];

    for (input_string, input) in &inputs {
        println!("Input {}.", input_string);
        let mut durations = vec![];
        for i in 0..num_runs {
            print!(
                "Solving run {}/{} started at {}.",
                i,
                num_runs,
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
        let mean = mean(&durations).unwrap();
        let deviation = std_deviation(&durations).unwrap() / mean;
        let min = *durations.iter().min().unwrap() as f32;
        let max = *durations.iter().max().unwrap() as f32;
        println!(
            "{} +- {:.2} %; {} – {}",
            format_duration(mean),
            deviation * 100.0,
            format_duration(min),
            format_duration(max),
        );
        println!();

        results.push((input_string, mean, deviation, min, max));
    }

    println!("Summary:");
    for (input, mean, deviation, min, max) in results {
        println!(
            "- {}: {} +- {:.2} %; {} - {}",
            PathBuf::from(input)
                .file_prefix()
                .unwrap()
                .to_str()
                .unwrap(),
            format_duration(mean),
            deviation * 100.0,
            format_duration(min),
            format_duration(max)
        );
    }
}
fn format_duration(mut value: f32) -> String {
    let units = ["ns", "us", "ms", "s"];
    let mut magnitude = 0;
    while value > 1000.0 && magnitude < units.len() - 1 {
        magnitude += 1;
        value /= 1000.0;
    }
    format!("{:.2} {}", value, units[magnitude])
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
    board
}
