use crate::tui;
use builders::build;
use crossterm::event::KeyCode;
use rand::{distributions::Bernoulli, distributions::Distribution, seq::SliceRandom, thread_rng};
use ratatui::{
    prelude::{CrosstermBackend, Rect, Terminal},
    widgets::ScrollDirection,
};
use std::{error, fmt, rc::Rc, sync::Arc, sync::Mutex};
use tui_textarea::{Input, Key};

static VALID_FLAGS: &str = "VALID FLAGS:[-b][-ba][-s][-sa][-w][-m]";
static VALID_ARGS: [(&str, &str); 6] = [
    ("-b", "see BUILDER FLAG section"),
    ("-m", "see MODIFICATION FLAG section"),
    ("-w", "see WALL FLAG section"),
    ("-s", "see SOLVER FLAG section"),
    ("-sa", "see SOLVER ANIMATION section"),
    ("-ba", "see BUILDER ANIMATION section"),
];

#[derive(Debug)]
pub struct Quit {
    pub q: bool,
}

impl Quit {
    pub fn new() -> Self {
        Quit { q: false }
    }
}

impl fmt::Display for Quit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Quit: {}", self.q)
    }
}

impl error::Error for Quit {}

// A Playback just extracts the internal histories of a maze::Maze for ease of use with animations.
#[derive(Debug, Clone)]
struct Playback {
    maze: maze::Blueprint,
    build_tape: maze::Tape,
    solve_tape: maze::Tape,
    forward: bool,
    pause: bool,
}

///
/// Main TUI program running and logic.
///

/// The main render loop from the home page. This loop is relatively simple. When the more
/// complex functionality of a maze animation is requested we will hand that off to another fn.
pub fn run() -> tui::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = tui::EventHandler::new(4.0);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;
    let mut play = new_home_tape(tui.padded_frame());
    'render: loop {
        tui.home(tui::SolveFrame { maze: &play.maze })?;
        if let Some(ev) = tui.events.next() {
            match ev {
                tui::Pack::Resize(_, _) => {
                    play = new_home_tape(tui.padded_frame());
                }
                tui::Pack::Press(ev) => match ev.into() {
                    Input { key: Key::Esc, .. } => break 'render,
                    Input { key: Key::Down, .. } => tui.scroll(ScrollDirection::Forward),
                    Input { key: Key::Up, .. } => tui.scroll(ScrollDirection::Backward),
                    Input {
                        key: Key::Enter, ..
                    } => match set_command_args(tui.cmd.lines()[0].to_string(), &mut tui) {
                        Ok(run) => {
                            render_maze(run, &mut tui)?;
                        }
                        Err(msg) => 'reading_message: loop {
                            if let Some(ev) = tui.events.next() {
                                match ev {
                                    tui::Pack::Render => {
                                        tui.error_popup(
                                            &msg,
                                            tui::SolveFrame { maze: &play.maze },
                                        )?;
                                    }
                                    _ => break 'reading_message,
                                }
                            }
                        },
                    },
                    input => {
                        tui.cmd_input(input);
                    }
                },
                _ => {}
            }
        }
        if play.pause {
            play.forward = !play.forward;
            play.pause = !play.pause;
        } else if !play.solve_delta() {
            play.forward = true;
        }
    }
    tui.exit()?;
    Ok(())
}

// Keeping the three loops visible in one function like this makes it easier to reason about
// playing the animation forward or in reverse. The handle_press function can mutate the
// play direction but needed to extract repetitive logic that made this function harder to read.
fn render_maze(this_run: tables::HistoryRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    let render_space = tui.inner_maze_rect();
    let mut play = new_tape(&this_run);
    'rendering: loop {
        'building: loop {
            if let Some(ev) = tui.events.next() {
                match ev {
                    tui::Pack::Press(ev) => {
                        if !handle_press(
                            tui,
                            ev.code,
                            tui::Process::Building,
                            &this_run,
                            &mut play,
                            &render_space,
                        ) {
                            break 'rendering;
                        }
                    }
                    tui::Pack::Render => {
                        if !play.build_delta() {
                            break 'building;
                        }
                    }
                    tui::Pack::Resize(_, _) => break 'rendering,
                }
            }
            tui.render_maze_frame(
                tui::BuildFrame {
                    maze: &mut play.maze,
                },
                &render_space,
                play.forward,
                play.pause,
            )?;
        }
        'solving: loop {
            if let Some(ev) = tui.events.next() {
                match ev {
                    tui::Pack::Press(ev) => {
                        if !handle_press(
                            tui,
                            ev.code,
                            tui::Process::Solving,
                            &this_run,
                            &mut play,
                            &render_space,
                        ) {
                            break 'rendering;
                        }
                    }
                    tui::Pack::Render => {
                        if !play.solve_delta() {
                            break 'solving;
                        }
                    }
                    tui::Pack::Resize(_, _) => break 'rendering,
                }
            }
            tui.render_maze_frame(
                tui::SolveFrame { maze: &play.maze },
                &render_space,
                play.forward,
                play.pause,
            )?;
        }
    }
    Ok(())
}

