use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use yes_claude_types::{ClaudePane, PaneState};

/// Messages sent to the model to update its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateModelEvent {
    /// Snapshot of all Claude panes currently found in tmux.
    UpdatePanes(Vec<ClaudePane>),
    /// A pane started or stopped being "yesed".
    ChangePaneState { pane_id: u32, yesing: bool },
}

/// Broadcast by the model after each update: the current state of all tracked panes, by pane id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelStateEvent {
    pub panes: HashMap<u32, PaneState>,
}
