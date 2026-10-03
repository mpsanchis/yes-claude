use std::env;
use std::path::PathBuf;

const SOCKET_NAME: &str = "myapp.sock";

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    runtime_dir.join(SOCKET_NAME)
}
