mod run_random;
use builders::arena;
use builders::eller;
use builders::grid;
use builders::kruskal;
use builders::modify;
use builders::prim;
use builders::recursive_backtracker;
use builders::recursive_subdivision;
use builders::wilson_adder;
use builders::wilson_carver;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use painters::distance;
use painters::runs;
use ratatui::widgets::Borders;
use ratatui::{
    prelude::{CrosstermBackend, Style, Terminal},
    widgets::{Block, Padding, Row, Table},
};

use solvers::bfs;
use solvers::darkbfs;
use solvers::darkdfs;
use solvers::darkfloodfs;
use solvers::darkrdfs;
use solvers::dfs;
use solvers::floodfs;
use solvers::key::ANSI_BLOCK;
use solvers::key::THREAD_COLORS;
use solvers::rdfs;

use std::collections::{HashMap, HashSet};
use std::{
    sync::mpsc,
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

enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

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
                recursive_backtracker::generate_maze,
                recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::AnimatedPlayback,
            solve_speed: speed::Speed::Speed4,
            solve: (dfs::hunt, dfs::animate_hunt),
            do_quit: false,
        })
    }
    fn quit(&mut self) {
        self.do_quit = true;
    }
}

type BoxMazeRunner = Box<MazeRunner>;

struct LookupTables {
    arg_flags: HashSet<String>,
    build_table: HashMap<String, BuildFunction>,
    mod_table: HashMap<String, BuildFunction>,
    solve_table: HashMap<String, SolveFunction>,
    style_table: HashMap<String, maze::MazeStyle>,
    animation_table: HashMap<String, speed::Speed>,
}

fn main() -> Result<()> {
    let status = run();
    status?;
    Ok(())
}

fn run() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let tables = LookupTables {
        arg_flags: HashSet::from([
            String::from("-r"),
            String::from("-c"),
            String::from("-b"),
            String::from("-s"),
            String::from("-h"),
            String::from("-d"),
            String::from("-m"),
            String::from("-sa"),
            String::from("-ba"),
        ]),
        build_table: HashMap::from([
            (
                String::from("rdfs"),
                (
                    recursive_backtracker::generate_maze as fn(&mut maze::Maze),
                    recursive_backtracker::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("fractal"),
                (
                    recursive_subdivision::generate_maze as fn(&mut maze::Maze),
                    recursive_subdivision::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("grid"),
                (
                    grid::generate_maze as fn(&mut maze::Maze),
                    grid::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("prim"),
                (
                    prim::generate_maze as fn(&mut maze::Maze),
                    prim::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("kruskal"),
                (
                    kruskal::generate_maze as fn(&mut maze::Maze),
                    kruskal::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("eller"),
                (
                    eller::generate_maze as fn(&mut maze::Maze),
                    eller::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson"),
                (
                    wilson_carver::generate_maze as fn(&mut maze::Maze),
                    wilson_carver::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson-walls"),
                (
                    wilson_adder::generate_maze as fn(&mut maze::Maze),
                    wilson_adder::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("arena"),
                (
                    arena::generate_maze as fn(&mut maze::Maze),
                    arena::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        mod_table: HashMap::from([
            (
                String::from("cross"),
                (
                    modify::add_cross as fn(&mut maze::Maze),
                    modify::add_cross_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("x"),
                (
                    modify::add_x as fn(&mut maze::Maze),
                    modify::add_x_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        solve_table: HashMap::from([
            (
                String::from("dfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    dfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    dfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    dfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    bfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    bfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    bfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    floodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    floodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    floodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    rdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    rdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    rdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    darkdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    darkdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    darkdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    darkfloodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    darkfloodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    darkfloodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    darkrdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    darkrdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    darkrdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    darkbfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    darkbfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    darkbfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("distance"),
                (
                    distance::paint_distance_from_center as fn(maze::BoxMaze),
                    distance::animate_distance_from_center as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("runs"),
                (
                    runs::paint_run_lengths as fn(maze::BoxMaze),
                    runs::animate_run_lengths as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
        ]),
        style_table: HashMap::from([
            (String::from("sharp"), maze::MazeStyle::Sharp),
            (String::from("round"), maze::MazeStyle::Round),
            (String::from("doubles"), maze::MazeStyle::Doubles),
            (String::from("bold"), maze::MazeStyle::Bold),
            (String::from("contrast"), maze::MazeStyle::Contrast),
            (String::from("spikes"), maze::MazeStyle::Spikes),
        ]),
        animation_table: HashMap::from([
            (String::from("0"), speed::Speed::Instant),
            (String::from("1"), speed::Speed::Speed1),
            (String::from("2"), speed::Speed::Speed2),
            (String::from("3"), speed::Speed::Speed3),
            (String::from("4"), speed::Speed::Speed4),
            (String::from("5"), speed::Speed::Speed5),
            (String::from("6"), speed::Speed::Speed6),
            (String::from("7"), speed::Speed::Speed7),
        ]),
    };
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
            Event::Key(key_event) => update(&mut run, key_event)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
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
    run.args.offset = maze::Offset {
        add_rows: inner_frame.x as i32,
        add_cols: inner_frame.y as i32,
    };
    let test = vec![vec!["Row1", "Row1", "Row1"], vec!["Row2", "Row2", "Row2"]];
    let mut rows = Vec::<Vec<String>>::new();
    for row in 1..(&THREAD_COLORS.len() / 3) {
        let col = &THREAD_COLORS[row * 3..row * 3 + 3];
        rows.push(
            THREAD_COLORS[row * 3..row * 3 + 3]
                .iter()
                .map(|s| ANSI_BLOCK.to_string() + s.binary)
                .collect(),
        );
    }

    let style = Style::default();
    let head = vec!["Header"];
    let header = Row::new(head).style(style).height(1).bottom_margin(1);

    f.render_widget(frame_block, f.size());
}

fn update(run: &mut MazeRunner, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        event::KeyCode::Char('q') | event::KeyCode::Esc => run.quit(),
        event::KeyCode::Char('r') => run_random::rand(run.args),
        _ => (),
    }
    Ok(())
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
