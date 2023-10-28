use crate::tui;
use builders::build;
use crossbeam_channel::bounded;
use crossbeam_channel::{select, tick};
use crossterm::event::KeyCode;
use rand::{
    distributions::Bernoulli, distributions::Distribution, seq::SliceRandom, thread_rng, Rng,
};
use solvers::solve;
use std::error;
use std::fmt;
use std::{thread, time::Duration};

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

pub fn run_command(cmd: &String, tui: &mut tui::Tui) -> tui::Result<()> {
    if cmd.is_empty() {
        rand_with_channels(tui)?;
        return Ok(());
    }
    match set_command_args(tui, cmd) {
        Ok(run) => {
            run_channels(run, tui)?;
        }
        Err(_) => return Err(Box::new(Quit::new())),
    };
    Ok(())
}

pub fn render_command(cmd: &String, tui: &mut tui::Tui) -> tui::Result<()> {
    if cmd.is_empty() {
        //rand_with_channels(tui)?;
        render_maze(set_command_args(tui, cmd).unwrap(), tui)?;
        return Ok(());
    }
    match set_command_args(tui, cmd) {
        Ok(run) => {
            render_maze(run, tui)?;
        }
        Err(_) => return Err(Box::new(Quit::new())),
    };
    Ok(())
}

pub fn rand_with_channels(tui: &mut tui::Tui) -> tui::Result<()> {
    run_channels(set_random_args(tui), tui)?;
    Ok(())
}

fn run_channels(this_run: tables::MazeRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    tui.terminal.clear()?;
    let (impatient_user, worker) = bounded::<bool>(1);
    let (finished_worker, patient_user) = bounded::<bool>(1);
    let mut should_quit = false;
    let maze = solve::Solver::new(maze::Maze::new_channel(&this_run.args, worker));
    let mc = maze.clone();
    let worker_thread = thread::spawn(move || {
        if let Ok(mut lk) = mc.lock() {
            match this_run.build_view {
                tables::ViewingMode::StaticImage => {
                    build::print_overlap_key(&lk.maze);
                    this_run.build.0(&mut lk.maze);
                    if let Some((static_mod, _)) = this_run.modify {
                        static_mod(&mut lk.maze);
                    }
                    build::flush_grid(&lk.maze);
                }
                tables::ViewingMode::AnimatedPlayback => {
                    this_run.build.1(&mut lk.maze, this_run.build_speed);
                    if let Some((_, animated_mod)) = this_run.modify {
                        animated_mod(&mut lk.maze, this_run.build_speed);
                    }
                }
            }
        }
        match this_run.solve_view {
            tables::ViewingMode::StaticImage => this_run.solve.0(mc),
            tables::ViewingMode::AnimatedPlayback => this_run.solve.1(mc, this_run.solve_speed),
        }
        match finished_worker.send(true) {
            Ok(_) => {}
            Err(_) => print::maze_panic!("Worker sender disconnected."),
        }
    });

    while patient_user.is_empty() {
        match tui.events.next()? {
            tui::Pack::Press(_) | tui::Pack::Resize(_, _) => match impatient_user.send(true) {
                Ok(_) => {
                    should_quit = true;
                    break;
                }
                Err(_) => return Err(Box::new(Quit::new())),
            },
            _ => {}
        }
    }
    worker_thread.join().unwrap();
    if should_quit {
        return Ok(());
    }
    handle_waiting_user(&this_run.build, maze.clone(), tui)
}

fn render_maze(this_run: tables::MazeRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    tui.terminal.clear()?;
    //let t_start = Instant::now();
    let render_space = tui.inner_maze_rect();
    let ticker = tick(Duration::from_millis(10));
    let maze = solve::Solver::new(maze::Maze::new(this_run.args));
    let mc = maze.clone();
    if let Ok(mut lk) = mc.lock() {
        build::fill_maze_with_walls(&mut lk.maze);
        let mut gen = thread_rng();
        let start: maze::Point = maze::Point {
            row: 2 * (gen.gen_range(1..lk.maze.row_size() - 2) / 2) + 1,
            col: 2 * (gen.gen_range(1..lk.maze.col_size() - 2) / 2) + 1,
        };
        let mut cur = start;
        'building: loop {
            select! {
                recv(ticker) -> _ => {
                    cur
                    = builders::recursive_backtracker::generate_delta(&mut lk.maze, cur);
                    tui.render_builder_frame(&lk.maze, render_space)?;
                    if cur == start {
                        break 'building;
                    }
                }
            }
        }
        // if let Some((_, animated_mod)) = this_run.modify {
        //     animated_mod(&mut lk.maze, this_run.build_speed);
        // }
        // this_run.solve.1(maze.clone(), this_run.solve_speed);
    }
    handle_waiting_user(&this_run.build, maze.clone(), tui)
}

