use crate::board::{Board, Piece, Path, Square, DateMap, Rotation, PlacementError};
use crate::board::compact::CompactBoard;

use std::thread;
use std::sync::mpsc::{self, Sender, Receiver, SendError};
use std::sync::Arc;

use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Write};

use console::style;
use indicatif::{ProgressBar, MultiProgress, ProgressStyle, ProgressIterator, ProgressFinish};

pub static SOLUTIONS: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/solutions.apad"));

pub fn generate(starting_board: Board, pieces: Vec<Piece>, progress: Option<&MultiProgress>) -> Result<Receiver<Board>, SendError<Board>> {
    let (tx0, rx0) = mpsc::channel();
    tx0.send(starting_board)?;
    drop(tx0);

    let mut last_rx = rx0;
    let mut last_piece = None;

    for piece in pieces {
        let mut paths = Vec::<Path>::new();

        let not_mirrored = Path::from(piece);
        for rotation in Rotation::all_by_symmetry(piece.rotational_symmetry()) {
            paths.push(not_mirrored.rotate(rotation));
        }
        if !piece.mirror_symmetric() {
            let mirrored = Path::from(piece).mirror();
            for rotation in Rotation::all_by_symmetry(piece.rotational_symmetry()) {
                paths.push(mirrored.rotate(rotation))
            }
        }

        let (tx, rx) = mpsc::channel();
        let mut forks = Vec::new();
        for path in paths {
            let tx_clone = tx.clone();
            let (fork_tx, fork_rx): (Sender<Arc<Board>>, Receiver<Arc<Board>>) = mpsc::channel();
            thread::spawn(move || {
                for board in fork_rx.iter() {
                    for start in Square::squares() {
                        if let Some(new_board) = board.place(piece, start, &path) {
                            tx_clone.send(new_board).unwrap();
                        }
                    }
                }
            });
            forks.push(fork_tx);
        }
        let bar = last_piece.and_then(|pc| progress.map(|mp| mp.add(make_spinner(pc))));
        thread::spawn(move || {
            for board in last_rx.iter() {
                let board = Arc::new(board);
                for fork_tx in forks.iter() {
                    fork_tx.send(Arc::clone(&board)).unwrap();
                }
                if let Some(b) = bar.as_ref() { b.inc(1) }
            }
            if let Some(b) = bar { finish_spinner(b) }
        });
        last_rx = rx;
        last_piece = Some(piece);
    }

    let (txn, rxn) = mpsc::channel();
    let bar = progress.map(|mp| mp.add(make_spinner(last_piece.unwrap())));
    thread::spawn(move || {
        for board in last_rx.iter() {
            txn.send(board).unwrap();
            if let Some(b) = bar.as_ref() { b.inc(1) }
        }
        if let Some(b) = bar { finish_spinner(b) }
    });

    Ok(rxn)
}

fn make_spinner(piece: Piece) -> ProgressBar {
    let bar = ProgressBar::new_spinner().with_message(format!("{piece}"));
    bar.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg} :{human_pos:>10.bold.dim} found ({elapsed:.dim})")
            .unwrap().tick_chars("⠇⡆⣄⣠⢰⠸⠙⠋⣿")
    );
    bar
}

fn finish_spinner(bar: ProgressBar) {
    bar.finish();
    bar.set_style(bar.style()
        .template("{spinner:.green} {msg} :{human_pos:>10.bold.green} found ({elapsed:.dim})")
        .unwrap()
    );
    bar.tick();
}

pub enum DataError {
    BoardError,
    IoError(io::ErrorKind),
}

impl From<PlacementError> for DataError {
    fn from(_error: PlacementError) -> Self {
        DataError::BoardError
    }
}

impl From<io::Error> for DataError {
    fn from(error: io::Error) -> Self {
        DataError::IoError(error.kind())
    }
}

pub fn write_boards(boards: Vec<Board>, file: PathBuf) -> Result<(), DataError> {
    let mut file_handle = File::create(file.clone())?;

    println!(
        "{} Writing to file {}",
        style("⣿").blue(),
        style(format!("`{}`", file.as_path().display())).blue().bold()
    );
    
    let style = ProgressStyle::with_template(format!(
        "{} {}{}{}", "{spinner:.blue}", style("[").blue(), "{bar:30.blue}", style("]").blue()
    ).as_str()).unwrap()
        .tick_chars("⠇⡆⣄⣠⢰⠸⠙⠋⣿")
        .progress_chars("⠶⠶⠆ ");

    for board in boards.iter().progress_with_style(style).with_finish(ProgressFinish::Abandon) {
        let bytes = CompactBoard::try_from(board.to_owned())?.to_bytes();
        file_handle.write_all(&bytes)?;
    }

    Ok(())
}

pub fn read_boards(bytes: &[u8]) -> Result<Vec<Board>, DataError> {
    let mut boards = Vec::new();

    for chunk in bytes.chunks_exact(9) {
        let chunk: [u8; 9] = chunk.try_into().expect("chunk size should be exactly 9");
        let compact = CompactBoard::from(chunk);
        boards.push(Board::try_from(compact)?);
    }

    Ok(boards)
}

pub fn classify(boards: Vec<Board>) -> DateMap<Vec<Board>> {
    let mut solutions = DateMap::<Vec<Board>>::new();
    for board in boards {
        if let Some(date) = board.solved_for() {
            solutions.entry(date).or_default().push(board);
        }
    }
    solutions
}
