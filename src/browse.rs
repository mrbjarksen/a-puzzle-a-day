pub mod solution_pane;
pub mod date_pane;

use self::solution_pane::SolutionPaneState;
use self::date_pane::DatePaneState;

use crate::board::Board;
use crate::board::square::{Date, DateMap};

use std::cmp::max;
use std::io;
use std::time::Duration;

use rand::Rng;

use crossterm::ExecutableCommand;
use crossterm::terminal;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Size, Flex, Position};
use ratatui::{prelude::*, widgets::*};

enum Behavior {
    Quit,
    Continue,
    Redraw,
}

#[derive(Debug)]
pub enum Pane {
    Date,
    Solution,
}

#[derive(Debug)]
pub struct State {
    solutions: DateMap<Vec<Board>>,
    solution_count: DateMap<usize>,
    selected_solutions: DateMap<usize>,
    focused_pane: Pane,
    date_pane: DatePaneState,
    solution_pane: SolutionPaneState,
}

impl State {
    fn init(boards: DateMap<Vec<Board>>, date: Date) -> Self {
        let solution_count = boards.iter()
            .map(|(&date, sols)| (date, sols.len()))
            .collect();

        let mut rng = rand::thread_rng();
        let selected_solutions = boards.iter()
            .map(|(&date, sols)| (date, rng.gen_range(0..max(1, sols.len()))))
            .collect();

        State {
            solutions: boards,
            solution_count,
            selected_solutions,
            focused_pane: Pane::Solution,
            date_pane: DatePaneState::new(date),
            solution_pane: SolutionPaneState::default(),
        }
    }
}

pub const BIG: Size = Size { width: 33, height: 17 };
pub const SMALL: Size = Size { width: 15, height: 8 };
pub const PADDING: u16 = 3;

fn draw(state: &mut State, frame: &mut Frame) {
    state.solution_pane.num_cols = {
        let max_solution_pane_width = frame.size().width - 4*(PADDING + 1) - BIG.width;
        (max_solution_pane_width + 1) / (SMALL.width + 1)
    };

    if state.solution_pane.num_cols == 0 || frame.size().height < BIG.height {
        frame.render_widget(Paragraph::new("Terminal size is too small"), frame.size());
        return;
    }

    let panes = Layout::horizontal([
        BIG.width + 2*(PADDING + 1),
        state.solution_pane.num_cols * (SMALL.width + 1) + 2*(PADDING + 1)
    ])
        .flex(Flex::Center)
        .split(frame.size());

    state.date_pane.area = panes[0];
    state.solution_pane.area = panes[1];

    date_pane::draw(state, frame);
    solution_pane::draw(state, frame);
}

fn update(state: &mut State) -> io::Result<Behavior> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    return Ok(Behavior::Quit)
                }

                match state.focused_pane {
                    Pane::Date     => { date_pane::update(state, &Event::Key(key)); }
                    Pane::Solution => { solution_pane::update(state, &Event::Key(key)); }
                }
            }
            Event::Mouse(mouse) => {
                let position = Position::new(mouse.column, mouse.row);
                if state.date_pane.area.contains(position) {
                    date_pane::update(state, &Event::Mouse(mouse));
                }
                else if state.solution_pane.area.contains(position) {
                    solution_pane::update(state, &Event::Mouse(mouse));
                }
            }
            Event::Resize(_, _) => {
                return Ok(Behavior::Redraw);
            }
            _ => {}
        }
    }
    Ok(Behavior::Continue)
}

fn run(boards: DateMap<Vec<Board>>, date: Date) -> io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut state = State::init(boards, date);

    terminal.draw(|frame| draw(&mut state, frame))?;
    date_pane::center_selection(&mut state);
    solution_pane::center_selection(&mut state);

    loop {
        terminal.draw(|frame| draw(&mut state, frame))?;
        match update(&mut state)? {
            Behavior::Quit => { break; }
            Behavior::Continue => {}
            Behavior::Redraw => {
                terminal.draw(|frame| draw(&mut state, frame))?;
                date_pane::center_selection(&mut state);
                solution_pane::center_selection(&mut state);
            }
        };
    }

    Ok(())
}

fn startup() -> io::Result<()> {
    io::stdout()
        .execute(terminal::EnterAlternateScreen)?
        .execute(event::EnableMouseCapture)?;
    terminal::enable_raw_mode()
}

fn shutdown() -> io::Result<()> {
    io::stdout()
        .execute(terminal::LeaveAlternateScreen)?
        .execute(event::DisableMouseCapture)?;
    terminal::disable_raw_mode()
}

pub fn browse(boards: DateMap<Vec<Board>>, date: Date) -> io::Result<()> {
    startup()?;
    let result = run(boards, date);
    shutdown()?;
    result
}
