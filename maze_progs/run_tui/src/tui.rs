use builders::build;
use crossbeam_channel::{self, unbounded};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, KeyEvent};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::widgets::Widget;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Color, Modifier, Rect},
    style::Style,
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, ScrollDirection, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use solvers::solve;
use tui_textarea::{Input, TextArea};

use std::rc::Rc;
use std::{
    cmp, thread,
    time::{Duration, Instant},
};

pub static PLACEHOLDER: &str = "Type Command or Press <ENTER> for Random";
pub type CtEvent = crossterm::event::Event;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;
pub type Err = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Err>;

#[derive(Copy, Clone)]
pub enum Process {
    Building,
    Solving,
}

pub struct BuildFrame<'a> {
    pub maze: &'a maze::Blueprint,
}

pub struct SolveFrame<'a> {
    pub maze: &'a maze::Blueprint,
}

impl<'a> Widget for BuildFrame<'a> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        if self.maze.is_mini() {
            let buf_area = buf.area;
            let row_len = cmp::min(buf_area.height, (self.maze.rows / 2) as u16);
            let col_len = cmp::min(buf_area.width, self.maze.cols as u16);
            for y in 0..row_len * 2 + 1 {
                for x in 0..col_len {
                    *buf.get_mut(x, y / 2) = build::decode_mini_square(
                        self.maze,
                        maze::Point {
                            row: y as i32,
                            col: x as i32,
                        },
                    );
                }
            }
        } else {
            let buf_area = buf.area;
            let row_len = cmp::min(buf_area.height, self.maze.rows as u16);
            let col_len = cmp::min(buf_area.width, self.maze.cols as u16);
            let wall_row = &maze::wall_row(self.maze.wall_style_index);
            let cols = col_len as usize;
            for y in 0..row_len {
                for x in 0..col_len {
                    *buf.get_mut(x, y) = build::decode_square(
                        wall_row,
                        self.maze.buf[y as usize * cols + x as usize],
                    );
                }
            }
        }
    }
}

impl<'a> Widget for SolveFrame<'a> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        if self.maze.is_mini() {
            let buf_area = buf.area;
            let row_len = cmp::min(buf_area.height, (self.maze.rows / 2) as u16);
            let col_len = cmp::min(buf_area.width, self.maze.cols as u16);
            for y in 0..row_len * 2 + 1 {
                for x in 0..col_len {
                    *buf.get_mut(x, y / 2) = solve::decode_mini_path(
                        self.maze,
                        maze::Point {
                            row: y as i32,
                            col: x as i32,
                        },
                    );
                }
            }
        } else {
            let buf_area = buf.area;
            let row_len = cmp::min(buf_area.height, self.maze.rows as u16);
            let col_len = cmp::min(buf_area.width, self.maze.cols as u16);
            let wall_row = &maze::wall_row(self.maze.wall_style_index);
            let cols = col_len as usize;
            for y in 0..row_len {
                for x in 0..col_len {
                    *buf.get_mut(x, y) = solve::decode_square(
                        wall_row,
                        self.maze.buf[y as usize * cols + x as usize],
                    );
                }
            }
        }
    }
}

// Event is a crowded name so we'll call it a pack.
#[derive(Debug)]
pub enum Pack {
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
            rows: (upper_portion.height - 2) as i32,
            cols: (upper_portion.width - 2) as i32,
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

