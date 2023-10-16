use crate::args;
use crate::tables;

use crossbeam_channel::{self, unbounded};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Color, CrosstermBackend},
    style::Style,
    widgets::{Block, Borders, Padding},
};

use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    pub sender: crossbeam_channel::Sender<Event>,
    pub receiver: crossbeam_channel::Receiver<Event>,
    pub handler: thread::JoinHandle<()>,
}

pub struct Tui {
    pub terminal: CrosstermTerminal,
    pub events: EventHandler,
}

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;
pub type Err = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Err>;

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    pub fn draw(&mut self, mut run: &mut args::MazeRunner) -> Result<()> {
        self.terminal.draw(|frame| ui(&mut run, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn splash(&mut self) -> Result<()> {
        self.terminal.draw(|frame| ui_splash(frame))?;
        Ok(())
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = unbounded();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("no events available") {
                        match event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    Ok(()) // ignore KeyEventKind::Release on windows
                                }
                            }
                            CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                            CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                            _ => unimplemented!(),
                        }
                        .expect("failed to send terminal event")
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}

fn ui(run: &mut args::MazeRunner, f: &mut Frame<'_>) {
    let frame_block = Block::default().padding(Padding::new(1, 1, 1, 1));
    let inner_frame = frame_block.inner(f.size());
    run.args.odd_rows = (inner_frame.height as f64 / 2.0) as i32;
    run.args.odd_cols = inner_frame.width as i32;
    run.args.offset = maze::Offset {
        add_rows: inner_frame.y as i32,
        add_cols: inner_frame.x as i32,
    };
    f.render_widget(frame_block, f.size());
}

fn ui_splash(f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(f.size());
    let frame_block = Block::default().padding(Padding::new(1, 1, 1, 1));
    let mut background_maze = args::MazeRunner::new();
    let mut rng = thread_rng();
    background_maze.args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not found."),
    };
    background_maze.build.0 = builders::wilson_adder::generate_maze;
    let inner = frame_block.inner(f.size());
    background_maze.args.odd_rows = inner.height as i32;
    background_maze.args.odd_cols = inner.width as i32;
    background_maze.args.offset = maze::Offset {
        add_rows: inner.y as i32,
        add_cols: inner.x as i32,
    };
    let popup_layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Percentage(50),
            Constraint::Percentage((100 - 50) / 2),
        ])
        .split(overall_layout[0]);
    let popup_layout_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Percentage(50),
            Constraint::Percentage((100 - 50) / 2),
        ])
        .split(popup_layout_v[1])[1];

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().bg(Color::Yellow).fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));
    f.render_widget(frame_block, overall_layout[0]);
    let mut bg_maze = maze::Maze::new(background_maze.args);
    background_maze.build.0(&mut bg_maze);
    f.render_widget(popup_block, popup_layout_h);
}
