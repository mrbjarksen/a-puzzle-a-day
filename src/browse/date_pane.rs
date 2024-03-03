use super::{solution_pane, Pane, State, BIG, PADDING};

use crate::board::{Board, DateMap};
use crate::board::square::Date;

use std::cmp::max;

use crossterm::event::{Event, KeyEventKind, KeyCode, MouseEventKind, MouseButton};

use ratatui::terminal::Frame;
use ratatui::layout::{Rect, Position, Size, Offset};
use ratatui::widgets::{Paragraph, Block, Padding};
use ratatui::text::Line;
use ratatui::style::{Style, Color, Modifier};

#[derive(Debug)]
pub struct DatePane {
    pub selected: Date,
    pub top_date: Date,
    pub scroll: i32,
    pub area: Rect,
    pub buttons: DateMap<Rect>,
}

impl DatePane {
    pub fn new(date: Date) -> Self {
        Self {
            selected: date,
            top_date: date,
            scroll: 0,
            area: Rect::default(),
            buttons: DateMap::new(),
        }
    }
}

pub fn draw(state: &mut State, frame: &mut Frame) {
    state.date_pane.buttons.clear();

    let color = match state.focused_pane {
        Pane::Date     => Color::Blue,
        Pane::Solution => Color::DarkGray,
    };
    let origin = Position::from(state.date_pane.area);
    let mut date = state.date_pane.top_date;
    for i in 0.. {
        let offset = Offset { x: PADDING as i32 - 1, y: (BIG.height as i32 + 1) * i - state.date_pane.scroll };

        let mut rect = Rect::from((origin, Size { width: BIG.width + 2, ..BIG }))
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
            .style(Style::default().fg(color))
            .block(Block::new().padding(Padding::horizontal(1)))
            .scroll(if date == state.date_pane.top_date { (BIG.height - rect.height, 0) } else { (0, 0) });

        frame.render_widget(thumbnail, rect);

        if let Some(&index) = state.selected_solutions.get(&date) {
            if let Some(&count) = state.solution_count.get(&date) {
                let info = Line::from(format!("#{} / {}", index + 1, count))
                    .style(Style::default().add_modifier(Modifier::ITALIC))
                    .right_aligned();

                let info_rect = state.date_pane.area.intersection(Rect {
                    x: rect.left() + 17,
                    y: if date == state.date_pane.top_date { rect.bottom() - 2 } else { rect.top() + 15 },
                    width: 15,
                    height: 1,
                });
                
                if !info_rect.is_empty() {
                    frame.render_widget(info, info_rect);
                }
            }
        }

        state.date_pane.buttons.insert(date, rect);

        date = date.next();
    }

    if let Some((_, &rect)) = state.date_pane.buttons.iter().find(|&(&date, _)| date == state.date_pane.selected) {
        let block = Block::new().style(Style::default().fg(Color::Black).bg(color));
        frame.render_widget(block, rect);
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
            let button = state.date_pane.buttons.iter().find(|&(_, &rect)| rect.contains(position));
            if let Some((&date, _)) = button {
                state.date_pane.selected = date;
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

    let bottom = (state.date_pane.area.bottom() + (BIG.height + 1)) as i32;
    let mut dates_on_screen = std::iter::successors(
        Some((state.date_pane.top_date.prev(), -state.date_pane.scroll - (BIG.height + 1) as i32)),
        |(date, y)| {
            let next_y = y + (BIG.height + 1) as i32;
            (next_y < bottom).then_some((date.next(), next_y))
        }
    );

    match dates_on_screen.find(|&(date, _)| date == state.date_pane.selected) {
        None => { center_selection(state); }
        Some((_, selected_y)) => {
            let top_y = (state.date_pane.area.top() + PADDING / 2) as i32 - 1;
            let bottom_y = (state.date_pane.area.bottom() - BIG.height - PADDING / 2 - 1) as i32;
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
