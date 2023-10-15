mod args;
mod run;
mod tables;
mod tui;
use crossterm::event::{self};
use ratatui::prelude::{CrosstermBackend, Terminal};

fn main() -> tui::Result<()> {
    let status = run();
    status?;
    Ok(())
}

fn run() -> tui::Result<()> {
    let mut run = args::MazeRunner::new();
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = tui::EventHandler::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;
    let mut quit = false;

    while !quit {
        tui.draw(&mut run)?;
        match tui.events.next()? {
            tui::Event::Tick => {}
            tui::Event::Key(key_event) => match key_event.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    quit = true;
                }
                event::KeyCode::Char('r') => match run::run_with_channels(*run, &mut tui) {
                    Err(_) => {
                        tui.terminal.clear()?;
                    }
                    Ok(_) => {}
                },
                _ => {}
            },
            tui::Event::Mouse(_) => {}
            tui::Event::Resize(cols, rows) => {
                run.args.odd_rows = (rows / 2) as i32;
                run.args.odd_cols = (cols / 2) as i32;
            }
        };
    }
    tui.exit()?;
    Ok(())
}
