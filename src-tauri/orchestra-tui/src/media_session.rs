use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

// ── Public types ───────────────────────────────────────────────────────────────

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
        position: Duration,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RemoteMediaEvent {
    Play,
    Pause,
    Next,
    Previous,
    Seek(Duration),
}

/// Channel-based handle to the media session background thread.
pub struct MediaSessionHandle {
    tx: mpsc::SyncSender<MediaCmd>,
    pub remote_rx: mpsc::Receiver<RemoteMediaEvent>,
}

impl MediaSessionHandle {
    /// Spawn the media session background thread and return a handle.
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::sync_channel::<MediaCmd>(32);
        let (remote_tx, remote_rx) = mpsc::sync_channel::<RemoteMediaEvent>(32);

        thread::spawn(move || {
            let config = PlatformConfig {
                dbus_name: "com.orchestra.tui",
                display_name: "orchestra-tui",
                hwnd: None,
            };

            let mut controls = match MediaControls::new(config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[media_session] Failed to create MediaControls: {e:?}");
                    return;
                }
            };

            if let Err(e) = controls.attach(move |event: MediaControlEvent| {
                let remote_event = match event {
                    MediaControlEvent::Play => Some(RemoteMediaEvent::Play),
                    MediaControlEvent::Pause => Some(RemoteMediaEvent::Pause),
                    MediaControlEvent::Toggle => Some(RemoteMediaEvent::Play),
                    MediaControlEvent::Next => Some(RemoteMediaEvent::Next),
                    MediaControlEvent::Previous => Some(RemoteMediaEvent::Previous),
                    MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                        Some(RemoteMediaEvent::Seek(pos))
                    }
                    _ => None,
                };
                if let Some(ev) = remote_event {
                    let _ = remote_tx.try_send(ev);
                }
            }) {
                eprintln!("[media_session] Failed to attach event handler: {e:?}");
                return;
            }

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    MediaCmd::UpdateMetadata {
                        title,
                        artist,
                        album,
                        duration_secs,
                        cover_url,
                    } => {
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
                    MediaCmd::UpdatePlayback { playing, position } => {
                        let progress = Some(MediaPosition(position));
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

        MediaSessionHandle {
            tx: cmd_tx,
            remote_rx,
        }
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
