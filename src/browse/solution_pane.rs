use super::{State, Pane, SMALL, PADDING};

use std::cmp::max;
use std::collections::HashMap;

use ratatui::layout::{Position, Offset};
use ratatui::{prelude::*, widgets::*};

use crossterm::event::{Event, KeyEventKind, KeyCode, MouseEventKind, MouseButton};

#[derive(Default, Debug)]
pub struct SolutionPaneState {
    pub num_cols: u16,
    pub scroll: i32,
    pub area: Rect,
    pub buttons: HashMap<Rect, usize>,
}

pub fn draw(state: &mut State, frame: &mut Frame) {
    let block = Block::default()
        .padding(Padding::horizontal(PADDING))
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(BorderType::Thick)
        .border_style(match state.focused_pane {
            Pane::Date     => Style::default().add_modifier(Modifier::DIM),
            Pane::Solution => Style::default().fg(Color::Blue),
        });

    frame.render_widget(block.clone(), state.solution_pane.area);

    state.solution_pane.buttons.clear();

    if let Some(boards) = state.solutions.get(&state.date_pane.selected) {
        let origin = Position::from(block.inner(state.solution_pane.area));

        for (i, board) in boards.iter().enumerate() {
            let offset = Offset {
                x: (SMALL.width as i32 + 1) * (i as i32 % state.solution_pane.num_cols as i32),
                y: SMALL.height as i32 * (i as i32 / state.solution_pane.num_cols as i32) - state.solution_pane.scroll,
            };

            let mut rect = Rect::from((origin, SMALL)).offset(offset).intersection(state.solution_pane.area);

            if offset.y < 0 {
                rect = Rect {
                    height: max(rect.height as i32 + offset.y, 0) as u16,
                    ..rect
                };
            }

            if rect.is_empty() {
                continue;
            }

            let thumbnail = Paragraph::new(board.to_mini_string())
                .style(match state.selected_solutions.get(&state.date_pane.selected) {
                    Some(&index) if i == index => Style::default(),
                    _ => Style::default().add_modifier(Modifier::DIM),
                })
                .scroll(if rect.y == origin.y { (SMALL.height - rect.height, 0) } else { (0, 0) });

            frame.render_widget(thumbnail, rect);

            state.solution_pane.buttons.insert(rect, i);
        }
    }
}

pub fn update(state: &mut State, event: &Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            let num_cols = state.solution_pane.num_cols as usize;
            match key.code {
                KeyCode::Tab | KeyCode::Enter => {
                    state.focused_pane = Pane::Date;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if let Some(i) = state.selected_solutions.get_mut(&state.date_pane.selected) {
                        if *i > 0 {
                            *i -= 1;
                            scroll_to_selection(state);
                        }
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if let Some(i) = state.selected_solutions.get_mut(&state.date_pane.selected) {
                        if let Some(&count) = state.solution_count.get(&state.date_pane.selected) {
                            if *i + 1 < count {
                                *i += 1;
                                scroll_to_selection(state);
                            }
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(i) = state.selected_solutions.get_mut(&state.date_pane.selected) {
                        if *i >= num_cols {
                            *i -= num_cols;
                            scroll_to_selection(state);
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(i) = state.selected_solutions.get_mut(&state.date_pane.selected) {
                        if let Some(&count) = state.solution_count.get(&state.date_pane.selected) {
                            if *i + num_cols < count {
                                *i += num_cols;
                                scroll_to_selection(state);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Event::Mouse(click) if click.kind == MouseEventKind::Down(MouseButton::Left) => {
            state.focused_pane = Pane::Solution;
            let position = Position::new(click.column, click.row);
            let button = state.solution_pane.buttons.iter().find(|&(rect, _)| rect.contains(position));
            if let Some((_, index)) = button {
                state.selected_solutions.insert(state.date_pane.selected, *index);
                scroll_to_selection(state);
            }
        }
        Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollDown => {
            state.solution_pane.scroll += 3;
            clamp_scroll(state);
        }
        Event::Mouse(scroll) if scroll.kind == MouseEventKind::ScrollUp => {
            state.solution_pane.scroll -= 3;
            clamp_scroll(state);
        }
        _ => {}
    }
}

pub fn center_selection(state: &mut State) {
    match state.selected_solutions.get(&state.date_pane.selected) {
        None => { state.solution_pane.scroll = 0; }
        Some(&index) => {
            let row_num = index as u16 / state.solution_pane.num_cols;
            let center_y = state.solution_pane.area.top() + state.solution_pane.area.height / 2 - SMALL.height / 2;
            state.solution_pane.scroll = (row_num * SMALL.height) as i32 - center_y as i32;
            clamp_scroll(state);
        }
    }
}

pub fn scroll_to_selection(state: &mut State) {
    match state.selected_solutions.get(&state.date_pane.selected) {
        None => { state.solution_pane.scroll = 0; }
        Some(&index) => {
            let row_num = index as u16 / state.solution_pane.num_cols;
            let selected_y = (row_num * SMALL.height) as i32 - state.solution_pane.scroll;
            let top_y = state.solution_pane.area.top() as i32 + (SMALL.height / 4) as i32;
            let bottom_y = (state.solution_pane.area.bottom() - (SMALL.height + SMALL.height / 4)) as i32;
            if top_y > selected_y {
                state.solution_pane.scroll -= top_y - selected_y;
                clamp_scroll(state);
            } else if selected_y > bottom_y {
                state.solution_pane.scroll += selected_y - bottom_y;
                clamp_scroll(state);
            }
        }
    }
}

fn clamp_scroll(state: &mut State) {
    let num_rows = match state.solution_count.get(&state.date_pane.selected) {
        None | Some(&0) => 0,
        Some(&num_solutions) => 1 + (num_solutions as u16 - 1) / state.solution_pane.num_cols
    };

    let max_scroll = (SMALL.height * num_rows)
        .saturating_sub(state.solution_pane.area.bottom());

    state.solution_pane.scroll = state.solution_pane.scroll
        .clamp(0, max_scroll as i32);
} 
