use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use orchestra_core::models::track::ArtistNode;

/// Renders the Artists pane into `area`.
///
/// - `artists`: the filtered set of artists to display (name + original index pairs)
/// - `selected`: index into `artists` slice
/// - `focused`: whether this pane has keyboard focus
/// - `filter_mode`: whether the user is actively typing a filter query
/// - `filter_text`: the current filter string (may be empty)
pub fn render(
    f: &mut Frame,
    area: Rect,
    artists: &[(usize, &ArtistNode)],
    selected: usize,
    focused: bool,
    filter_mode: bool,
    filter_text: &str,
) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    // Build block title
    let title = if filter_mode {
        "Artists".to_string()
    } else if !filter_text.is_empty() {
        format!("Artists [filter: {}]", filter_text)
    } else {
        "Artists".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if filter_mode {
        // Split the block interior: one line for the filter input, rest for the list
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);

        // Render filter input line: "/ filter_text█"
        let input_line = Line::styled(
            format!("/ {}\u{2588}", filter_text),
            Style::default().fg(Color::Yellow),
        );
        f.render_widget(Paragraph::new(input_line), chunks[0]);

        // Render list in remaining space
        render_list(f, chunks[1], artists, selected);
    } else {
        let list = build_list(artists, selected)
            .block(block)
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
}

fn build_list<'a>(artists: &'a [(usize, &'a ArtistNode)], _selected: usize) -> List<'a> {
    let items: Vec<ListItem> = artists
        .iter()
        .map(|(_, a)| ListItem::new(a.name.as_str()))
        .collect();
    List::new(items)
}

fn render_list(f: &mut Frame, area: Rect, artists: &[(usize, &ArtistNode)], selected: usize) {
    let items: Vec<ListItem> = artists
        .iter()
        .map(|(_, a)| ListItem::new(a.name.as_str()))
        .collect();

    let list = List::new(items)
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
