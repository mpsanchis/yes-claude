use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use yes_claude_types::{ClaudePane, PaneState};

/// Tracked Claude panes, keyed by their global tmux pane id.
#[derive(Debug, Default)]
pub struct TmuxClaudePanesModel {
    panes: HashMap<u32, PaneState>,
}

impl TmuxClaudePanesModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn panes(&self) -> &HashMap<u32, PaneState> {
        &self.panes
    }

    /// Syncs the model with a snapshot of the panes currently in tmux.
    /// Known panes get their metadata refreshed (keeping `yesing`), new panes are added,
    /// and panes missing from the snapshot are dropped.
    pub fn update_panes_from(&mut self, panes: Vec<ClaudePane>) {
        let live_ids: HashSet<u32> = panes.iter().map(|pane| pane.id).collect();

        for pane in panes {
            match self.panes.entry(pane.id) {
                Entry::Occupied(mut tracked) => tracked.get_mut().metadata = pane.metadata,
                Entry::Vacant(slot) => {
                    slot.insert(PaneState::from(pane));
                }
            }
        }

        self.panes.retain(|id, _| live_ids.contains(id));
    }

    /// Sets `yesing` for a tracked pane. Unknown pane ids are ignored.
    pub fn change_pane_state(&mut self, pane_id: u32, yesing: bool) {
        if let Some(state) = self.panes.get_mut(&pane_id) {
            state.yesing = yesing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yes_claude_types::PaneMetadata;

    fn pane(id: u32, session: &str) -> ClaudePane {
        ClaudePane {
            id,
            metadata: PaneMetadata {
                session_name: session.to_owned(),
                window_index: 0,
                pane_index: 0,
            },
        }
    }

    #[test]
    fn adds_updates_and_removes_panes_keeping_yesing() {
        let mut model = TmuxClaudePanesModel::new();
        model.update_panes_from(vec![pane(1, "a"), pane(2, "b")]);
        model.change_pane_state(1, true);

        // pane 1 renamed, pane 2 gone, pane 3 new
        model.update_panes_from(vec![pane(1, "renamed"), pane(3, "c")]);

        assert_eq!(model.panes().len(), 2);
        assert_eq!(model.panes()[&1].metadata.session_name, "renamed");
        assert!(model.panes()[&1].yesing);
        assert!(!model.panes()[&3].yesing);
        assert!(!model.panes().contains_key(&2));
    }

    #[test]
    fn change_state_of_unknown_pane_is_ignored() {
        let mut model = TmuxClaudePanesModel::new();
        model.change_pane_state(42, true);
        assert!(model.panes().is_empty());
    }
}
