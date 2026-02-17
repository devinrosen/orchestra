use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use orchestra_core::models::track::ArtistNode;

/// Renders the Artists pane into `area`.
pub fn render(f: &mut Frame, area: Rect, artists: &[ArtistNode], selected: usize, focused: bool) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = artists
        .iter()
        .map(|a| ListItem::new(a.name.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Artists")
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
    if !artists.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(list, area, &mut state);
}
