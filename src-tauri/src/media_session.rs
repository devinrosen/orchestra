use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use tauri::{AppHandle, Emitter};

/// Serializable payload emitted to the frontend as a "media-remote" event.
#[derive(serde::Serialize, Clone)]
#[serde(tag = "type")]
enum RemoteCommandPayload {
    Play,
    Pause,
    Toggle,
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
                    MediaControlEvent::Toggle => Some(RemoteCommandPayload::Toggle),
                    MediaControlEvent::Next => Some(RemoteCommandPayload::Next),
                    MediaControlEvent::Previous => Some(RemoteCommandPayload::Previous),
                    MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                        Some(RemoteCommandPayload::Seek {
                            position: pos.as_secs_f64(),
                        })
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
                    MediaCmd::UpdateMetadata {
                        title,
                        artist,
                        album,
                        duration_secs,
                        cover_url,
                    } => {
                        let duration = duration_secs.map(Duration::from_secs_f64);
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
                    MediaCmd::UpdatePlayback {
                        playing,
                        position_secs,
                    } => {
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
        if let Err(e) = self.tx.try_send(MediaCmd::UpdateMetadata {
            title,
            artist,
            album,
            duration_secs,
            cover_url,
        }) {
            if matches!(e, mpsc::TrySendError::Disconnected(_)) {
                eprintln!("[media_session] update_metadata: channel disconnected — media session thread has exited");
            }
        }
    }

    pub fn update_playback(&self, playing: bool, position_secs: f64) {
        if let Err(e) = self.tx.try_send(MediaCmd::UpdatePlayback {
            playing,
            position_secs,
        }) {
            if matches!(e, mpsc::TrySendError::Disconnected(_)) {
                eprintln!("[media_session] update_playback: channel disconnected — media session thread has exited");
            }
        }
    }
}
