use a_puzzle_a_day::board::*;
use a_puzzle_a_day::search;
use a_puzzle_a_day::browse;
use a_puzzle_a_day::cli::*;

use std::process;
use std::path::PathBuf;

use rand::seq::SliceRandom;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use dialoguer::Confirm;

fn generate(file: PathBuf) -> Vec<Board> {
    let progress = MultiProgress::new();
    
    use Piece::*;
    let pieces = vec![O, Z, V, U, Y, N, P, L];
    let boards = match search::generate_solutions(Board::new(), pieces, Some(&progress)) {
        Ok(rx) => rx,
        Err(_) => {
            eprintln!("error encountered while generating solutions");
            process::exit(1);
        }
    };

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

    match search::create_database(solutions.clone(), file) {
        Ok(_) => {
            println!("\x1b[33m⣿ Finished\x1b[0m");
            solutions
        }
        Err(_) => {
            eprintln!("error encountered while writing solutions to file");
            process::exit(1);
        }
    }
}

fn get_solutions(file: PathBuf) -> DateMap<Vec<Board>> {
    search::classify_solutions(if file.as_path().exists() {
        match search::read_database(file) {
            Ok(boards) => boards,
            Err(_) => {
                eprintln!("error encountered when reading solutions from file");
                process::exit(1);
            }
        }
    } else {
        let confirm = Confirm::new()
            .with_prompt(format!("File `{}` not found. Generate solutions?", file.as_path().display()))
            .interact();
        match confirm {
            Ok(true) => generate(file),
            _ => { process::exit(1); }
        }
    })
}

fn main() {
    let config = Config::parse();

    match config.mode {
        Mode::Generate => { generate(config.file); }
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
