use std::collections::{HashMap, HashSet};
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::broadcast::{Receiver, error::RecvError};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use yes_claude_api::ModelStateEvent;

const YES_INTERVAL: Duration = Duration::from_millis(500);
const TMUX_CMD: &str = "tmux";

/// A background task that keeps saying "yes" for one pane.
/// Dropping it aborts the task (dropping a bare `JoinHandle` would only detach it).
struct YesingTask {
    handle: JoinHandle<()>,
}

impl YesingTask {
    fn spawn(pane_id: u32) -> Self {
        Self {
            handle: tokio::spawn(yes_to_pane(pane_id)),
        }
    }
}

impl Drop for YesingTask {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn yes_to_pane(pane_id: u32) {
    let target = format!("%{pane_id}");
    loop {
        send_enter(&target).await;
        sleep(YES_INTERVAL).await;
    }
}

async fn send_enter(target: &str) {
    let _ = Command::new(TMUX_CMD)
        .args(["send-keys", "-t", target, "Enter"])
        .status()
        .await;
}

/// Listens for model updates and keeps exactly one `YesingTask` per pane that is yesing.
/// Runs until the sending side of the channel is dropped.
pub async fn send_yes_to_tmux(mut model_rx: Receiver<ModelStateEvent>) {
    let mut tasks: HashMap<u32, YesingTask> = HashMap::new();

    loop {
        match model_rx.recv().await {
            Ok(state) => sync_tasks(&mut tasks, &state),
            // We were too slow and missed `skipped` updates. Only the latest state matters, so carry on.
            Err(RecvError::Lagged(_skipped)) => continue,
            // The daemon dropped the sender: nothing more will arrive.
            // `tasks` is dropped on return, which aborts any remaining task.
            Err(RecvError::Closed) => break,
        }
    }
}

/// Makes `tasks` match the panes that are yesing in `state`.
fn sync_tasks(tasks: &mut HashMap<u32, YesingTask>, state: &ModelStateEvent) {
    let yesing_ids: HashSet<u32> = state
        .panes
        .iter()
        .filter(|(_, pane)| pane.yesing)
        .map(|(id, _)| *id)
        .collect();

    // Panes no longer yesing (or gone): removing the task drops it, which aborts it.
    tasks.retain(|id, _| yesing_ids.contains(id));

    // Newly yesing panes get a task; already-tracked ones are left alone.
    for id in yesing_ids {
        tasks.entry(id).or_insert_with(|| YesingTask::spawn(id));
    }
}
