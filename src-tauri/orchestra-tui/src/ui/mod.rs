use ratatui::Frame;

use crate::app::{App, Pane};

pub mod albums;
pub mod artists;
pub mod layout;
pub mod now_playing;
pub mod tracks;

/// Main draw function — called every frame.
pub fn draw(f: &mut Frame, app: &App) {
    let (top, bottom) = layout::split_frame(f);
    let (artist_area, album_area, track_area) = layout::split_top(top);

    let filtered = app.filtered_artists();

    artists::render(
        f,
        artist_area,
        &filtered,
        app.selected_artist,
        app.focused_pane == Pane::Artists,
        app.filter_mode,
        &app.filter_text,
    );

    albums::render(
        f,
        album_area,
        app.current_albums(),
        app.selected_album,
        app.focused_pane == Pane::Albums,
    );

    tracks::render(
        f,
        track_area,
        app.current_tracks(),
        app.selected_track,
        app.focused_pane == Pane::Tracks,
    );

    now_playing::render(
        f,
        bottom,
        &app.now_playing,
        app.volume,
        &app.status_msg,
        app.filter_mode,
    );
}
