use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use yes_claude_api::ModelStateEvent;

/// Everything the UI loop can wake up for.
pub enum AppEvent {
    Key(KeyEvent),
    Daemon(ModelStateEvent),
    DaemonDisconnected,
}

/// Spawns one thread per blocking source (keyboard, daemon socket), both feeding the returned channel.
/// The threads end on their own once the receiver is dropped (their next `send` fails).
pub fn spawn_event_threads(stream: &UnixStream) -> Result<Receiver<AppEvent>> {
    let (tx, rx) = mpsc::channel();

    // Socket -> channel. A second handle to the same socket, so the caller keeps `stream` for writing.
    let reader = BufReader::new(stream.try_clone()?);
    let socket_tx = tx.clone();
    thread::spawn(move || {
        for line in reader.lines() {
            let Ok(line) = line else { break };
            // Ignore lines that aren't a valid state event.
            let Ok(state) = serde_json::from_str::<ModelStateEvent>(&line) else {
                continue;
            };
            if socket_tx.send(AppEvent::Daemon(state)).is_err() {
                return;
            }
        }
        let _ = socket_tx.send(AppEvent::DaemonDisconnected);
    });

    // Keyboard -> channel.
    thread::spawn(move || {
        while let Ok(event) = event::read() {
            if let Event::Key(key) = event
                && key.is_press()
                && tx.send(AppEvent::Key(key)).is_err()
            {
                return;
            }
        }
    });

    Ok(rx)
}
