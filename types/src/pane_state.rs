use serde::{Deserialize, Serialize};

use crate::{AIAgentPane, PaneMetadata};

/// Everything we know about a single pane: what tmux told us, plus our own state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneState {
    pub metadata: PaneMetadata,
    pub yesing: bool,
}

impl From<AIAgentPane> for PaneState {
    fn from(pane: AIAgentPane) -> Self {
        Self {
            metadata: pane.metadata,
            yesing: false,
        }
    }
}
