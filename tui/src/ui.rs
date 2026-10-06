use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let [list_area, footer_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    if !app.has_state() {
        frame.render_widget("Waiting for daemon...", list_area);
    } else {
        let header = Row::new(["yes", "pane_id", "session:window.pane"])
            .style(Style::new().add_modifier(Modifier::BOLD));
        let rows: Vec<Row> = app
            .sorted_panes()
            .into_iter()
            .map(|(id, pane)| {
                let m = &pane.metadata;
                Row::new([
                    if pane.yesing { "[x]" } else { "[ ]" }.to_owned(),
                    format!("%{id}"),
                    format!("{}:{}.{}", m.session_name, m.window_index, m.pane_index),
                ])
            })
            .collect();
        let widths = [
            Constraint::Length(3),  // "yes" / "[x]"
            Constraint::Length(7),  // "pane_id" (ids like %1234 fit too)
            Constraint::Fill(1),    // session:window.pane
        ];
        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(3)
            .highlight_symbol("> ")
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        let mut table_state = TableState::default().with_selected(app.selected_index());
        frame.render_stateful_widget(table, list_area, &mut table_state);
    }

    let footer = if app.disconnected {
        "Daemon disconnected. q: quit"
    } else {
        "↑/↓ or j/k: move   space/enter: toggle yes   q: quit"
    };
    frame.render_widget(Paragraph::new(footer), footer_area);
}
