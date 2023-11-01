use builders::build;
use crossbeam_channel::{self, unbounded};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, KeyEvent};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use rand::distributions::Bernoulli;
use rand::prelude::Distribution;
use rand::{seq::SliceRandom, thread_rng};
use ratatui::widgets::Widget;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Color, Frame, Modifier, Rect},
    style::Style,
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph, ScrollDirection, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use solvers::solve;
use tui_textarea::{Input, TextArea};

use std::rc::Rc;
use std::{
    thread,
    time::{Duration, Instant},
};

pub static PLACEHOLDER: &str = "Type Command or Press <ENTER> for Random";
pub type CtEvent = crossterm::event::Event;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;
pub type Err = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Err>;

pub struct BuildFrame<'a> {
    maze: &'a maze::Maze,
}

pub struct SolveFrame<'a> {
    maze: &'a maze::Maze,
}

impl<'a> BuildFrame<'a> {
    fn new(m: &'a maze::Maze) -> Self {
        Self { maze: m }
    }
    fn row_size(&self) -> i32 {
        self.maze.row_size()
    }
    fn col_size(&self) -> i32 {
        self.maze.col_size()
    }
}

impl<'a> SolveFrame<'a> {
    fn new(m: &'a maze::Maze) -> Self {
        Self { maze: m }
    }
    fn row_size(&self) -> i32 {
        self.maze.row_size()
    }
    fn col_size(&self) -> i32 {
        self.maze.col_size()
    }
}

impl<'a> Widget for BuildFrame<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for r in 0..self.row_size() {
            for c in 0..self.col_size() {
                build::update_buffer(self.maze, &area, buf, maze::Point { row: r, col: c });
            }
        }
    }
}

impl<'a> Widget for SolveFrame<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for r in 0..self.row_size() {
            for c in 0..self.col_size() {
                solve::update_buffer(self.maze, &area, buf, maze::Point { row: r, col: c });
            }
        }
    }
}

// Event is a crowded name so we'll call it a pack.
#[derive(Debug)]
pub enum Pack {
    Tick,
    Press(KeyEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    pub sender: crossbeam_channel::Sender<Pack>,
    pub receiver: crossbeam_channel::Receiver<Pack>,
    pub handler: thread::JoinHandle<()>,
}

pub struct Tui<'a> {
    pub terminal: CrosstermTerminal,
    pub events: EventHandler,
    pub scroll: Scroller,
    pub cmd: TextArea<'a>,
}

#[derive(Clone, Copy, Debug)]
pub struct Dimension {
    pub rows: i32,
    pub cols: i32,
    pub offset: maze::Offset,
}

#[derive(Default)]
pub struct Scroller {
    pub state: ScrollbarState,
    pub pos: usize,
}

impl Scroller {
    pub fn scroll(&mut self, dir: ScrollDirection) {
        match dir {
            ScrollDirection::Forward => {
                self.pos = self.pos.saturating_add(2);
                self.state = self.state.position(self.pos);
            }
            ScrollDirection::Backward => {
                self.pos = self.pos.saturating_sub(2);
                self.state = self.state.position(self.pos);
            }
        }
    }
}

