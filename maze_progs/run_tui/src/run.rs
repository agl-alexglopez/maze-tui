use crate::tui;
use builders::build;
use crossterm::event::KeyCode;
use rand::{distributions::Bernoulli, distributions::Distribution, seq::SliceRandom, thread_rng};
use ratatui::{
    prelude::{CrosstermBackend, Rect, Terminal},
    widgets::ScrollDirection,
};
use std::{error, fmt, rc::Rc, sync::Arc, sync::Mutex, time::Duration, time::Instant};
use tui_textarea::{Input, Key};

const DEFAULT_DURATION: Duration = Duration::from_micros(2000);
const HOME_DURATION: Duration = Duration::from_millis(88);
const MAX_DURATION: Duration = Duration::from_secs(5);
const MIN_DURATION: Duration = Duration::from_micros(1);

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

#[derive(Debug, Clone)]
struct Playback {
    maze: maze::Blueprint,
    build_tape: tape::Tape<maze::Point, maze::Square>,
    solve_tape: tape::Tape<maze::Point, maze::Square>,
    forward: bool,
    pause: bool,
    speed: Duration,
    last_render: Instant,
}

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
}

pub fn run() -> tui::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = tui::EventHandler::new(250);
    let mut tui = tui::Tui::new(terminal, events);
    tui.enter()?;
    let mut play = new_home_tape(tui.padded_frame());
    'render: loop {
        tui.home_animated(tui::SolveFrame { maze: &play.maze })?;
        if let Some(ev) = tui.events.try_next() {
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
                        Err(msg) => {
                            tui.error_popup(msg, tui::SolveFrame { maze: &play.maze })?;
                        }
                    },
                    input => {
                        tui.cmd_input(input);
                    }
                },
            }
        }
        let now = Instant::now();
        if now - play.last_render >= play.speed {
            play.forward = if play.forward {
                play.solve_step()
            } else {
                !play.solve_step()
            };
            play.last_render = now;
        }
    }
    tui.exit()?;
    Ok(())
}

fn render_maze(mut this_run: tables::MazeRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    let render_space = tui.inner_maze_rect();
    this_run.args.style = maze::MazeStyle::Contrast;
    let mut play = new_tape(&this_run);
    'rendering: loop {
        'building: loop {
            tui.render_maze_frame(
                tui::BuildFrame {
                    maze: &mut play.maze,
                },
                &render_space,
            )?;
            if let Some(ev) = tui.events.try_next() {
                if !handle_press(
                    tui,
                    ev,
                    tui::Process::Building,
                    &this_run,
                    &mut play,
                    &render_space,
                ) {
                    break 'rendering;
                }
            }
            let now = Instant::now();
            if !play.pause && now - play.last_render >= play.speed {
                if play.forward {
                    if !play.build_step() {
                        break 'building;
                    }
                } else {
                    play.forward = !play.build_step();
                }
                play.last_render = now;
            }
        }
        'solving: loop {
            tui.render_maze_frame(tui::SolveFrame { maze: &play.maze }, &render_space)?;
            if let Some(ev) = tui.events.try_next() {
                if !handle_press(
                    tui,
                    ev,
                    tui::Process::Solving,
                    &this_run,
                    &mut play,
                    &render_space,
                ) {
                    break 'rendering;
                }
            }
            let now = Instant::now();
            if !play.pause && now - play.last_render >= play.speed {
                if play.forward {
                    play.forward = play.solve_step();
                } else if !play.solve_step() {
                    break 'solving;
                }
                play.last_render = now;
            }
        }
    }
    Ok(())
}

fn handle_press(
    tui: &mut tui::Tui,
    ev: tui::Pack,
    process: tui::Process,
    args: &tables::MazeRunner,
    play: &mut Playback,
    render_space: &Rc<[Rect]>,
) -> bool {
    match ev {
        tui::Pack::Press(ev) => match ev.code {
            KeyCode::Char('i') => {
                let description = match process {
                    tui::Process::Building => tables::load_desc(&args.build),
                    tui::Process::Solving => tables::load_desc(&args.solve),
                };
                if handle_reader(tui, process, description, &play.maze, render_space).is_err() {
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
            KeyCode::Down => {
                play.speed = std::cmp::min(play.speed.saturating_mul(2), MAX_DURATION);
            }
            KeyCode::Up => {
                play.speed = match play.speed.checked_div(2) {
                    Some(t) => t,
                    None => Duration::ZERO,
                };
                play.speed = std::cmp::max(play.speed, MIN_DURATION);
            }
            KeyCode::Esc => return false,
            _ => return true,
        },
        tui::Pack::Resize(_, _) => return false,
    }
    true
}

fn new_tape(run: &tables::MazeRunner) -> Playback {
    let monitor = monitor::Solver::new(maze::Maze::new(run.args));
    builders::recursive_backtracker::generate_history(monitor.clone());
    solvers::bfs::hunt_history(monitor.clone());
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
                    speed: DEFAULT_DURATION,
                    last_render: Instant::now(),
                }
            }
            Err(_) => print::maze_panic!("rendering cannot progress without lock"),
        },
        None => print::maze_panic!("rendering cannot progress without lock"),
    }
}

