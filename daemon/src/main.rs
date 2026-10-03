use tokio::sync::{broadcast, mpsc};
use yes_claude_api::{ModelStateEvent, UpdateModelEvent};
use yes_claude_tmux_actuator::send_yes_to_tmux;
use yes_claude_tmux_model::TmuxClaudePanesModel;
use yes_claude_tmux_monitor::monitor_tmux;

mod socket_server;

const EVENT_CHANNEL_BUFFER: usize = 32;
const MODEL_CHANNEL_BUFFER: usize = 16;

#[tokio::main]
async fn main() {
    // Inbound: monitor (and later the TUI) -> daemon. Many senders, one receiver.
    let (inbound_tx, mut inbound_rx) = mpsc::channel::<UpdateModelEvent>(EVENT_CHANNEL_BUFFER);
    // Outbound: daemon -> everyone interested in the model's new state. One sender, many receivers.
    let (outbound_tx, outbound_rx) = broadcast::channel::<ModelStateEvent>(MODEL_CHANNEL_BUFFER);

    tokio::spawn(monitor_tmux(inbound_tx.clone()));
    tokio::spawn(send_yes_to_tmux(outbound_rx.resubscribe()));
    tokio::spawn(socket_server::run(
        outbound_rx.resubscribe(),
        inbound_tx.clone(),
    ));

    let mut model = TmuxClaudePanesModel::new();
    while let Some(event) = inbound_rx.recv().await {
        match event {
            UpdateModelEvent::UpdatePanes(panes) => model.update_panes_from(panes),
            UpdateModelEvent::ChangePaneState { pane_id, yesing } => {
                model.change_pane_state(pane_id, yesing)
            }
        }
        _ = outbound_tx.send(ModelStateEvent {
            panes: model.panes().clone(),
        });
    }
}
