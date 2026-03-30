use std::sync::mpsc;
use std::time::Duration;

use orchestra_core::media_session::spawn_session_thread;
pub use orchestra_core::media_session::{MediaCmd, RemoteMediaEvent};

/// Channel-based handle to the media session background thread.
pub struct MediaSessionHandle {
    tx: mpsc::SyncSender<MediaCmd>,
    pub remote_rx: mpsc::Receiver<RemoteMediaEvent>,
}

impl MediaSessionHandle {
    /// Spawn the media session background thread and return a handle.
    pub fn spawn() -> Self {
        let (remote_tx, remote_rx) = mpsc::sync_channel::<RemoteMediaEvent>(32);
        let tx = spawn_session_thread("com.orchestra.tui", "orchestra-tui", move |event| {
            let _ = remote_tx.try_send(event);
        });

        MediaSessionHandle { tx, remote_rx }
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

    pub fn update_playback(&self, playing: bool, position: Duration) {
        let _ = self
            .tx
            .try_send(MediaCmd::UpdatePlayback { playing, position });
    }
}
