use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::NowPlaying;

/// Renders the now-playing bottom bar into `area`.
pub fn render(
    f: &mut Frame,
    area: Rect,
    now_playing: &Option<NowPlaying>,
    volume: f32,
    status_msg: &Option<String>,
    filter_mode: bool,
) {
    let vol_pct = (volume * 100.0).round() as u32;

    let content: Line = if filter_mode {
        Line::styled(
            "  Type to filter artists, Enter to confirm, Esc to cancel",
            Style::default().fg(Color::Yellow),
        )
    } else if let Some(err) = status_msg {
        Line::styled(format!("  {err}"), Style::default().fg(Color::Red))
    } else if let Some(np) = now_playing {
        let icon = if np.is_paused { "\u{23F8}" } else { "\u{25B6}" };
        Line::raw(format!(
            "  {} {} \u{2014} {}    Vol {}%",
            icon, np.title, np.artist, vol_pct
        ))
    } else {
        Line::styled(
            "  No track playing  (/ = filter artists, Enter = play, Space = pause, n/p = next/prev, +/- = volume, q = quit)",
            Style::default().fg(Color::DarkGray),
        )
    };

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(paragraph, area);
}
