mod args;
mod run;
mod tables;
mod tui;
use builders;
use crossterm::event::{self};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::ScrollDirection,
};
use tui_textarea::{Input, Key, TextArea};

fn main() -> tui::Result<()> {
    let status = run();
    status?;
    Ok(())
}

fn run() -> tui::Result<()> {
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = tui::EventHandler::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;
    let mut quit = false;
    tui.background_maze()?;

    while !quit {
        tui.home()?;
        match tui.events.next()? {
            tui::Event::Tick => {}
            tui::Event::Key(key_event) => match key_event.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    quit = true;
                }
                event::KeyCode::Up => {
                    tui.scroll(ScrollDirection::Backward);
                }
                event::KeyCode::Down => {
                    tui.scroll(ScrollDirection::Forward);
                }
                event::KeyCode::Char('r') => {
                    tui.terminal.clear()?;
                    match run::rand_with_channels(&mut tui) {
                        _ => {
                            tui.terminal.clear()?;
                            tui.background_maze()?;
                        }
                    }
                }
                _ => {}
            },
            tui::Event::Mouse(_) => {}
            tui::Event::Resize(_, _) => {
                tui.background_maze()?;
            }
        };
    }
    tui.exit()?;
    Ok(())
}