fn new_home_tape(rect: Rect) -> Playback {
    let mut run_bg = set_random_args(&rect);
    run_bg.args.style = maze::MazeStyle::Contrast;
    let bg_maze = monitor::Solver::new(maze::Maze::new(run_bg.args));
    builders::recursive_backtracker::generate_history(bg_maze.clone());
    solvers::bfs::hunt_history(bg_maze.clone());
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
                    speed: HOME_DURATION,
                    last_render: Instant::now(),
                }
            }
            Err(_) => print::maze_panic!("rendering cannot progress without lock"),
        },
        None => print::maze_panic!("rendering cannot progress without lock"),
    }
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
        tui.info_popup(process, render_space, maze, &mut scroll, description)?;
        if let Some(tui::Pack::Press(k)) = tui.events.try_next() {
            match k.code {
                KeyCode::Char('i') => break 'reading,
                KeyCode::Down => scroll.scroll(ScrollDirection::Forward),
                KeyCode::Up => scroll.scroll(ScrollDirection::Backward),
                KeyCode::Esc => return Err(Box::new(Quit::new())),
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn set_command_args(cmd: String, tui: &mut tui::Tui) -> Result<tables::MazeRunner, String> {
    let mut run = tables::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
    run.args.odd_rows = (dimensions.rows as f64 / 1.2) as i32;
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

fn set_arg(run: &mut tables::MazeRunner, args: &tables::FlagArg) -> Result<(), String> {
    match args.flag {
        "-b" => tables::search_table(args.arg, &tables::CURSOR_BUILDERS)
            .map(|func_pair| run.build = func_pair)
            .ok_or(err_string(args)),
        "-m" => tables::search_table(args.arg, &tables::CURSOR_MODIFICATIONS)
            .map(|mod_tuple| run.modify = Some(mod_tuple))
            .ok_or(err_string(args)),
        "-s" => tables::search_table(args.arg, &tables::CURSOR_SOLVERS)
            .map(|solve_tuple| run.solve = solve_tuple)
            .ok_or(err_string(args)),
        "-w" => tables::search_table(args.arg, &tables::WALL_STYLES)
            .map(|wall_style| run.args.style = wall_style)
            .ok_or(err_string(args)),
        "-ba" => match tables::search_table(args.arg, &tables::SPEEDS) {
            Some(speed) => {
                run.build_speed = speed;
                run.build_view = tables::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        "-sa" => match tables::search_table(args.arg, &tables::SPEEDS) {
            Some(speed) => {
                run.solve_speed = speed;
                run.solve_view = tables::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        _ => Err(err_string(args)),
    }
}

fn set_random_args(rect: &Rect) -> tables::MazeRunner {
    let mut rng = thread_rng();
    let mut this_run = tables::MazeRunner::new();
    this_run.build_view = tables::ViewingMode::AnimatedPlayback;
    this_run.solve_view = tables::ViewingMode::AnimatedPlayback;
    this_run.args.odd_rows = (rect.height - 1) as i32;
    this_run.args.odd_cols = (rect.width - 1) as i32;
    this_run.args.offset = maze::Offset {
        add_rows: rect.y as i32,
        add_cols: rect.x as i32,
    };
    let modification_probability = Bernoulli::new(0.2);
    this_run.args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not set for loop, broken"),
    };
    this_run.build_speed = match tables::SPEEDS.choose(&mut rng) {
        Some(&speed) => speed.1,
        None => print::maze_panic!("Build speed array empty."),
    };
    this_run.solve_speed = match tables::SPEEDS.choose(&mut rng) {
        Some(&speed) => speed.1,
        None => print::maze_panic!("Solve speed array empty."),
    };
    this_run.build = match tables::CURSOR_BUILDERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Build algorithm array empty."),
    };
    this_run.solve = match tables::CURSOR_SOLVERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Solve algorithm array empty."),
    };
    this_run.modify = None;
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        this_run.modify = match tables::CURSOR_MODIFICATIONS.choose(&mut rng) {
            Some(&m) => Some(m.1),
            None => print::maze_panic!("Modification table empty."),
        }
    }
    if this_run.args.style == maze::MazeStyle::Mini {
        this_run.args.odd_rows *= 2;
    }
    this_run
}

pub fn err_string(args: &tables::FlagArg) -> String {
    format!("invalid flag[{}] arg[{}] combo", args.flag, args.arg)
}

fn get_arg_section(flag: &str) -> &'static str {
    VALID_ARGS
        .iter()
        .find(|(f, _)| *f == flag)
        .expect("check VALID_ARGS table.")
        .1
}

pub static VALID_FLAGS: &str = "VALID FLAGS:[-b][-ba][-s][-sa][-w][-m]";
pub static VALID_ARGS: [(&str, &str); 6] = [
    ("-b", "see BUILDER FLAG section"),
    ("-m", "see MODIFICATION FLAG section"),
    ("-w", "see WALL FLAG section"),
    ("-s", "see SOLVER FLAG section"),
    ("-sa", "see SOLVER ANIMATION section"),
    ("-ba", "see BUILDER ANIMATION section"),
];
