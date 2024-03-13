use a_puzzle_a_day::board::*;
use a_puzzle_a_day::solutions;
use a_puzzle_a_day::browse;
use a_puzzle_a_day::cli::*;

use std::process;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use rand::seq::SliceRandom;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use dialoguer::Confirm;

fn error(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1);
}

fn generate(file: PathBuf) -> Vec<Board> {
    let progress = MultiProgress::new();
    
    use Piece::*;
    let pieces = vec![O, Z, V, U, Y, N, P, L];
    let boards =
        solutions::generate(Board::default(), pieces, Some(&progress))
            .unwrap_or_else(|_| error("error encountered while generating solutions"));

    let bar = progress.add(ProgressBar::new_spinner());
    bar.set_style(
        ProgressStyle::with_template("{spinner:.blue} {human_pos:.bold.dim} valid solutions found")
            .unwrap().tick_chars("⠇⡆⣄⣠⢰⠸⠙⠋⣿")
    );

    let mut solutions = boards.iter().filter(|board|
        board.solved_for().map(|_| bar.inc(1)).is_some()
    ).collect::<Vec<_>>();
    solutions.sort_unstable();

    bar.finish();
    bar.set_style(
        bar.style().template("{spinner:.blue} {human_pos:.bold.blue} valid solutions found")
            .unwrap()
    );
    bar.tick();

    progress.set_move_cursor(true);

    match solutions::write_boards(solutions.clone(), file) {
        Ok(_) => {
            println!("\x1b[33m⣿ Finished\x1b[0m");
            solutions
        }
        Err(_) => error("error encountered while writing solutions to file")
    }
}

fn get_solutions(file: Option<PathBuf>) -> DateMap<Vec<Board>> {
    solutions::classify(match file {
        None => {
            solutions::read_boards(solutions::SOLUTIONS)
                .unwrap_or_else(|_| error("error encountered when decoding solutions"))
        }
        Some(file) => {
            if file.as_path().exists() {
                let handle = File::open(file).unwrap_or_else(|err| error(&format!("error: {err})")));
                let bytes: Vec<u8> = handle.bytes().map(|res| res.unwrap_or_else(|err| error(&format!("error: {err}")))).collect();
                solutions::read_boards(&bytes)
                    .unwrap_or_else(|_| error("error encountered when decoding solutions"))
            } else {
                let confirm = Confirm::new()
                    .with_prompt(format!("File `{}` not found. Generate solutions?", file.as_path().display()))
                    .interact();
                match confirm {
                    Ok(true) => generate(file),
                    _ => { process::exit(1); }
                }
            }
        }
    })
}

fn main() {
    let config = Config::parse();

    match config.mode {
        Mode::Generate => {
            let file = config.file.unwrap_or(PathBuf::from("solutions.apad"));
            generate(file);
        }
        Mode::Browse => {
            let solutions = get_solutions(config.file);
            if let Err(e) = browse::browse(solutions, config.date) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Mode::Random => {
            let solutions = get_solutions(config.file);
            match solutions.get(&config.date) {
                Some(sols) => {
                    let mut rng = rand::thread_rng();
                    println!("{}", Vec::from_iter(sols).choose(&mut rng).unwrap());
                },
                None => {
                    eprintln!("No solutions found for date {}", config.date);
                    process::exit(1);
                }
            }
        }
    }
}
