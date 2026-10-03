use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Error, Result};

fn launch_daemon() -> Result<()> {
    // Look for the daemon next to this executable (target/debug/ or target/release/, or wherever
    // both were installed) so it doesn't need to be on PATH.
    let daemon_path = std::env::current_exe()?.with_file_name("yes-claude-d");
    let _child = Command::new(daemon_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

fn attempt_connection_to_daemon(socket_path: &PathBuf) -> Result<UnixStream> {
    let mut final_error = Error::msg("Could not connect to daemon. Unknown error.");
    for _ in 0..10 {
        match UnixStream::connect(socket_path) {
            Ok(stream) => return Ok(stream),
            Err(error) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                final_error = Error::new(error);
                continue;
            }
        }
    }
    Err(final_error)
}

pub fn ensure_daemon_running() -> Result<UnixStream> {
    // Check if daemon already running
    let socket_path = yes_claude_api::get_socket_path();
    if let Ok(stream) = UnixStream::connect(&socket_path) {
        // Daemon already running: we could connect to it
        return Ok(stream);
    }
    launch_daemon()?;
    attempt_connection_to_daemon(&socket_path)
}
