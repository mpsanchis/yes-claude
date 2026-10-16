use serde::{Deserialize, Serialize};

/// The AI coding agents we can recognise in a tmux pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIAgent {
    Claude,
    Codex,
    Antigravity,
}

impl AIAgent {
    pub const ALL: [AIAgent; 3] = [AIAgent::Claude, AIAgent::Codex, AIAgent::Antigravity];

    /// The command name tmux reports (`#{pane_current_command}`) while this agent is running.
    pub fn as_str(&self) -> &'static str {
        match self {
            AIAgent::Claude => "claude",
            AIAgent::Codex => "codex",
            AIAgent::Antigravity => "agy",
        }
    }

    /// Inverse of [`AIAgent::as_str`]: `None` if `command` isn't a known agent.
    pub fn from_command(command: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|agent| agent.as_str() == command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_names_round_trip() {
        for agent in AIAgent::ALL {
            assert_eq!(AIAgent::from_command(agent.as_str()), Some(agent));
        }
        assert_eq!(AIAgent::from_command("nvim"), None);
    }
}
