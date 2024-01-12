use crate::board::Board;
use crate::board::square::{Date, DateMap};

use std::cmp::max;
use std::io;
use std::time::Duration;
use std::collections::HashMap;

use rand::Rng;

use crossterm::ExecutableCommand;
use crossterm::terminal;
use crossterm::event::{self, Event, KeyEventKind, KeyCode, MouseEventKind, MouseButton};
use ratatui::{prelude::*, widgets::*};

enum Behavior {
    Quit,
    Continue,
}

enum Pane {
    Date,
    Solution,
}

struct State {
    solutions: DateMap<Vec<Board>>,
    solution_count: DateMap<usize>,
    selected: DateMap<usize>,
    date: Date,
    pane: Pane,

    num_cols: u16,
    center_date: Date,
    date_scroll: u8,
    solution_offset: u8,
    solution_scroll: u8,
    seperator_col: u16,
    buttons: HashMap<Rect, Button>,
}

enum Button {
    Date(Date),
    Solution(usize),
}

impl State {
    fn init(boards: DateMap<Vec<Board>>, date: Date) -> Self {
        let solution_count = boards.iter()
            .map(|(&date, sols)| (date, sols.len()))
            .collect();

        let mut rng = rand::thread_rng();
        let selected = boards.iter()
            .map(|(&date, sols)| (date, rng.gen_range(0..max(1, sols.len()))))
            .collect();

        State {
            solutions: boards,
            solution_count,
            selected,
            date,
            pane: Pane::Solution,

            num_cols: 1,
            center_date: date,
            date_scroll: 0,
            solution_offset: 0,
            solution_scroll: 0,
            seperator_col: 1,
            buttons: HashMap::new(),
        }
    }
}

fn ui(state: &mut State, frame: &mut Frame<'_, impl Backend>) {
    state.buttons.clear();

    let padding = 3;

    let max_sol_pane_width = frame.size().width - 4*padding - 33 - 2 - 1;
    state.num_cols = max_sol_pane_width / 16;
    if state.num_cols < 1 || frame.size().height < 17 {
        frame.render_widget(Paragraph::new("Terminal size is too small"), frame.size());
        return;
    }
    let remaining = max_sol_pane_width % 16;

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Min((remaining + 1) / 2),
            Constraint::Min(padding),
            Constraint::Min(33),
            Constraint::Min(padding + 1),
            Constraint::Min(padding),
            Constraint::Min(state.num_cols * 16),
            Constraint::Min(padding),
            Constraint::Min(remaining / 2),
        ])
        .split(frame.size());

    let date_border = Block::default()
        .borders(Borders::RIGHT)
        .border_type(BorderType::Thick)
        .border_style(match state.pane {
            Pane::Date     => Style::default().fg(Color::Blue),
            Pane::Solution => Style::default().add_modifier(Modifier::DIM),
        });

    frame.render_widget(date_border.clone(), panes[0]);
    frame.render_widget(date_border, panes[3]);

    let solutions_border = Block::default()
        .borders(Borders::LEFT)
        .border_type(BorderType::Thick)
        .border_style(match state.pane {
            Pane::Date     => Style::default().add_modifier(Modifier::DIM),
            Pane::Solution => Style::default().fg(Color::Blue),
        });

    frame.render_widget(solutions_border.clone(), panes[4]);
    frame.render_widget(solutions_border, panes[7]);
    

    state.buttons.insert(panes[2], Button::Date(state.date));

    let selected = *state.selected.get(&state.date).unwrap_or(&0);
    let thumbnail = Paragraph::new(
        state.solutions.get(&state.date).unwrap_or(&Vec::new())
            .get(selected).map_or(String::from(""), |b| b.to_string())
    );
    // ).style(match state.pane {
    //     Pane::Date     => Style::default().fg(Color::Blue),
    //     Pane::Solution => Style::default(),
    // });

    frame.render_widget(thumbnail, panes[2]);

    // let seperator = Block::default()
    //     .borders(Borders::RIGHT)
    //     .border_type(BorderType::Thick)
    //     .border_style(Style::default().fg(Color::Blue));
    //
    state.seperator_col = panes[3].x;
    //
    // frame.render_widget(seperator, panes[3]);

    let num_rows = {
        let solution_count = *state.solution_count.get(&state.date).unwrap_or(&0);
        let num_cols = state.num_cols as usize;
        let total_num_rows = (solution_count + num_cols - 1) / num_cols;
        total_num_rows - state.solution_offset as usize
    };

    let mut row_constraints = vec![Constraint::Min(8); num_rows];
    if state.solution_scroll > 0 {
        row_constraints[0] = Constraint::Min(8 - state.solution_scroll as u16);
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(panes[5]);

    let num_cols = state.num_cols as usize;
    for (i, &row) in rows.iter().enumerate() {
        let i = i + state.solution_offset as usize;
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(16); num_cols])
            .split(row);

        for (j, &col) in cols.iter().enumerate() {
            let index = i*num_cols + j;
            if let Some(sols) = state.solutions.get(&state.date) {
                if let Some(sol) = sols.get(index) {
                    let style = match state.selected.get(&state.date) {
                        // Some(&n) if n == index => match state.pane {
                        //     Pane::Date     => Style::default(),
                        //     Pane::Solution => Style::default().fg(Color::Blue),
                        // }
                        // _ => match state.pane {
                        //     Pane::Date     => Style::default().add_modifier(Modifier::DIM),
                        //     Pane::Solution => Style::default(),
                        // }
                        Some(&n) if n == index => Style::default(),
                        _ => Style::default().add_modifier(Modifier::DIM),
                    };

                    let mut sol_str = sol.to_mini_string();
                    if i == state.solution_offset as usize && row.height < 8 {
                        for _ in 0..state.solution_scroll {
                            sol_str = sol_str.split_once('\n').unwrap_or(("", "")).1.to_string();
                        }
                    }
                    let solution = Paragraph::new(sol_str)
                        .block(Block::default().padding(Padding::new(1, 0, 0, 0)))
                        .style(style);

                    frame.render_widget(solution, col);

                    state.buttons.insert(col, Button::Solution(index));
                }
            }
        }
    }
}

