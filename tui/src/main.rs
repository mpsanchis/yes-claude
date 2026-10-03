use ratatui::DefaultTerminal;

mod app;
mod daemon;
mod events;
mod ui;

use app::App;
use daemon::ensure_daemon_running;
use events::spawn_event_threads;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(run).map_err(|e| color_eyre::eyre::eyre!("{e:#}"))?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let socket_connection = ensure_daemon_running()?;
    let events = spawn_event_threads(&socket_connection)?;
    let mut app = App::new(socket_connection);

    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;
        app.handle_event(events.recv()?);
    }
    Ok(())
}
