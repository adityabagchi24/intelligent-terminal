use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::agent_registry;
use crate::app::{App, ConnectionState};
use crate::theme;

pub const HEIGHT: u16 = 1; // color-only separation from body; no explicit separator row

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let row = Rect::new(area.x, area.y, area.width, 1);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(row);

    // ── Left: [●] AgentName [model] [∨] ─────────────────────────────────────
    let display_name = if app.agent_name.is_empty() {
        "Agent".to_string()
    } else {
        agent_registry::lookup_profile_by_id(&app.agent_name)
            .display_name
            .to_string()
    };

    let (dot, dot_style) = match &app.state {
        ConnectionState::Connected => ("●", theme::STATUS_CONNECTED),
        ConnectionState::Connecting(_) => ("●", theme::STATUS_CONNECTING),
        ConnectionState::Failed(_) => ("●", theme::STATUS_FAILED),
        ConnectionState::Disconnected => ("●", theme::STATUS_DISCONNECTED),
    };

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(dot, dot_style),
        Span::raw(" "),
        Span::styled(display_name, Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
    ];
    if let Some(model) = &app.agent_model {
        if !model.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(model.clone(), theme::DIM));
        }
    }
    spans.push(Span::styled(" ∨", theme::DIM));

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(theme::PANEL_STYLE),
        chunks[0],
    );

    // ── Right: history button ────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new(" ⊙  ").style(theme::PANEL_STYLE),
        chunks[1],
    );
}
