use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use lofty::prelude::*;
use lofty::picture::PictureType;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use tauri::{AppHandle, Emitter};

/// Serializable payload emitted to the frontend as a "media-remote" event.
#[derive(serde::Serialize, Clone)]
#[serde(tag = "type")]
enum RemoteCommandPayload {
    Play,
    Pause,
    Next,
    Previous,
    Seek { position: f64 },
}

pub enum MediaCmd {
    UpdateMetadata {
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        duration_secs: Option<f64>,
        cover_url: Option<String>,
    },
    UpdatePlayback {
        playing: bool,
        position_secs: f64,
    },
}

/// Tauri-managed state for the Now Playing / media session.
///
/// Holds only a channel sender so it is `Send + Sync`. The actual
/// `MediaControls` (which is `!Send` on macOS) lives on a dedicated thread.
pub struct MediaSessionState {
    tx: mpsc::SyncSender<MediaCmd>,
}

impl MediaSessionState {
    /// Initialise the media session. Spawns a dedicated thread that owns
    /// `MediaControls` for the lifetime of the application.
    pub fn init(app_handle: AppHandle) -> Self {
        let (tx, rx) = mpsc::sync_channel::<MediaCmd>(32);

        thread::spawn(move || {
            let config = PlatformConfig {
                dbus_name: "com.orchestra.app",
                display_name: "Orchestra",
                hwnd: None,
            };

            let mut controls = match MediaControls::new(config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[media_session] Failed to create MediaControls: {e:?}");
                    return;
                }
            };

            let ah = app_handle.clone();
            if let Err(e) = controls.attach(move |event: MediaControlEvent| {
                let payload = match event {
                    MediaControlEvent::Play => Some(RemoteCommandPayload::Play),
                    MediaControlEvent::Pause => Some(RemoteCommandPayload::Pause),
                    MediaControlEvent::Toggle => Some(RemoteCommandPayload::Play),
                    MediaControlEvent::Next => Some(RemoteCommandPayload::Next),
                    MediaControlEvent::Previous => Some(RemoteCommandPayload::Previous),
                    MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                        Some(RemoteCommandPayload::Seek { position: pos.as_secs_f64() })
                    }
                    _ => None,
                };
                if let Some(p) = payload {
                    let _ = ah.emit("media-remote", p);
                }
            }) {
                eprintln!("[media_session] Failed to attach event handler: {e:?}");
                return;
            }

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    MediaCmd::UpdateMetadata { title, artist, album, duration_secs, cover_url } => {
                        let duration = duration_secs.map(|d| Duration::from_secs_f64(d));
                        let metadata = MediaMetadata {
                            title: title.as_deref(),
                            artist: artist.as_deref(),
                            album: album.as_deref(),
                            duration,
                            cover_url: cover_url.as_deref(),
                        };
                        if let Err(e) = controls.set_metadata(metadata) {
                            eprintln!("[media_session] set_metadata error: {e:?}");
                        }
                    }
                    MediaCmd::UpdatePlayback { playing, position_secs } => {
                        let progress = Some(MediaPosition(Duration::from_secs_f64(position_secs)));
                        let playback = if playing {
                            MediaPlayback::Playing { progress }
                        } else {
                            MediaPlayback::Paused { progress }
                        };
                        if let Err(e) = controls.set_playback(playback) {
                            eprintln!("[media_session] set_playback error: {e:?}");
                        }
                    }
                }
            }
        });

        MediaSessionState { tx }
    }

    pub fn update_metadata(
        &self,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        duration_secs: Option<f64>,
        cover_url: Option<String>,
    ) {
        let _ = self.tx.try_send(MediaCmd::UpdateMetadata {
            title,
            artist,
            album,
            duration_secs,
            cover_url,
        });
    }

    pub fn update_playback(&self, playing: bool, position_secs: f64) {
        let _ = self.tx.try_send(MediaCmd::UpdatePlayback { playing, position_secs });
    }
}

/// Extract embedded album art from `file_path` and write it to a fixed temp
/// file path (`$TMPDIR/orchestra-art.jpg`). Returns a `file://` URI on success.
///
/// Using a fixed path avoids accumulating orphaned temp files — the single
/// file is overwritten on each track change.
pub fn extract_cover(file_path: &str) -> Option<String> {
    let path = std::path::Path::new(file_path);
    let tagged_file = lofty::read_from_path(path).ok()?;

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag())?;

    let picture = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first())?;

    let temp_path = std::env::temp_dir().join("orchestra-art.jpg");
    std::fs::write(&temp_path, picture.data()).ok()?;

    Some(format!("file://{}", temp_path.display()))
}
