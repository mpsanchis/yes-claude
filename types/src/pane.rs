use serde::{Deserialize, Serialize};

use crate::AIAgent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneMetadata {
    pub session_name: String,
    pub window_index: u32,
    pub pane_index: u32,
    pub agent: AIAgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIAgentPane {
    /// Global ID of the pane across sessions and windows (tmux's `%N`, without the `%`).
    pub id: u32,
    pub metadata: PaneMetadata,
}

impl AIAgentPane {
    /// Parses a line of `#{pane_id} #{session_name} #{window_index} #{pane_index} #{pane_current_command}`.
    /// Returns `None` if the line is malformed or the pane isn't running a known AI agent.
    pub fn from_raw_tmux_output(line: &str) -> Option<Self> {
        let mut columns = line.split_whitespace();
        let id = columns.next()?.strip_prefix('%')?.parse().ok()?;
        let session_name = columns.next()?.to_owned();
        let window_index = columns.next()?.parse().ok()?;
        let pane_index = columns.next()?.parse().ok()?;
        let agent = AIAgent::from_command(columns.next()?)?;

        Some(Self {
            id,
            metadata: PaneMetadata {
                session_name,
                window_index,
                pane_index,
                agent,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_panes() {
        let pane = AIAgentPane::from_raw_tmux_output("%17 conformance 0 1 claude").unwrap();
        assert_eq!(pane.id, 17);
        assert_eq!(pane.metadata.session_name, "conformance");
        assert_eq!(pane.metadata.window_index, 0);
        assert_eq!(pane.metadata.pane_index, 1);
        assert_eq!(pane.metadata.agent, AIAgent::Claude);

        let codex = AIAgentPane::from_raw_tmux_output("%20 todo 0 2 codex").unwrap();
        assert_eq!(codex.metadata.agent, AIAgent::Codex);
        let agy = AIAgentPane::from_raw_tmux_output("%16 nx 0 1 agy").unwrap();
        assert_eq!(agy.metadata.agent, AIAgent::Antigravity);
    }

    #[test]
    fn skips_non_agent_and_malformed_lines() {
        assert_eq!(
            AIAgentPane::from_raw_tmux_output("%2 conformance 0 0 nvim"),
            None
        );
        assert_eq!(AIAgentPane::from_raw_tmux_output("garbage"), None);
        assert_eq!(AIAgentPane::from_raw_tmux_output(""), None);
    }
}
