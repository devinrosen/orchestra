use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;

use rodio::{Decoder, OutputStream, Sink};

pub enum PlayerCmd {
    Play(String),
    Pause,
    Resume,
    SetVolume(f32),
    Stop,
}

pub struct PlayerHandle {
    sender: mpsc::Sender<PlayerCmd>,
    pub errors: mpsc::Receiver<String>,
}

impl PlayerHandle {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlayerCmd>();
        let (err_tx, err_rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            // Create the output stream on this thread. Keep _stream alive for
            // the lifetime of the thread so the audio device stays open.
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(pair) => pair,
                Err(e) => {
                    let _ = err_tx.send(format!("Audio device error: {e}"));
                    return;
                }
            };

            let mut current_sink: Option<Sink> = None;
            let mut current_volume: f32 = 1.0;

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    PlayerCmd::Play(path) => {
                        // Drop old sink to stop previous playback.
                        current_sink = None;

                        let file = match File::open(&path) {
                            Ok(f) => f,
                            Err(e) => {
                                let _ = err_tx
                                    .send(format!("Cannot open file \"{path}\": {e}"));
                                continue;
                            }
                        };

                        let decoder = match Decoder::new(BufReader::new(file)) {
                            Ok(d) => d,
                            Err(e) => {
                                let _ = err_tx
                                    .send(format!("Cannot decode \"{path}\": {e}"));
                                continue;
                            }
                        };

                        let sink = match Sink::try_new(&stream_handle) {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = err_tx
                                    .send(format!("Cannot create sink: {e}"));
                                continue;
                            }
                        };
                        sink.set_volume(current_volume);
                        sink.append(decoder);
                        sink.play();
                        current_sink = Some(sink);
                    }
                    PlayerCmd::Pause => {
                        if let Some(ref s) = current_sink {
                            s.pause();
                        }
                    }
                    PlayerCmd::Resume => {
                        if let Some(ref s) = current_sink {
                            s.play();
                        }
                    }
                    PlayerCmd::SetVolume(v) => {
                        current_volume = v.clamp(0.0, 1.0);
                        if let Some(ref s) = current_sink {
                            s.set_volume(current_volume);
                        }
                    }
                    PlayerCmd::Stop => {
                        current_sink = None;
                    }
                }
            }
        });

        PlayerHandle {
            sender: cmd_tx,
            errors: err_rx,
        }
    }

    pub fn play(&self, path: String) {
        let _ = self.sender.send(PlayerCmd::Play(path));
    }

    pub fn pause(&self) {
        let _ = self.sender.send(PlayerCmd::Pause);
    }

    pub fn resume(&self) {
        let _ = self.sender.send(PlayerCmd::Resume);
    }

    pub fn set_volume(&self, v: f32) {
        let _ = self.sender.send(PlayerCmd::SetVolume(v));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PlayerCmd::Stop);
    }
}