    pub fn padded_frame(&mut self) -> Rect {
        let frame_block = Block::default();
        frame_block.inner(self.terminal.get_frame().size())
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
    pub fn home_animated(&mut self, maze_frame: impl Widget) -> Result<()> {
        let frame = self.padded_frame();
        self.terminal.draw(|f| {
            f.render_widget(maze_frame, frame);
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
                .scroll((self.scroll.pos as u16, 0));
            f.render_widget(Clear, popup_layout_h);
            f.render_widget(popup_instructions, popup_layout_h);
            self.scroll.state = self.scroll.state.content_length(INSTRUCTIONS_LINE_COUNT);
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .begin_symbol(Some("⮝"))
                    .end_symbol(Some("⮟")),
                popup_layout_h,
                &mut self.scroll.state,
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
            let tb = self.cmd.widget();
            f.render_widget(Clear, text_h);
            f.render_widget(tb, text_h);
        })?;
        Ok(())
    }

    pub fn scroll(&mut self, dir: ScrollDirection) {
        self.scroll.scroll(dir)
    }

    pub fn error_popup(&mut self, msg: String, maze_frame: impl Widget) -> Result<()> {
        let frame = self.padded_frame();
        self.terminal.draw(|f| {
            f.render_widget(maze_frame, frame);
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
                        .style(Style::default().bg(Color::DarkGray)),
                )
                .alignment(Alignment::Left)
                .scroll((self.scroll.pos as u16, 0));
            f.render_widget(Clear, popup_layout_h);
            f.render_widget(popup_instructions, popup_layout_h);
            self.scroll.state = self.scroll.state.content_length(INSTRUCTIONS_LINE_COUNT);
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .begin_symbol(Some("⮝"))
                    .end_symbol(Some("⮟")),
                popup_layout_h,
                &mut self.scroll.state,
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
            let tb = self.cmd.widget();
            f.render_widget(Clear, text_h);
            f.render_widget(tb, text_h);
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
        })?;
        'reading_message: loop {
            if self.events.try_next().is_some() {
                break 'reading_message;
            }
        }
        Ok(())
    }

    pub fn info_popup(
        &mut self,
        process: Process,
        rect: &Rc<[Rect]>,
        replay_maze: &maze::Blueprint,
        scroll: &mut Scroller,
        msg: &str,
    ) -> Result<()> {
        self.terminal.draw(|f| {
            match process {
                Process::Building => f.render_widget(BuildFrame { maze: replay_maze }, rect[0]),
                Process::Solving => f.render_widget(SolveFrame { maze: replay_maze }, rect[0]),
            }
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
            f.render_widget(Clear, popup_layout_h);
            f.render_widget(popup_instructions, popup_layout_h);
            scroll.state = scroll.state.content_length(DESCRIPTION_LINE_COUNT);
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .begin_symbol(Some("⮝"))
                    .end_symbol(Some("⮟")),
                popup_layout_h,
                &mut scroll.state,
            );
        })?;
        Ok(())
    }

    pub fn cmd_input(&mut self, input: Input) -> bool {
        self.cmd.input(input)
    }

    pub fn render_maze_frame(
        &mut self,
        frame: impl Widget,
        rect: &Rc<[Rect]>,
        forward: bool,
        pause: bool,
    ) -> Result<()> {
        let popup_layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 17) / 2),
                Constraint::Min(4),
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
        let popup_instructions = Paragraph::new(POPUP_INSTRUCTIONS)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(match (pause, forward) {
                        (true, _) => RED_PAUSE,
                        (false, true) => GREEN_FORWARD,
                        (false, false) => BLUE_REVERSE,
                    }))
                    .style(Style::default().bg(Color::Black)),
            )
            .alignment(Alignment::Center);
        self.terminal.draw(|f| {
            f.render_widget(frame, rect[0]);
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
    pub fn _next(&self) -> Result<Pack> {
        Ok(self.receiver.recv()?)
    }

    /// Receive the next event but only try to receive and return an option with a
    /// pack if one is available. Makes for easier conditions to check in render loops.
    pub fn try_next(&self) -> Option<Pack> {
        self.receiver.try_recv().ok()
    }
}

static INSTRUCTIONS: &str = include_str!("../../res/instructions.txt");
static INSTRUCTIONS_LINE_COUNT: usize = 70;
static DESCRIPTION_LINE_COUNT: usize = 50;
static POPUP_INSTRUCTIONS: &str =
    "<i>info <esc>exit <space>play/pause\n<←/→>backstep/nextstep <↑/↓>faster/slower";
const RED_PAUSE: Color = Color::Rgb(201, 77, 83);
const GREEN_FORWARD: Color = Color::Rgb(77, 201, 81);
const BLUE_REVERSE: Color = Color::Rgb(42, 111, 222);