fn handle_waiting_user(
    builder: &tables::BuildFunction,
    maze: solve::SolverMonitor,
    tui: &mut tui::Tui,
) -> tui::Result<()> {
    tui.info_prompt()?;
    let mut scroll = tui::Scroller::default();
    let mut info_popup = false;
    let description = tables::load_desc(builder);
    'looking_at_maze: loop {
        if info_popup {
            tui.info_popup(&mut scroll, description)?;
        }
        match tui.events.next()? {
            tui::Pack::Press(ke) => match ke.code {
                KeyCode::Char('i') => {
                    if info_popup {
                        if let Ok(lk) = maze.lock() {
                            tui.terminal.clear()?;
                            solve::print_paths(&lk.maze);
                            build::print_overlap_key(&lk.maze);
                            tui.info_prompt()?;
                        }
                    }
                    info_popup = !info_popup;
                }
                KeyCode::Down => {
                    if info_popup {
                        scroll.scroll(ratatui::widgets::ScrollDirection::Forward);
                    }
                }
                KeyCode::Up => {
                    if info_popup {
                        scroll.scroll(ratatui::widgets::ScrollDirection::Backward);
                    }
                }
                KeyCode::Esc => break 'looking_at_maze,
                _ => {}
            },
            tui::Pack::Resize(_, _) => break 'looking_at_maze,
            _ => {}
        }
    }
    Ok(())
}

pub fn set_command_args(tui: &mut tui::Tui, cmd: &str) -> Result<tables::MazeRunner, Quit> {
    let mut run = tables::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
    run.args.odd_rows = (dimensions.rows as f64 / 1.2) as i32;
    run.args.odd_cols = dimensions.cols;
    run.args.offset = dimensions.offset;
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in cmd.split_whitespace() {
        if process_current {
            match set_arg(
                &mut run,
                &tables::FlagArg {
                    flag: prev_flag,
                    arg: a,
                },
            ) {
                Ok(_) => {}
                Err(msg) => {
                    tui.error_popup(format!(
                        "{}\n{}\npress any key to continue",
                        msg,
                        get_arg_section(prev_flag)
                    ))
                    .expect("Tui error");
                    return Err(Quit::new());
                }
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
                tui.error_popup(format!(
                    "unknown flag[{}].\n{}\npress any key to continue",
                    a, VALID_FLAGS
                ))
                .expect("Tui error");
                return Err(Quit::new());
            }
        }
    }
    if process_current {
        tui.error_popup(format!(
            "flag[{}] with missing arg[?]\n{}\npress any key to continue",
            prev_flag,
            get_arg_section(prev_flag)
        ))
        .expect("Tui error");
        return Err(Quit::new());
    }
    if run.args.style == maze::MazeStyle::Mini {
        run.args.odd_rows *= 2;
    }
    Ok(run)
}

fn set_arg(run: &mut tables::MazeRunner, args: &tables::FlagArg) -> Result<(), String> {
    match args.flag {
        "-b" => tables::search_table(args.arg, &tables::BUILDERS)
            .map(|func_pair| run.build = func_pair)
            .ok_or(err_string(args)),
        "-m" => tables::search_table(args.arg, &tables::MODIFICATIONS)
            .map(|mod_tuple| run.modify = Some(mod_tuple))
            .ok_or(err_string(args)),
        "-s" => tables::search_table(args.arg, &tables::SOLVERS)
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

fn set_random_args(tui: &mut tui::Tui) -> tables::MazeRunner {
    let mut rng = thread_rng();
    let mut this_run = tables::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
    this_run.build_view = tables::ViewingMode::AnimatedPlayback;
    this_run.solve_view = tables::ViewingMode::AnimatedPlayback;
    this_run.args.odd_rows = (dimensions.rows as f64 / 1.3) as i32;
    this_run.args.odd_cols = dimensions.cols;
    this_run.args.offset = dimensions.offset;
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
    this_run.build = match tables::BUILDERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Build algorithm array empty."),
    };
    this_run.solve = match tables::SOLVERS.choose(&mut rng) {
        Some(&algo) => algo.1,
        None => print::maze_panic!("Solve algorithm array empty."),
    };
    this_run.modify = None;
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        this_run.modify = match tables::MODIFICATIONS.choose(&mut rng) {
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
