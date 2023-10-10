use crossterm::terminal::EnterAlternateScreen;
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::Paragraph,
};

struct MazeRunner {
    quit: bool,
}

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

fn main() -> std::io::Result<()> {
    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}

fn run() -> std::io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut run = MazeRunner { quit: false };

    'poll: loop {
        terminal.draw(|f| {
            ui(f);
        })?;
        update(&mut run)?;
        if run.quit {
            break 'poll;
        }
    }
    Ok(())
}

fn startup() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> std::io::Result<()> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn ui(f: &mut Frame<'_>) {
    f.render_widget(Paragraph::new("Hello World! (press 'q' to quit)"), f.size());
}

fn update(run: &mut MazeRunner) -> std::io::Result<()> {
    if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => run.quit = true,
                    _ => (),
                }
            }
        }
    }
    Ok(())
}
