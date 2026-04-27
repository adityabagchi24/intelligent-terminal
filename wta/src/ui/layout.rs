use ratatui::prelude::*;
use crate::app::{App, AppMode};

use super::{chat, debug_panel, input, permission, recommendations, setup, title_bar};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if app.mode == AppMode::Setup {
        setup::render(frame, app, area);
        return;
    }

    let (main_area, debug_area) = if app.show_debug_panel {
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);
        (h[0], Some(h[1]))
    } else {
        (area, None)
    };

    let rec_height = if app.recommendations.is_some() {
        Constraint::Length(app.rec_panel_height())
    } else {
        Constraint::Length(0)
    };
    let input_height = input::input_height(&app.input, app.cursor_pos, main_area.width.saturating_sub(2));

    // Outer vertical split: title bar (full width) | content below
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_bar::HEIGHT),
            Constraint::Min(0),
        ])
        .split(main_area);

    title_bar::render(frame, app, v_chunks[0]);

    // Horizontal margin for the content area (restore padding removed from the profile)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1), // left padding
            Constraint::Min(0),
            Constraint::Length(1), // right padding
        ])
        .split(v_chunks[1]);

    let content_area = h_chunks[1];

    // Vertical split for chat | recommendations | input within padded area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            rec_height,
            Constraint::Length(input_height),
        ])
        .split(content_area);

    chat::render(frame, app, chunks[0]);
    recommendations::render(frame, app, chunks[1]);
    input::render(frame, app, chunks[2]);

    if let Some(debug_area) = debug_area {
        debug_panel::render(frame, app, debug_area);
    }

    if app.permission.is_some() {
        permission::render(frame, app, area);
    }
}

pub fn input_cursor_position(app: &App, area: Rect) -> Option<Position> {
    if app.mode == AppMode::Setup {
        return None;
    }

    let main_area = if app.show_debug_panel {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)[0]
    } else {
        area
    };

    let rec_height = if app.recommendations.is_some() {
        Constraint::Length(app.rec_panel_height())
    } else {
        Constraint::Length(0)
    };
    let input_height = input::input_height(&app.input, app.cursor_pos, main_area.width.saturating_sub(2));

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_bar::HEIGHT),
            Constraint::Min(0),
        ])
        .split(main_area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(v_chunks[1]);

    let content_area = h_chunks[1];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            rec_height,
            Constraint::Length(input_height),
        ])
        .split(content_area);

    input::cursor_position(app, chunks[2])
}
