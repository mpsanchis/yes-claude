use std::time::Duration;

use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use yes_claude_api::UpdateModelEvent;

mod commands;

pub use commands::list_claude_panes;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Polls tmux every 500ms and sends the Claude panes it finds through `tx`.
/// Runs until the receiving end of the channel is dropped.
pub async fn monitor_tmux(tx: Sender<UpdateModelEvent>) {
    loop {
        let panes = list_claude_panes().await;

        if tx.send(UpdateModelEvent::UpdatePanes(panes)).await.is_err() {
            // Receiver was dropped: nobody is listening, so stop monitoring.
            break;
        }

        sleep(POLL_INTERVAL).await;
    }
}
