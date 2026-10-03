use std::process::Output;

use tokio::process::Command;
use yes_claude_types::ClaudePane;

pub const TMUX_CMD: &str = "tmux";
pub const LIST_PANES: [&str; 4] = [
    "list-panes",
    "-a",
    "-F",
    "#{pane_id} #{session_name} #{window_index} #{pane_index} #{pane_current_command}",
];

fn list_panes_to_claude_panes(output: Output) -> Vec<ClaudePane> {
    if !output.status.success() {
        return vec![];
    }
    let Ok(output_str) = String::from_utf8(output.stdout) else {
        return vec![];
    };
    output_str
        .lines()
        .filter_map(ClaudePane::from_raw_tmux_output)
        .collect()
}

/// Lists all panes (across all sessions) running Claude.
/// Returns an empty vec if tmux can't be run or exits with an error.
pub async fn list_claude_panes() -> Vec<ClaudePane> {
    Command::new(TMUX_CMD)
        .args(LIST_PANES)
        .output()
        .await
        .ok()
        .map(list_panes_to_claude_panes)
        .unwrap_or_default()
}