impl<'a> Tui<'a> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        let mut cmd_prompt = TextArea::default();
        cmd_prompt.set_cursor_line_style(Style::default());
        cmd_prompt.set_placeholder_text(PLACEHOLDER);
        let text_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::new().fg(Color::Yellow))
            .style(Style::default().bg(Color::Black));
        cmd_prompt.set_block(text_block);
        cmd_prompt.set_alignment(Alignment::Center);
        Self {
            terminal,
            events,
            scroll: Scroller::default(),
            cmd: cmd_prompt,
        }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

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

    pub fn inner_maze_rect(&mut self) -> Rc<[Rect]> {
        let f = self.terminal.get_frame();
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(f.size())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
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
        self.terminal
            .draw(|frame| ui_home(&mut self.cmd, &mut self.scroll, frame))?;
        Ok(())
    }

    pub fn background_maze(&mut self) -> Result<()> {
        self.terminal.draw(|frame| ui_bg_maze(frame))?;
        Ok(())
    }

    pub fn scroll(&mut self, dir: ScrollDirection) {
        self.scroll.scroll(dir)
    }

    pub fn error_popup(&mut self, msg: String) -> Result<()> {
        self.terminal.draw(|f| ui_err(&msg, f))?;
        'reading_message: loop {
            match self.events.next()? {
                Pack::Press(_) | Pack::Resize(_, _) => {
                    break 'reading_message;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn info_popup(&mut self, scroll: &mut Scroller, msg: &str) -> Result<()> {
        self.terminal.draw(|f| ui_info(msg, scroll, f))?;
        Ok(())
    }

    pub fn info_prompt(&mut self) -> Result<()> {
        self.terminal.draw(|f| ui_info_prompt(f))?;
        Ok(())
    }

    pub fn cmd_input(&mut self, input: Input) -> bool {
        self.cmd.input(input)
    }

    pub fn render_builder_frame(
        &mut self,
        step: Option<&[maze::Delta]>,
        replay_maze: &mut maze::Maze,
        forward: bool,
        rect: &Rc<[Rect]>,
    ) -> Result<()> {
        let popup_layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 17) / 2),
                Constraint::Min(3),
                Constraint::Percentage((100 - 17) / 2),
            ])
            .split(rect[1]);
        let popup_layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - 50) / 2),
                Constraint::Percentage(50),
                Constraint::Percentage((100 - 50) / 2),
            ])
            .split(popup_layout_v[1])[1];
        let popup_instructions = Paragraph::new("Toggle <i> for more <i>nfo. Exit <esc>.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        if let Some(history) = step {
            if forward {
                for delta in history {
                    replay_maze[delta.p.row as usize][delta.p.col as usize] = delta.after;
                }
            } else {
                for delta in history.iter().rev() {
                    replay_maze[delta.p.row as usize][delta.p.col as usize] = delta.before;
                }
            }
        }
        self.terminal.draw(|f| {
            f.render_widget(BuildFrame::new(replay_maze), rect[0]);
            f.render_widget(popup_instructions, popup_layout_h);
        })?;
        Ok(())
    }

    pub fn render_solver_frame(&mut self, maze: &maze::Maze, rect: &Rc<[Rect]>) -> Result<()> {
        let popup_layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 15) / 2),
                Constraint::Min(3),
                Constraint::Percentage((100 - 15) / 2),
            ])
            .split(rect[1]);
        let popup_layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - 50) / 2),
                Constraint::Percentage(50),
                Constraint::Percentage((100 - 50) / 2),
            ])
            .split(popup_layout_v[1])[1];
        let popup_instructions = Paragraph::new("Toggle <i> for more <i>nfo. Exit <esc>.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        self.terminal.draw(|f| {
            f.render_widget(SolveFrame::new(maze), rect[0]);
            f.render_widget(popup_instructions, popup_layout_h);
        })?;
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
                        .checked_sub(Instant::now().elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("no events available") {
                        match event::read().expect("unable to read event") {
                            CtEvent::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Pack::Press(e)).expect("couldn't send.");
                                }
                            }
                            CtEvent::Resize(w, h) => {
                                sender.send(Pack::Resize(w, h)).expect("could not send.");
                            }
                            _ => {}
                        }
                    }
                    // Ticks are important for some submodule channel communications.
                    if last_tick.elapsed() >= tick_rate {
                        //sender.send(Pack::Tick).expect("failed to send tick event");
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
    pub fn next(&self) -> Result<Pack> {
        Ok(self.receiver.recv()?)
    }

    pub fn try_next(&self) -> Option<Pack> {
        self.receiver.try_recv().ok()
    }
}

// The UI section has a lot of boilerplate, repetition, and magic numbers but at least we can edit
// it in one place. Look into more sane organization.

