use super::{solution_pane, Pane, State, BIG};

use crate::board::Board;
use crate::board::square::Date;

use std::cmp::max;
use std::collections::HashMap;

use crossterm::event::{Event, KeyEventKind, KeyCode, MouseEventKind, MouseButton};

use ratatui::terminal::Frame;
use ratatui::layout::{Rect, Position, Offset};
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};

#[derive(Debug)]
pub struct DatePane {
    pub selected: Date,
    pub top_date: Date,
    pub scroll: i32,
    pub area: Rect,
    pub buttons: HashMap<Rect, Date>,
}

impl DatePane {
    pub fn new(date: Date) -> Self {
        Self {
            selected: date,
            top_date: date,
            scroll: 0,
            area: Rect::default(),
            buttons: HashMap::new(),
        }
    }
}

pub fn draw(state: &mut State, frame: &mut Frame) {
    state.date_pane.buttons.clear();

    let origin = Position::from(state.date_pane.area);
    let mut date = state.date_pane.top_date;
    for i in 0.. {
        let offset = Offset { x: 0, y: (BIG.height as i32 + 1) * i - state.date_pane.scroll };

        let mut rect = Rect::from((origin, BIG))
            .offset(offset)
            .intersection(state.date_pane.area);

        if offset.y < 0 {
            rect = Rect {
                height: max(0, rect.height as i32 + offset.y + 1) as u16,
                ..rect
            };
        }

        if rect.is_empty() && i > 0 {
            break;
        }

        let index = *state.selected_solutions.get(&date).unwrap_or(&usize::MAX);
        let (empty_vec, empty_board) = (Vec::default(), Board::default());
        let board = state.solutions
            .get(&date)
            .unwrap_or(&empty_vec)
            .get(index)
            .unwrap_or(&empty_board);

        let thumbnail = Paragraph::new(board.to_string())
            .style(Style::default().fg(match state.date_pane.selected {
                selected if date == selected => Color::White,
                _ => Color::DarkGray,
            }))
            .scroll(if date == state.date_pane.top_date { (BIG.height - rect.height, 0) } else { (0, 0) });

        frame.render_widget(thumbnail, rect);

        state.date_pane.buttons.insert(rect, date);

        date = date.next();
    }
}

pub fn update(state: &mut State, event: &Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            match key.code {
                KeyCode::Tab | KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                    state.focused_pane = Pane::Solution;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.date_pane.selected = state.date_pane.selected.prev();
                    solution_pane::center_selection(state);
                    scroll_to_selection(state);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.date_pane.selected = state.date_pane.selected.next();
                    solution_pane::center_selection(state);
                    scroll_to_selection(state);
                }
                _ => {}
            }
        }
        Event::Mouse(click) if click.kind == MouseEventKind::Down(MouseButton::Left) => {
            state.focused_pane = Pane::Date;
            let position = Position::new(click.column, click.row);
            let button = state.date_pane.buttons.iter().find(|&(rect, _)| rect.contains(position));
            if let Some((_, date)) = button {
                state.date_pane.selected = *date;
                solution_pane::center_selection(state);
                scroll_to_selection(state);
            }
        }
        Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollDown => {
            state.date_pane.scroll += 3;
            fix_scroll(state);
        }
        Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollUp => {
            state.date_pane.scroll -= 3;
            fix_scroll(state);
        }
        _ => {}
    }
}

pub fn center_selection(state: &mut State) {
    state.date_pane.top_date = state.date_pane.selected;
    let center_y = state.date_pane.area.top() + state.date_pane.area.height / 2 - (BIG.height + 1) / 2;
    state.date_pane.scroll = -(center_y as i32);
    fix_scroll(state);
}

pub fn scroll_to_selection(state: &mut State) {
    if state.date_pane.area.height < 3 * (BIG.height + 1) {
        center_selection(state);
        return;
    }

    let bottom = state.date_pane.area.bottom() as i32;
    let mut dates_on_screen = std::iter::successors(
        Some((state.date_pane.top_date, -state.date_pane.scroll)),
        |(date, y)| {
            let next_y = y + (BIG.height + 1) as i32;
            (next_y < bottom).then_some((date.next(), next_y))
        }
    );

    match dates_on_screen.find(|&(date, _)| date == state.date_pane.selected) {
        None => { center_selection(state); }
        Some((_, selected_y)) => {
            let top_y = state.date_pane.area.top() as i32 + (BIG.height + 1) as i32 / 4;
            let bottom_y = bottom - ((BIG.height + 1) + (BIG.height + 1) / 4) as i32;
            if top_y > selected_y {
                state.date_pane.scroll -= top_y - selected_y;
                fix_scroll(state);
            } else if selected_y > bottom_y {
                state.date_pane.scroll += selected_y - bottom_y;
                fix_scroll(state)
            }
        }
    }
}

fn fix_scroll(state: &mut State) {
    while state.date_pane.scroll < 0 {
        state.date_pane.scroll += (BIG.height + 1) as i32;
        state.date_pane.top_date = state.date_pane.top_date.prev();
    }

    while state.date_pane.scroll >= (BIG.height + 1) as i32 {
        state.date_pane.scroll -= (BIG.height + 1) as i32;
        state.date_pane.top_date = state.date_pane.top_date.next();
    }
}
