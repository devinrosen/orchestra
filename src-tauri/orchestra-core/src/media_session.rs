use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

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
pub enum RemoteMediaEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Seek(Duration),
}

/// Spawn a dedicated media-session thread that owns `MediaControls`.
///
/// The returned `SyncSender` is the only way to push metadata/playback updates
/// into the thread. Remote events from the OS (play, pause, seek …) are
/// delivered by calling `on_remote_event`; each crate supplies its own closure
/// to route events appropriately (e.g. Tauri emits to the frontend; TUI sends
/// through an mpsc channel to the app loop).
///
/// `MediaControls` is `!Send` on macOS, so it must stay on the thread it was
/// created on — this function upholds that invariant.
pub fn spawn_session_thread(
    dbus_name: &'static str,
    display_name: &'static str,
    on_remote_event: impl Fn(RemoteMediaEvent) + Send + 'static,
) -> mpsc::SyncSender<MediaCmd> {
    let (tx, rx) = mpsc::sync_channel::<MediaCmd>(32);

    thread::spawn(move || {
        let config = PlatformConfig {
            dbus_name,
            display_name,
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
                MediaControlEvent::Toggle => Some(RemoteMediaEvent::Toggle),
                MediaControlEvent::Next => Some(RemoteMediaEvent::Next),
                MediaControlEvent::Previous => Some(RemoteMediaEvent::Previous),
                MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                    Some(RemoteMediaEvent::Seek(pos))
                }
                _ => None,
            };
            if let Some(ev) = remote_event {
                on_remote_event(ev);
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

    tx
}
