use std::process::Output;

use tokio::process::Command;
use yes_claude_types::AIAgentPane;

pub const TMUX_CMD: &str = "tmux";
pub const LIST_PANES: [&str; 4] = [
    "list-panes",
    "-a",
    "-F",
    "#{pane_id} #{session_name} #{window_index} #{pane_index} #{pane_current_command}",
];

fn list_panes_to_ai_agent_panes(output: Output) -> Vec<AIAgentPane> {
    if !output.status.success() {
        return vec![];
    }
    let Ok(output_str) = String::from_utf8(output.stdout) else {
        return vec![];
    };
    output_str
        .lines()
        .filter_map(AIAgentPane::from_raw_tmux_output)
        .collect()
}

/// Lists all panes (across all sessions) running an AI agent.
/// Returns an empty vec if tmux can't be run or exits with an error.
pub async fn list_ai_agent_panes() -> Vec<AIAgentPane> {
    Command::new(TMUX_CMD)
        .args(LIST_PANES)
        .output()
        .await
        .ok()
        .map(list_panes_to_ai_agent_panes)
        .unwrap_or_default()
}