fn ui_bg_maze(f: &mut Frame<'_>) {
    let frame_block = Block::default().padding(Padding::new(1, 1, 1, 1));
    let mut background_maze = tables::MazeRunner::new();
    let mut rng = thread_rng();
    background_maze.args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not found."),
    };
    let modification_probability = Bernoulli::new(0.2);
    background_maze.modify = None;
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        background_maze.modify = match tables::MODIFICATIONS.choose(&mut rng) {
            Some(&m) => Some(m.1),
            None => print::maze_panic!("Modification table empty."),
        }
    }
    let mut rng = thread_rng();
    background_maze.build.0 = match &tables::BUILDERS.choose(&mut rng) {
        Some(b) => b.1 .0,
        None => print::maze_panic!("Builder table empty!"),
    };
    let inner = frame_block.inner(f.size());
    background_maze.args.odd_rows = inner.height as i32;
    background_maze.args.odd_cols = inner.width as i32;
    background_maze.args.offset = maze::Offset {
        add_rows: inner.y as i32,
        add_cols: inner.x as i32,
    };
    if background_maze.args.style == maze::MazeStyle::Mini {
        background_maze.args.odd_rows *= 2;
    }
    let mut bg_maze = maze::Maze::new(background_maze.args);
    background_maze.build.0(&mut bg_maze);
    if let Some(m) = background_maze.modify {
        m.0(&mut bg_maze);
    }
    let monitor = monitor::Solver::new(bg_maze);
    background_maze.solve.0(monitor.clone());
    match monitor.clone().lock() {
        Ok(lk) => solve::print_paths(&lk.maze),
        Err(_) => print::maze_panic!("Home screen broke."),
    }
}

fn ui_home(cmd: &mut TextArea, scroll: &mut Scroller, f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());
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
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left)
        .scroll((scroll.pos as u16, 0));
    f.render_widget(popup_instructions, popup_layout_h);
    scroll.state = scroll.state.content_length(INSTRUCTIONS_LINE_COUNT);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        popup_layout_h,
        &mut scroll.state,
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
    let tb = cmd.widget();
    f.render_widget(tb, text_h);
}

fn ui_info_prompt(f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.size());
    let popup_layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 15) / 2),
            Constraint::Min(3),
            Constraint::Percentage((100 - 15) / 2),
        ])
        .split(overall_layout[1]);
    let popup_layout_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Percentage(50),
            Constraint::Percentage((100 - 50) / 2),
        ])
        .split(popup_layout_v[1])[1];
    let popup_instructions = Paragraph::new("Toggle <i> for more <i>nfo. Exit <esc>.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    f.render_widget(popup_instructions, popup_layout_h);
}

fn ui_info(msg: &str, scroll: &mut Scroller, f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(65)])
        .split(f.size());
    let popup_layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 70) / 2),
            Constraint::Percentage(70),
            Constraint::Percentage((100 - 70) / 2),
        ])
        .split(overall_layout[0]);
    let popup_layout_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 80) / 2),
            Constraint::Percentage(80),
            Constraint::Percentage((100 - 80) / 2),
        ])
        .split(popup_layout_v[1])[1];
    let popup_instructions = Paragraph::new(msg)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll.pos as u16, 0));
    f.render_widget(popup_instructions, popup_layout_h);
    scroll.state = scroll.state.content_length(DESCRIPTION_LINE_COUNT);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        popup_layout_h,
        &mut scroll.state,
    );
}

fn ui_err(msg: &str, f: &mut Frame<'_>) {
    let overall_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());
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
    // This is just a dummy popup to show during the error. No need to track scroll state.
    let popup_instructions = Paragraph::new(INSTRUCTIONS)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);
    f.render_widget(popup_instructions, popup_layout_h);
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
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(text_block, text_h);
    let err_layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 15) / 2),
            Constraint::Min(4),
            Constraint::Percentage((100 - 15) / 2),
        ])
        .split(f.size());
    let err_layout_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 30) / 2),
            Constraint::Percentage(30),
            Constraint::Percentage((100 - 30) / 2),
        ])
        .split(err_layout_v[1])[1];
    let err_instructions = Paragraph::new(msg)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Red).bg(Color::Red))
                .style(
                    Style::default()
                        .bg(Color::Black)
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center);
    f.render_widget(Clear, err_layout_h);
    f.render_widget(err_instructions, err_layout_h);
}

static INSTRUCTIONS: &str = include_str!("../../res/instructions.txt");
static INSTRUCTIONS_LINE_COUNT: usize = 70;
static DESCRIPTION_LINE_COUNT: usize = 50;
