mod args;
mod run;
mod tables;
mod tui;
use ratatui::prelude::{CrosstermBackend, Terminal};

fn main() -> tui::Result<()> {
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = tui::EventHandler::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;
    let _status = tui.run()?;
    tui.exit()?;
    Ok(())
}