fn handle_press(
    tui: &mut tui::Tui,
    ev: crossterm::event::KeyCode,
    process: tui::Process,
    args: &tables::HistoryRunner,
    play: &mut Playback,
    render_space: &Rc<[Rect]>,
) -> bool {
    match ev {
        KeyCode::Char('i') => {
            if handle_reader(
                tui,
                process,
                tables::load_info(&args.build),
                &play.maze,
                render_space,
            )
            .is_err()
            {
                return false;
            }
        }
        KeyCode::Char(' ') => play.pause = !play.pause,
        KeyCode::Right => {
            play.forward = true;
            play.pause = true;
            match process {
                tui::Process::Building => play.build_step(),
                tui::Process::Solving => play.solve_step(),
            };
        }
        KeyCode::Left => {
            play.forward = false;
            play.pause = true;
            match process {
                tui::Process::Building => play.build_step(),
                tui::Process::Solving => play.solve_step(),
            };
        }
        KeyCode::Esc => return false,
        _ => return true,
    }
    true
}

fn handle_reader(
    tui: &mut tui::Tui,
    process: tui::Process,
    description: &str,
    maze: &maze::Blueprint,
    render_space: &Rc<[Rect]>,
) -> tui::Result<()> {
    let mut scroll = tui::Scroller::default();
    'reading: loop {
        if let Some(k) = tui.events.next() {
            match k {
                tui::Pack::Press(k) => match k.code {
                    KeyCode::Char('i') => break 'reading,
                    KeyCode::Down => scroll.scroll(ScrollDirection::Forward),
                    KeyCode::Up => scroll.scroll(ScrollDirection::Backward),
                    KeyCode::Esc => return Err(Box::new(Quit::new())),
                    _ => {}
                },
                tui::Pack::Render => {
                    tui.info_popup(process, render_space, maze, &mut scroll, description)?;
                }
                tui::Pack::Resize(_, _) => return Err(Box::new(Quit::new())),
            }
        }
    }
    Ok(())
}

///
/// Maze generation and solving. It is simple because we don't have to worry about animations
/// until the maze generation and solving histories have been recorded. Then we decide how
/// we want to play all of that back with the help of builder and solver decoding functions.
///

// A new tape runs to completion then resets the maze buffer to its starting state.
fn new_tape(run: &tables::HistoryRunner) -> Playback {
    let monitor = monitor::Monitor::new(maze::Maze::new(run.args));
    (run.build)(monitor.clone());
    if let Some(m) = run.modify {
        (m)(monitor.clone());
    }
    (run.solve)(monitor.clone());
    match Arc::into_inner(monitor) {
        Some(a) => match Mutex::into_inner(a) {
            Ok(mut solver) => {
                build::reset_build(&mut solver.maze);
                Playback {
                    maze: solver.maze.maze,
                    build_tape: solver.maze.build_history,
                    solve_tape: solver.maze.solve_history,
                    forward: true,
                    pause: false,
                }
            }
            Err(_) => print::maze_panic!("rendering cannot progress without lock"),
        },
        None => print::maze_panic!("rendering cannot progress without lock"),
    }
}

// A new home tape solves everything but then only resets the solver for less distracting home.
fn new_home_tape(rect: Rect) -> Playback {
    let run_bg = set_random_args(&rect);
    let bg_maze = monitor::Monitor::new(maze::Maze::new(run_bg.args));
    (run_bg.build)(bg_maze.clone());
    if let Some(m) = run_bg.modify {
        (m)(bg_maze.clone());
    }
    (run_bg.solve)(bg_maze.clone());
    match Arc::into_inner(bg_maze) {
        Some(a) => match Mutex::into_inner(a) {
            Ok(mut solver) => {
                solvers::solve::reset_solve(&mut solver.maze);
                Playback {
                    maze: solver.maze.maze,
                    build_tape: solver.maze.build_history,
                    solve_tape: solver.maze.solve_history,
                    forward: true,
                    pause: false,
                }
            }
            Err(_) => print::maze_panic!("rendering cannot progress without lock"),
        },
        None => print::maze_panic!("rendering cannot progress without lock"),
    }
}

///
/// Argument parsing from the tui-textarea or random generation if empty
///

