pub mod solution_pane;
pub mod date_pane;

use self::solution_pane::SolutionPane;
use self::date_pane::DatePane;

use crate::board::Board;
use crate::board::square::{Date, DateMap};

use std::io;
use std::cmp::max;
use std::time::Duration;

use rand::Rng;

use crossterm::ExecutableCommand;
use crossterm::terminal;
use crossterm::event::{self, Event, KeyCode};

use ratatui::backend::CrosstermBackend;
use ratatui::terminal::{Frame, Terminal};
use ratatui::layout::{Layout, Flex, Position, Size};
use ratatui::widgets::{Block, Paragraph};
use ratatui::text::Line;
use ratatui::style::{Style, Color};
use ratatui::symbols::border;

enum Message {
    Quit,
    Continue,
    Redraw,
}

#[derive(Debug, Copy, Clone)]
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
    date_pane: DatePane,
    solution_pane: SolutionPane,
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
            date_pane: DatePane::new(date),
            solution_pane: SolutionPane::default(),
        }
    }
}

pub const BIG: Size = Size { width: 33, height: 17 };
pub const SMALL: Size = Size { width: 15, height: 8 };
pub const PADDING: u16 = 3;

fn draw(state: &mut State, frame: &mut Frame) {
    state.solution_pane.num_cols = {
        let max_solution_pane_width = frame.size().width - 4*(PADDING + 1) - BIG.width;
        max_solution_pane_width / (SMALL.width + 2)
    };

    if state.solution_pane.num_cols == 0 || frame.size().height < BIG.height + 2 {
        frame.render_widget(Paragraph::new("Terminal size is too small"), frame.size());
        return;
    }

    let panes = Layout::horizontal([
        BIG.width + 2*(PADDING + 1),
        state.solution_pane.num_cols * (SMALL.width + 2) + 2*PADDING
    ])
        .flex(Flex::Center)
        .split(frame.size());

    let block = Block::bordered()
        .border_set(border::Set {
            bottom_left: border::QUADRANT_RIGHT_HALF,
            bottom_right: border::QUADRANT_LEFT_HALF,
            horizontal_bottom: border::QUADRANT_BLOCK,
            ..border::QUADRANT_INSIDE
        })
        .title_style(Style::default().fg(Color::Black).bg(Color::Blue));

    // Render date pane

    frame.render_widget(
        match state.focused_pane {
            Pane::Date => block.clone()
                .border_style(Style::default().fg(Color::Blue))
                .title_bottom(Line::from("▎󰌒 : Switch▕▏↑/↓ : Choose▕▏").left_aligned())
                .title_bottom(Line::from("▕▏󱊷 : Quit🮇").right_aligned()),
            Pane::Solution => block.clone()
                .border_style(Style::default().fg(Color::DarkGray)),
        },
        panes[0]
    );

    state.date_pane.area = block.inner(panes[0]);
    date_pane::draw(state, frame);

    // Render solution pane
    
    frame.render_widget(
        match state.focused_pane {
            Pane::Date => block.clone()
                .border_style(Style::default().fg(Color::DarkGray)),
            Pane::Solution => block.clone()
                .border_style(Style::default().fg(Color::Blue))
                .title_bottom(Line::from("▎󰌒 : Switch▕▏↑/↓/←/→ : Choose▕▏").left_aligned())
                .title_bottom(Line::from("▕▏󱊷 : Quit🮇").right_aligned()),
        },
        panes[1]
    );

    state.solution_pane.area = block.inner(panes[1]);
    solution_pane::draw(state, frame);
}

fn update(state: &mut State) -> io::Result<Message> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    return Ok(Message::Quit)
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
                return Ok(Message::Redraw);
            }
            _ => {}
        }
    }
    Ok(Message::Continue)
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
            Message::Quit => { break; }
            Message::Continue => {}
            Message::Redraw => {
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
