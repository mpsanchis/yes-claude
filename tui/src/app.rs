use std::io::Write;
use std::os::unix::net::UnixStream;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use yes_claude_api::{ModelStateEvent, UpdateModelEvent};
use yes_claude_types::PaneState;

use crate::events::AppEvent;

/// UI state. The daemon's model is the source of truth: toggling only sends a request,
/// and the pane flips on screen once the daemon broadcasts the new state.
pub struct App {
    /// Connection to the daemon, used for sending `UpdateModelEvent`s.
    stream: UnixStream,
    state: Option<ModelStateEvent>,
    /// Selected by pane id (not by row), so the selection survives panes appearing or disappearing.
    selected: Option<u32>,
    pub disconnected: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            state: None,
            selected: None,
            disconnected: false,
            should_quit: false,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::Daemon(state) => self.state = Some(state),
            AppEvent::DaemonDisconnected => self.disconnected = true,
        }
    }

    pub fn has_state(&self) -> bool {
        self.state.is_some()
    }

    /// Panes as `(id, state)`, ordered by session name (alphabetical), then window, then pane index.
    pub fn sorted_panes(&self) -> Vec<(u32, &PaneState)> {
        let mut panes: Vec<_> = self
            .state
            .iter()
            .flat_map(|state| state.panes.iter())
            .map(|(id, pane)| (*id, pane))
            .collect();
        panes.sort_by(|(id_a, a), (id_b, b)| {
            let (a, b) = (&a.metadata, &b.metadata);
            a.session_name
                .to_lowercase()
                .cmp(&b.session_name.to_lowercase())
                // Sessions differing only by case: keep a deterministic order.
                .then_with(|| a.session_name.cmp(&b.session_name))
                .then_with(|| a.window_index.cmp(&b.window_index))
                .then_with(|| a.pane_index.cmp(&b.pane_index))
                // Same location (shouldn't happen): fall back to the pane id.
                .then_with(|| id_a.cmp(id_b))
        });
        panes
    }

    /// Row of the selected pane. Falls back to the first row if nothing (valid) is selected.
    pub fn selected_index(&self) -> Option<usize> {
        let panes = self.sorted_panes();
        if panes.is_empty() {
            return None;
        }
        let index = self
            .selected
            .and_then(|selected| panes.iter().position(|(id, _)| *id == selected))
            .unwrap_or(0);
        Some(index)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Char(' ') | KeyCode::Enter => self.toggle_selected(),
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let panes = self.sorted_panes();
        let Some(current) = self.selected_index() else {
            return;
        };
        // Wrap around: up from the first row goes to the last, down from the last to the first.
        // `rem_euclid` (unlike `%`) is never negative, so -1 maps to the last row.
        let next = (current as isize + delta).rem_euclid(panes.len() as isize) as usize;
        self.selected = Some(panes[next].0);
    }

    fn toggle_selected(&mut self) {
        let panes = self.sorted_panes();
        let Some(index) = self.selected_index() else {
            return;
        };
        let (pane_id, yesing) = (panes[index].0, panes[index].1.yesing);
        self.send(&UpdateModelEvent::ChangePaneState {
            pane_id,
            yesing: !yesing,
        });
    }

    /// Sends one newline-delimited JSON message to the daemon.
    fn send(&mut self, event: &UpdateModelEvent) {
        let Ok(mut json) = serde_json::to_string(event) else {
            return;
        };
        json.push('\n');
        if self.stream.write_all(json.as_bytes()).is_err() {
            self.disconnected = true;
        }
    }
}
