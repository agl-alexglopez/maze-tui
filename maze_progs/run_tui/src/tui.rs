use crate::args;
use crate::tables;

use builders::build;
use crossbeam_channel::{self, unbounded};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::prelude::Alignment;
use ratatui::widgets::ScrollDirection;
use ratatui::widgets::ScrollbarState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Color, CrosstermBackend},
    style::Style,
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation},
};
use solvers::solve;
use tui_textarea::{Input, Key, TextArea};

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
    pub instructions_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Dimension {
    pub rows: i32,
    pub cols: i32,
    pub offset: maze::Offset,
}

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;
pub type Err = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Err>;

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self {
            terminal,
            events,
            instructions_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
        }
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

    pub fn inner_dimensions(&mut self) -> Dimension {
        let f = self.terminal.get_frame();
        let overall_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(f.size());
        let upper_portion = overall_layout[0];
        Dimension {
            rows: (upper_portion.height - 1) as i32,
            cols: (upper_portion.width - 1) as i32,
            offset: maze::Offset {
                add_rows: upper_portion.y as i32,
                add_cols: upper_portion.x as i32,
            },
        }
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

    pub fn home(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            ui_home(
                &mut self.vertical_scroll,
                &mut self.instructions_scroll_state,
                frame,
            )
        })?;
        Ok(())
    }

    pub fn background_maze(&mut self) -> Result<()> {
        self.terminal.draw(|frame| ui_bg_maze(frame))?;
        Ok(())
    }

    pub fn scroll(&mut self, dir: ScrollDirection) {
        match dir {
            ScrollDirection::Forward => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.instructions_scroll_state = self
                    .instructions_scroll_state
                    .position(self.vertical_scroll as u16);
            }
            ScrollDirection::Backward => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.instructions_scroll_state = self
                    .instructions_scroll_state
                    .position(self.vertical_scroll as u16);
            }
        }
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

fn ui_bg_maze(f: &mut Frame<'_>) {
    let frame_block = Block::default().padding(Padding::new(1, 1, 1, 1));
    let mut background_maze = args::MazeRunner::new();
    let mut rng = thread_rng();
    background_maze.args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not found."),
    };
    background_maze.build.0 = builders::recursive_backtracker::generate_maze;
    let inner = frame_block.inner(f.size());
    background_maze.args.odd_rows = inner.height as i32;
    background_maze.args.odd_cols = inner.width as i32;
    background_maze.args.offset = maze::Offset {
        add_rows: inner.y as i32,
        add_cols: inner.x as i32,
    };
    let mut bg_maze = maze::Maze::new(background_maze.args);
    background_maze.build.0(&mut bg_maze);
    let monitor = solve::Solver::new(bg_maze);
    background_maze.solve.0(monitor.clone());
    if let Ok(lk) = monitor.clone().lock() {
        solve::print_paths(&lk.maze);
    }
}

fn ui_home(scroll: &mut usize, scroll_state: &mut ScrollbarState, f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());
    let frame_block = Block::default().padding(Padding::new(1, 1, 1, 1));
    let popup_layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 80) / 2),
            Constraint::Percentage(80),
            Constraint::Percentage((100 - 80) / 2),
        ])
        .split(overall_layout[0]);
    let popup_layout_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Min(70),
            Constraint::Percentage((100 - 50) / 2),
        ])
        .split(popup_layout_v[1])[1];
    let popup_instructions = Paragraph::new(INSTRUCTIONS)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Center)
        .scroll((*scroll as u16, 0));
    f.render_widget(frame_block, overall_layout[0]);
    f.render_widget(popup_instructions, popup_layout_h);
    // I can scroll but the scrollbar does not appear?
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        popup_layout_v[0],
        scroll_state,
    );
    let text_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 15) / 2),
            Constraint::Min(3),
            Constraint::Percentage((100 - 15) / 2),
        ])
        .split(overall_layout[1]);
    let text_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Min(70),
            Constraint::Percentage((100 - 50) / 2),
        ])
        .split(text_v[1])[1];
    let text_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));
    let mut text_box = TextArea::default();
    let default_text = DEFAULT_TEXT.replace(&['\n', '\r'], " ");
    text_box.set_placeholder_text(default_text);
    text_box.set_block(text_block);
    text_box.set_alignment(Alignment::Center);
    let tb = text_box.widget();
    f.render_widget(tb, text_h);
}

static DEFAULT_TEXT: &'static str = "Enter Flags Here";
static INSTRUCTIONS: &'static str = "
███╗   ███╗ █████╗ ███████╗███████╗    ████████╗██╗   ██╗██╗
████╗ ████║██╔══██╗╚══███╔╝██╔════╝    ╚══██╔══╝██║   ██║██║
██╔████╔██║███████║  ███╔╝ █████╗         ██║   ██║   ██║██║
██║╚██╔╝██║██╔══██║ ███╔╝  ██╔══╝         ██║   ██║   ██║██║
██║ ╚═╝ ██║██║  ██║███████╗███████╗       ██║   ╚██████╔╝██║
╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝       ╚═╝    ╚═════╝ ╚═╝

- Use flags, followed by arguments, in any order
- Press <Enter> to confirm your flag choices.

(scroll with ↓↑, exit with <q> or <esc>)

BUILDER FLAG[-b] Set maze building algorithm.
    [rdfs] - Randomized Depth First Search.
    [kruskal] - Randomized Kruskal's algorithm.
    [prim] - Randomized Prim's algorithm.
    [eller] - Randomized Eller's algorithm.
    [wilson] - Loop-Erased Random Path Carver.
    [wilso]n-walls - Loop-Erased Random Wall Adder.
    [fractal] - Randomized recursive subdivision.
    [grid] - A random grid pattern.
    [arena] - Open floor with no walls.

MODIFICATION FLAG[-m] Add shortcuts to the maze.
    [cross]- Add crossroads through the center.
    [x]- Add an x of crossing paths through center.

SOLVER FLAG[-s] Set maze solving algorithm.
    [dfs-hunt] - Depth First Search
    [dfs-gather] - Depth First Search
    [dfs-corners] - Depth First Search
    [floodfs-hunt] - Depth First Search
    [floodfs-gather] - Depth First Search
    [floodfs-corners] - Depth First Search
    [rdfs-hunt] - Randomized Depth First Search
    [rdfs-gather] - Randomized Depth First Search
    [rdfs-corners] - Randomized Depth First Search
    [bfs-hunt] - Breadth First Search
    [bfs-gather] - Breadth First Search
    [bfs-corners] - Breadth First Search
    [dark[algorithm]-[game]] - A mystery...

DRAW FLAG[-d] Set the line style for the maze.
    [sharp] - The default straight lines.
    [round] - Rounded corners.
    [doubles] - Sharp double lines.
    [bold] - Thicker straight lines.
    [contrast] - Full block width and height walls.
    [spikes] - Connected lines with spikes.

SOLVER ANIMATION FLAG[-sa] Watch the maze solution.
    [1-7] - Speed increases with number.

BUILDER ANIMATION FLAG[-ba] Watch the maze build.
    [1-7] - Speed increases with number.

Cancel any animation by pressing any key.
Zoom out/in with <Ctrl-[-/+]>
If any flags are omitted, defaults are used.
No arguments will create a random maze.

EXAMPLES:

-b rdfs -s bfs-hunt
-s bfs-gather -b prim
-s bfs-corners -d round -b fractal
-s dfs-hunt -ba 4 -sa 5 -b wilson-walls -m x
-h

Enjoy!

";
