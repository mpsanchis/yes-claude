use serde::{Deserialize, Serialize};

use crate::{ClaudePane, PaneMetadata};

/// Everything we know about a single pane: what tmux told us, plus our own state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneState {
    pub metadata: PaneMetadata,
    pub yesing: bool,
}

impl From<ClaudePane> for PaneState {
    fn from(pane: ClaudePane) -> Self {
        Self {
            metadata: pane.metadata,
            yesing: false,
        }
    }
}
