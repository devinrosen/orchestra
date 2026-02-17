use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use orchestra_core::models::track::Track;

/// Renders the Tracks pane into `area`.
pub fn render(f: &mut Frame, area: Rect, tracks: &[Track], selected: usize, focused: bool) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = tracks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let title = t
                .title
                .as_deref()
                .unwrap_or("Unknown Title");
            let num = t.track_number.map(|n| format!("{n}. ")).unwrap_or_else(|| format!("{}. ", i + 1));
            ListItem::new(format!("{num}{title}"))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Tracks")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !tracks.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(list, area, &mut state);
}