pub fn set_command_args(cmd: String, tui: &mut tui::Tui) -> Result<tables::HistoryRunner, String> {
    if cmd.is_empty() {
        return Ok(set_random_args(&tui.inner_maze_rect()[0]));
    }
    let dimensions = tui.inner_dimensions();
    let mut run = tables::HistoryRunner::new();
    run.args.odd_rows = dimensions.rows;
    run.args.odd_cols = dimensions.cols;
    run.args.offset = dimensions.offset;
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in cmd.split_whitespace() {
        if process_current {
            if let Err(msg) = set_arg(
                &mut run,
                &tables::FlagArg {
                    flag: prev_flag,
                    arg: a,
                },
            ) {
                return Err(msg.to_string());
            }
            process_current = false;
            continue;
        }
        match tables::search_table(a, &tables::FLAGS) {
            Some(flag) => {
                process_current = true;
                prev_flag = flag;
            }
            None => {
                return Err(format!(
                    "unknown flag[{}].\n{}\npress any key to continue",
                    a, VALID_FLAGS
                ));
            }
        }
    }
    if process_current {
        return Err(format!(
            "flag[{}] with missing arg[?]\n{}\npress any key to continue",
            prev_flag,
            get_arg_section(prev_flag)
        ));
    }
    if run.args.style == maze::MazeStyle::Mini {
        run.args.odd_rows *= 2;
    }
    Ok(run)
}

fn set_arg(run: &mut tables::HistoryRunner, args: &tables::FlagArg) -> Result<(), String> {
    match args.flag {
        "-b" => tables::search_table(args.arg, &tables::HISTORY_BUILDERS)
            .map(|func_pair| run.build = func_pair)
            .ok_or(err_string(args)),
        "-m" => tables::search_table(args.arg, &tables::HISTORY_MODIFICATIONS)
            .map(|mod_tuple| run.modify = Some(mod_tuple))
            .ok_or(err_string(args)),
        "-s" => tables::search_table(args.arg, &tables::HISTORY_SOLVERS)
            .map(|solve_tuple| run.solve = solve_tuple)
            .ok_or(err_string(args)),
        "-w" => tables::search_table(args.arg, &tables::WALL_STYLES)
            .map(|wall_style| run.args.style = wall_style)
            .ok_or(err_string(args)),
        _ => Err(err_string(args)),
    }
}

fn set_random_args(rect: &Rect) -> tables::HistoryRunner {
    let mut rng = thread_rng();
    let mut this_run = tables::HistoryRunner::new();
    this_run.args.odd_rows = (rect.height - 2) as i32;
    this_run.args.odd_cols = (rect.width - 2) as i32;
    this_run.args.offset = maze::Offset {
        add_rows: rect.y as i32,
        add_cols: rect.x as i32,
    };
    let modification_probability = Bernoulli::new(0.2);
    this_run.args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not set for loop, broken"),
    };
    this_run.build = match tables::HISTORY_BUILDERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Build algorithm array empty."),
    };
    this_run.solve = match tables::HISTORY_SOLVERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Solve algorithm array empty."),
    };
    this_run.modify = None;
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        this_run.modify = match tables::HISTORY_MODIFICATIONS.choose(&mut rng) {
            Some(&m) => Some(m.1),
            None => print::maze_panic!("Modification table empty."),
        }
    }
    if this_run.args.style == maze::MazeStyle::Mini {
        this_run.args.odd_rows *= 2;
    }
    this_run
}

fn err_string(args: &tables::FlagArg) -> String {
    format!("invalid flag[{}] arg[{}] combo", args.flag, args.arg)
}

fn get_arg_section(flag: &str) -> &'static str {
    VALID_ARGS
        .iter()
        .find(|(f, _)| *f == flag)
        .expect("check VALID_ARGS table.")
        .1
}

///
/// History function wrappers to help simplify what the runner is responsible for with playback.
///

// A step just progresses the Tape based on whatever the current direction state is.
impl Playback {
    fn build_step(&mut self) -> bool {
        if let Some(history) = self.build_tape.cur_step() {
            if self.forward {
                for delta in history {
                    self.maze.buf[(delta.id.row * self.maze.cols + delta.id.col) as usize] =
                        delta.after;
                }
                return self.build_tape.set_next();
            }
            for delta in history.iter().rev() {
                self.maze.buf[(delta.id.row * self.maze.cols + delta.id.col) as usize] =
                    delta.before;
            }
            return self.build_tape.set_prev();
        }
        false
    }

    fn solve_step(&mut self) -> bool {
        if let Some(history) = self.solve_tape.cur_step() {
            if self.forward {
                for delta in history {
                    self.maze.buf[(delta.id.row * self.maze.cols + delta.id.col) as usize] =
                        delta.after;
                }
                return self.solve_tape.set_next();
            }
            for delta in history.iter().rev() {
                self.maze.buf[(delta.id.row * self.maze.cols + delta.id.col) as usize] =
                    delta.before;
            }
            return self.solve_tape.set_prev();
        }
        false
    }

    fn build_delta(&mut self) -> bool {
        if self.pause {
            return true;
        }
        if self.forward {
            if !self.build_step() {
                return false;
            }
        } else {
            self.forward = !self.build_step();
        }
        true
    }

    fn solve_delta(&mut self) -> bool {
        if self.pause {
            return true;
        }
        if self.forward {
            if !self.solve_step() {
                self.pause = true;
            }
        } else if !self.solve_step() {
            return false;
        }
        true
    }
}
