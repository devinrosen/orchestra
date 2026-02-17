use crossterm::event::KeyCode;
use orchestra_core::models::track::{AlbumNode, ArtistNode, LibraryTree, Track};

use crate::player::PlayerHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Artists,
    Albums,
    Tracks,
}

impl Pane {
    pub fn next(self) -> Self {
        match self {
            Pane::Artists => Pane::Albums,
            Pane::Albums => Pane::Tracks,
            Pane::Tracks => Pane::Artists,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Pane::Artists => Pane::Tracks,
            Pane::Albums => Pane::Artists,
            Pane::Tracks => Pane::Albums,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub is_paused: bool,
}

pub struct App {
    pub tree: LibraryTree,
    pub selected_artist: usize,
    pub selected_album: usize,
    pub selected_track: usize,
    pub focused_pane: Pane,
    pub player: PlayerHandle,
    pub now_playing: Option<NowPlaying>,
    pub volume: f32,
    pub status_msg: Option<String>,
    pub should_quit: bool,
    pub filter_mode: bool,
    pub filter_text: String,
}

impl App {
    pub fn new(tree: LibraryTree) -> Self {
        App {
            tree,
            selected_artist: 0,
            selected_album: 0,
            selected_track: 0,
            focused_pane: Pane::Artists,
            player: PlayerHandle::spawn(),
            now_playing: None,
            volume: 0.7,
            status_msg: None,
            should_quit: false,
            filter_mode: false,
            filter_text: String::new(),
        }
    }

    /// Drain audio thread errors into `status_msg`. Call once per frame.
    pub fn tick(&mut self) {
        while let Ok(err) = self.player.errors.try_recv() {
            self.status_msg = Some(err);
        }
    }

    /// Returns artists matching the current filter, paired with their original index.
    /// When filter_text is empty, returns all artists with their original indices.
    pub fn filtered_artists(&self) -> Vec<(usize, &ArtistNode)> {
        if self.filter_text.is_empty() {
            self.tree
                .artists
                .iter()
                .enumerate()
                .collect()
        } else {
            let needle = self.filter_text.to_lowercase();
            self.tree
                .artists
                .iter()
                .enumerate()
                .filter(|(_, a)| a.name.to_lowercase().contains(&needle))
                .collect()
        }
    }

    /// Maps `selected_artist` (index into filtered list) back to the real tree index.
    /// Returns 0 if the filtered list is empty.
    pub fn real_artist_index(&self) -> usize {
        let filtered = self.filtered_artists();
        filtered
            .get(self.selected_artist)
            .map(|(real_idx, _)| *real_idx)
            .unwrap_or(0)
    }

    /// Returns the albums for the currently selected artist (if any).
    pub fn current_albums(&self) -> &[AlbumNode] {
        let real_idx = self.real_artist_index();
        self.tree
            .artists
            .get(real_idx)
            .map(|a| a.albums.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the tracks for the currently selected album (if any).
    pub fn current_tracks(&self) -> &[Track] {
        self.current_albums()
            .get(self.selected_album)
            .map(|a| a.tracks.as_slice())
            .unwrap_or(&[])
    }

    fn play_selected(&mut self) {
        if let Some(track) = self.current_tracks().get(self.selected_track) {
            let title = track
                .title
                .clone()
                .unwrap_or_else(|| "Unknown Title".to_string());
            let artist = track
                .album_artist
                .clone()
                .or_else(|| track.artist.clone())
                .unwrap_or_else(|| "Unknown Artist".to_string());

            self.player.play(track.file_path.clone());
            self.player.set_volume(self.volume);
            self.now_playing = Some(NowPlaying {
                title,
                artist,
                is_paused: false,
            });
            self.status_msg = None;
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        // Filter mode captures most keys
        if self.filter_mode {
            match key {
                KeyCode::Esc => {
                    // Exit filter mode, clear filter
                    self.filter_mode = false;
                    self.filter_text.clear();
                    self.selected_artist = 0;
                    self.selected_album = 0;
                    self.selected_track = 0;
                }
                KeyCode::Enter => {
                    // Lock the filter and exit filter mode
                    self.filter_mode = false;
                    self.selected_artist = 0;
                    self.selected_album = 0;
                    self.selected_track = 0;
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                    self.selected_artist = 0;
                    self.selected_album = 0;
                    self.selected_track = 0;
                }
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                    self.selected_artist = 0;
                    self.selected_album = 0;
                    self.selected_track = 0;
                }
                // All other keys (j/k/Tab/etc.) are ignored while in filter mode
                _ => {}
            }
            return;
        }

        // Not in filter mode
        match key {
            // Quit
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            // Enter filter mode
            KeyCode::Char('/') => {
                self.filter_mode = true;
                self.filter_text.clear();
                self.selected_artist = 0;
                self.selected_album = 0;
                self.selected_track = 0;
            }

            // Esc when filter is active (but not in filter mode) → clear filter
            KeyCode::Esc => {
                if !self.filter_text.is_empty() {
                    self.filter_text.clear();
                    self.selected_artist = 0;
                    self.selected_album = 0;
                    self.selected_track = 0;
                }
            }

            // Navigation: down
            KeyCode::Char('j') | KeyCode::Down => match self.focused_pane {
                Pane::Artists => {
                    let max = self.filtered_artists().len().saturating_sub(1);
                    if self.selected_artist < max {
                        self.selected_artist += 1;
                        self.selected_album = 0;
                        self.selected_track = 0;
                    }
                }
                Pane::Albums => {
                    let max = self.current_albums().len().saturating_sub(1);
                    if self.selected_album < max {
                        self.selected_album += 1;
                        self.selected_track = 0;
                    }
                }
                Pane::Tracks => {
                    let max = self.current_tracks().len().saturating_sub(1);
                    if self.selected_track < max {
                        self.selected_track += 1;
                    }
                }
            },

            // Navigation: up
            KeyCode::Char('k') | KeyCode::Up => match self.focused_pane {
                Pane::Artists => {
                    if self.selected_artist > 0 {
                        self.selected_artist -= 1;
                        self.selected_album = 0;
                        self.selected_track = 0;
                    }
                }
                Pane::Albums => {
                    if self.selected_album > 0 {
                        self.selected_album -= 1;
                        self.selected_track = 0;
                    }
                }
                Pane::Tracks => {
                    if self.selected_track > 0 {
                        self.selected_track -= 1;
                    }
                }
            },

            // Tab: advance pane
            KeyCode::Tab => {
                self.focused_pane = self.focused_pane.next();
                match self.focused_pane {
                    Pane::Albums => self.selected_album = 0,
                    Pane::Tracks => self.selected_track = 0,
                    Pane::Artists => {}
                }
            }

            // Shift+Tab: reverse pane
            KeyCode::BackTab => {
                self.focused_pane = self.focused_pane.prev();
                match self.focused_pane {
                    Pane::Albums => self.selected_album = 0,
                    Pane::Tracks => self.selected_track = 0,
                    Pane::Artists => {}
                }
            }

            // Enter: play if in Tracks pane, else advance pane
            KeyCode::Enter => {
                if self.focused_pane == Pane::Tracks {
                    self.play_selected();
                } else {
                    self.focused_pane = self.focused_pane.next();
                    match self.focused_pane {
                        Pane::Albums => self.selected_album = 0,
                        Pane::Tracks => self.selected_track = 0,
                        Pane::Artists => {}
                    }
                }
            }

            // Space: toggle play/pause
            KeyCode::Char(' ') => {
                if let Some(ref mut np) = self.now_playing {
                    if np.is_paused {
                        self.player.resume();
                        np.is_paused = false;
                    } else {
                        self.player.pause();
                        np.is_paused = true;
                    }
                }
            }

            // n: next track
            KeyCode::Char('n') => {
                let max = self.current_tracks().len().saturating_sub(1);
                if self.selected_track < max {
                    self.selected_track += 1;
                    self.play_selected();
                }
            }

            // p: previous track
            KeyCode::Char('p') => {
                if self.selected_track > 0 {
                    self.selected_track -= 1;
                    self.play_selected();
                }
            }

            // Volume up
            KeyCode::Char('+') => {
                self.volume = (self.volume + 0.05).min(1.0);
                self.player.set_volume(self.volume);
            }

            // Volume down
            KeyCode::Char('-') => {
                self.volume = (self.volume - 0.05).max(0.0);
                self.player.set_volume(self.volume);
            }

            _ => {}
        }
    }

}
