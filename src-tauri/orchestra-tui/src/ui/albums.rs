use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use orchestra_core::models::track::AlbumNode;

/// Renders the Albums pane into `area`.
pub fn render(f: &mut Frame, area: Rect, albums: &[AlbumNode], selected: usize, focused: bool) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = albums
        .iter()
        .map(|a| {
            let label = if let Some(year) = a.year {
                format!("{} ({})", a.name, year)
            } else {
                a.name.clone()
            };
            ListItem::new(label)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Albums")
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
    if !albums.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(list, area, &mut state);
}
