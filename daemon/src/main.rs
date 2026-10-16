use tokio::sync::{broadcast, mpsc};
use yes_claude_api::{ModelStateEvent, UpdateModelEvent};
use yes_claude_tmux_actuator::send_yes_to_tmux;
use yes_claude_tmux_model::TmuxAIAgentPanesModel;
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
    let socket_outbound_rx = outbound_rx.resubscribe();
    let socket_inbound_tx = inbound_tx.clone();
    tokio::spawn(async move {
        // Without its socket the daemon can't serve anyone, so don't linger silently.
        if socket_server::run(socket_outbound_rx, socket_inbound_tx)
            .await
            .is_err()
        {
            std::process::exit(1);
        }
    });

    let mut model = TmuxAIAgentPanesModel::new();
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
