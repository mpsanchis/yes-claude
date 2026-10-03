use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use yes_claude_api::{ModelStateEvent, UpdateModelEvent, get_socket_path};

/// Serves one client. Messages are newline-delimited JSON in both directions.
/// Returns when the client disconnects or either channel closes.
async fn handle_connection(
    stream: UnixStream,                                    // Server <> Clients
    mut outbound_rx: broadcast::Receiver<ModelStateEvent>, // Model -> Server
    inbound_tx: mpsc::Sender<UpdateModelEvent>,            // Server -> Model
) {
    // Split so that reading and writing can be awaited at the same time.
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    loop {
        tokio::select! {
            // Model -> Client
            result = outbound_rx.recv() => match result {
                Ok(model_state_event) => {
                    let Ok(mut json) = serde_json::to_string(&model_state_event) else {
                        continue;
                    };
                    json.push('\n');
                    if write_half.write_all(json.as_bytes()).await.is_err() {
                        break; // client went away
                    }
                }
                // Too slow: some states were skipped. Only the latest matters, so carry on.
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            },

            // Client -> Model
            result = lines.next_line() => match result {
                Ok(Some(line)) => {
                    // Ignore lines that aren't a valid event.
                    if let Ok(event) = serde_json::from_str::<UpdateModelEvent>(&line)
                        && inbound_tx.send(event).await.is_err()
                    {
                        break; // daemon is shutting down
                    }
                }
                Ok(None) | Err(_) => break, // EOF or read error: client disconnected
            },
        }
    }
}

pub async fn run(
    outbound_rx: broadcast::Receiver<ModelStateEvent>, // Model -> Server -> Clients
    inbound_tx: mpsc::Sender<UpdateModelEvent>,        // Clients -> Server -> Model
) -> Result<()> {
    // Initialize socket
    let socket_listener = UnixListener::bind(get_socket_path())?;

    // Main loop: wait for connections from clients
    // Pass over channels so that they can communicate directly
    loop {
        let (stream, _) = socket_listener.accept().await?;

        tokio::spawn(handle_connection(
            stream,
            outbound_rx.resubscribe(),
            inbound_tx.clone(),
        ));
    }
}
