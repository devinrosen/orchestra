use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Splits the frame into:
/// - `top`: full-width area for the 3 browser panes
/// - `bottom`: 3-row bar for now-playing info
pub fn split_frame(f: &Frame) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());
    (chunks[0], chunks[1])
}

/// Splits the top area into three equal columns.
pub fn split_top(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}
