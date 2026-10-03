use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let [list_area, footer_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    if !app.has_state() {
        frame.render_widget("Waiting for daemon...", list_area);
    } else {
        let items: Vec<ListItem> = app
            .sorted_panes()
            .into_iter()
            .map(|(id, pane)| {
                let toggle = if pane.yesing { "[x]" } else { "[ ]" };
                let m = &pane.metadata;
                ListItem::new(format!(
                    "{toggle} %{id}  {}:{}.{}",
                    m.session_name, m.window_index, m.pane_index
                ))
            })
            .collect();
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        let mut list_state = ListState::default().with_selected(app.selected_index());
        frame.render_stateful_widget(list, list_area, &mut list_state);
    }

    let footer = if app.disconnected {
        "Daemon disconnected. q: quit"
    } else {
        "↑/↓ or j/k: move   space/enter: toggle yes   q: quit"
    };
    frame.render_widget(Paragraph::new(footer), footer_area);
}