fn update(state: &mut State) -> io::Result<Behavior> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let num_cols = state.num_cols as usize;
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => { return Ok(Behavior::Quit); }
                    KeyCode::Tab | KeyCode::Enter => match state.pane {
                        Pane::Date     => { state.pane = Pane::Solution; }
                        Pane::Solution => { state.pane = Pane::Date;     }
                    }
                    KeyCode::Left | KeyCode::Char('h') => match state.pane {
                        Pane::Date     => {},
                        Pane::Solution => if let Some(i) = state.selected.get_mut(&state.date) {
                            if *i > 0 { *i -= 1; }
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => match state.pane {
                        Pane::Date     => { state.pane = Pane::Solution; },
                        Pane::Solution => if let Some(i) = state.selected.get_mut(&state.date) {
                            if let Some(&count) = state.solution_count.get(&state.date) {
                                if *i + 1 < count { *i += 1 };
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => match state.pane {
                        Pane::Date     => { state.date = state.date.prev(); }
                        Pane::Solution => if let Some(i) = state.selected.get_mut(&state.date) {
                            if *i >= num_cols { *i -= num_cols; }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => match state.pane {
                        Pane::Date     => { state.date = state.date.next(); }
                        Pane::Solution => if let Some(i) = state.selected.get_mut(&state.date) {
                            if let Some(&count) = state.solution_count.get(&state.date) {
                                if *i + num_cols < count { *i += num_cols }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(click) if click.kind == MouseEventKind::Down(MouseButton::Left) => {
                if click.column <= state.seperator_col {
                    state.pane = Pane::Date;
                } else {
                    state.pane = Pane::Solution;
                }

                for (rect, button) in state.buttons.iter() {
                    let in_column_range = rect.x <= click.column && click.column <= rect.x + rect.width;
                    let in_row_range = rect.y <= click.row && click.row <= rect.y + rect.height;
                    if in_column_range && in_row_range {
                        match button {
                            Button::Date(date)      => { state.date = *date; }
                            Button::Solution(index) => { state.selected.insert(state.date, *index); }
                        }
                        break;
                    }
                }
            }
            Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollDown => {
                if scroll.column < state.seperator_col {
                    state.date_scroll += 3;
                    if state.date_scroll >= 19 {
                        state.center_date = state.center_date.next();
                        state.date_scroll -= 19;
                    }
                } else if scroll.column > state.seperator_col {
                    state.solution_scroll += 3;
                    if state.solution_scroll >= 8 {
                        let solution_count = *state.solution_count.get(&state.date).unwrap_or(&0);
                        if solution_count < (state.solution_offset as usize + 1) * state.num_cols as usize {
                            state.solution_scroll = 7
                        } else {
                            state.solution_offset += 1;
                            state.solution_scroll -= 8;
                        }
                    }
                }
            }
            Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollUp => {
                if scroll.column < state.seperator_col {
                    if state.date_scroll < 3 {
                        state.center_date = state.center_date.prev();
                        state.date_scroll += 16;
                    } else {
                        state.date_scroll -= 3;
                    }
                } else if scroll.column > state.seperator_col {
                    if state.solution_scroll < 3 {
                        if state.solution_offset == 0 {
                            state.solution_scroll = 0;
                        } else {
                            state.solution_offset -= 1;
                            state.solution_scroll += 5;
                        }
                    } else {
                        state.solution_scroll -= 3;
                    }
                }
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

    loop {
        terminal.draw(|frame| ui(&mut state, frame))?;
        match update(&mut state)? {
            Behavior::Quit => { break; }
            Behavior::Continue => {}
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
