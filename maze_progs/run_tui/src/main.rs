mod run_random;
mod tables;
use crossbeam_channel::bounded;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use print::maze_panic;
use ratatui::prelude::Constraint;
use ratatui::widgets::Borders;
use ratatui::{
    layout::{Direction, Layout},
    prelude::{CrosstermBackend, Style, Terminal},
    widgets::{Block, Padding, Row},
};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;
type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;
type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
type SolveFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

struct Tui {
    terminal: CrosstermTerminal,
    events: EventHandler,
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

struct FlagArg<'a, 'b> {
    flag: &'a str,
    arg: &'b str,
}

#[derive(Clone, Copy)]
enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

#[derive(Clone, Copy)]
struct MazeRunner {
    args: maze::MazeArgs,
    build_view: ViewingMode,
    build_speed: speed::Speed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    solve_view: ViewingMode,
    solve_speed: speed::Speed,
    solve: SolveFunction,
    do_quit: bool,
}

impl MazeRunner {
    fn new() -> Box<Self> {
        Box::new(Self {
            args: maze::MazeArgs {
                odd_rows: 33,
                odd_cols: 111,
                offset: maze::Offset::default(),
                style: maze::MazeStyle::Contrast,
            },
            build_view: ViewingMode::AnimatedPlayback,
            build_speed: speed::Speed::Speed4,
            build: (
                tables::recursive_backtracker::generate_maze,
                tables::recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::AnimatedPlayback,
            solve_speed: speed::Speed::Speed4,
            solve: (tables::dfs::hunt, tables::dfs::animate_hunt),
            do_quit: false,
        })
    }
    fn quit(&mut self) {
        self.do_quit = true;
    }
}

fn main() -> Result<()> {
    let status = run();
    status?;
    Ok(())
}

fn run() -> Result<()> {
    let tables = tables::load_function_tables();
    let mut run = MazeRunner::new();
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !run.do_quit {
        tui.draw(&mut run)?;
        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => match key_event.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    run.quit();
                }
                event::KeyCode::Char('r') => {
                    let (impatient_user, worker) = bounded::<bool>(1);
                    let (work_complete, main_loop) = bounded::<bool>(1);
                    let this_run = run.clone();
                    let builder_thread = thread::spawn(move || {
                        let mut maze = maze::Maze::new_channel(&this_run.args, worker);
                        this_run.build.1(&mut maze, this_run.build_speed);
                        match work_complete.send(true) {
                            Ok(_) => {}
                            Err(_) => maze_panic!("Worker sender disconnected."),
                        }
                    });
                    while let Err(_) = main_loop.try_recv() {
                        match tui.events.next()? {
                            Event::Tick => {}
                            Event::Key(key_event) => match key_event.code {
                                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                                    match impatient_user.send(true) {
                                        Ok(_) => break,
                                        Err(_) => maze_panic!("User couldn't exit."),
                                    }
                                }
                                _ => {}
                            },
                            Event::Resize(cols, rows) => {
                                run.args.odd_rows = (rows / 2) as i32;
                                run.args.odd_cols = (cols / 2) as i32;
                                match impatient_user.send(true) {
                                    Ok(_) => break,
                                    Err(_) => maze_panic!("User couldn't resize."),
                                }
                            }
                            _ => {}
                        }
                    }
                    builder_thread.join().unwrap();
                }
                _ => {}
            },
            Event::Mouse(_) => {}
            Event::Resize(cols, rows) => {
                run.args.odd_rows = (rows / 2) as i32;
                run.args.odd_cols = (cols / 2) as i32;
            }
        };
    }
    tui.exit()?;
    Ok(())
}

fn ui(run: &mut MazeRunner, f: &mut Frame<'_>) {
    let frame_block = Block::default()
        .title("Maze Algorithms")
        .borders(Borders::ALL)
        .padding(Padding::new(1, 5, 5, 5));
    let inner_frame = frame_block.inner(f.size());
    run.args.odd_rows = (inner_frame.height as f64 / 1.3) as i32;
    run.args.odd_cols = (inner_frame.width - inner_frame.x) as i32;
    run.args.offset = maze::Offset {
        add_rows: inner_frame.x as i32,
        add_cols: inner_frame.y as i32,
    };
    f.render_widget(frame_block, f.size());
}

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
    pub fn draw(&mut self, mut run: &mut MazeRunner) -> Result<()> {
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
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
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
