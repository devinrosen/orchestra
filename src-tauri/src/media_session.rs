use std::sync::mpsc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

pub use orchestra_core::media_session::MediaCmd;
use orchestra_core::media_session::{spawn_session_thread, RemoteMediaEvent};

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

/// Tauri-managed state for the Now Playing / media session.
///
/// Holds only a channel sender so it is `Send + Sync`. The actual
/// `MediaControls` (which is `!Send` on macOS) lives on a dedicated thread
/// inside `orchestra_core::media_session::spawn_session_thread`.
pub struct MediaSessionState {
    tx: mpsc::SyncSender<MediaCmd>,
}

impl MediaSessionState {
    /// Initialise the media session. Spawns a dedicated thread that owns
    /// `MediaControls` for the lifetime of the application.
    pub fn init(app_handle: AppHandle) -> Self {
        let ah = app_handle.clone();
        let tx = spawn_session_thread("com.orchestra.app", "Orchestra", move |event| {
            let payload = match event {
                RemoteMediaEvent::Play => Some(RemoteCommandPayload::Play),
                RemoteMediaEvent::Pause => Some(RemoteCommandPayload::Pause),
                RemoteMediaEvent::Toggle => Some(RemoteCommandPayload::Toggle),
                RemoteMediaEvent::Next => Some(RemoteCommandPayload::Next),
                RemoteMediaEvent::Previous => Some(RemoteCommandPayload::Previous),
                RemoteMediaEvent::Seek(pos) => Some(RemoteCommandPayload::Seek {
                    position: pos.as_secs_f64(),
                }),
            };
            if let Some(p) = payload {
                let _ = ah.emit("media-remote", p);
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

    pub fn update_playback(&self, playing: bool, position: Duration) {
        if let Err(e) = self
            .tx
            .try_send(MediaCmd::UpdatePlayback { playing, position })
        {
            if matches!(e, mpsc::TrySendError::Disconnected(_)) {
                eprintln!("[media_session] update_playback: channel disconnected — media session thread has exited");
            }
        }
    }
}
