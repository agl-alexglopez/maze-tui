use crate::tables;
use crate::tui;
use builders::build;
use crossbeam_channel::bounded;
use crossterm::event::KeyCode;
use maze;
use print;
use rand::{distributions::Bernoulli, distributions::Distribution, seq::SliceRandom, thread_rng};
use solvers::solve;
use std::error;
use std::fmt;
use std::thread;

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
    match set_command_args(tui, &cmd) {
        Ok(run) => {
            run_channels(run, tui)?;
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
                    build::flush_grid(&lk.maze);
                    if let Some((static_mod, _)) = this_run.modify {
                        static_mod(&mut lk.maze);
                    }
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
    if !should_quit {
        tui.info_prompt()?;
        let mut scroll = tui::Scroller::default();
        let mut info_popup = false;
        let description = load_desc(&this_run.build);
        'looking_at_maze: loop {
            if info_popup {
                tui.info_popup(&mut scroll, &description)?;
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
                            info_popup = false;
                        } else {
                            info_popup = true;
                        }
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
                    _ => break 'looking_at_maze,
                },
                tui::Pack::Resize(_, _) => break 'looking_at_maze,
                _ => {}
            }
        }
    }
    Ok(())
}

fn set_command_args(tui: &mut tui::Tui, cmd: &String) -> Result<tables::MazeRunner, Quit> {
    let mut run = tables::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
    run.args.odd_rows = (dimensions.rows as f64 / 1.2) as i32;
    run.args.odd_cols = dimensions.cols;
    run.args.offset = dimensions.offset;
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in cmd.split_whitespace().into_iter() {
        if process_current {
            match set_arg(
                &mut run,
                &tables::FlagArg {
                    flag: prev_flag,
                    arg: &a,
                },
            ) {
                Ok(_) => {}
                Err(msg) => {
                    tui.error_popup(msg).expect("Tui error");
                    return Err(Quit::new());
                }
            }
            process_current = false;
            continue;
        }
        match tables::search_table(&a, &tables::FLAGS) {
            Some(flag) => {
                process_current = true;
                prev_flag = flag;
            }
            None => {
                tui.error_popup(format!("Unknown Flag[{}]\nPress any key to continue.", &a))
                    .expect("Tui error");
                return Err(Quit::new());
            }
        }
    }
    if process_current {
        tui.error_popup(format!(
            "Flag[{}] with missing Arg[?]\nPress any key to continue.",
            &prev_flag
        ))
        .expect("Tui error");
        return Err(Quit::new());
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
    this_run
}

pub fn err_string(args: &tables::FlagArg) -> String {
    String::from(format!(
        "Invalid Flag[{}] Arg[{}] combo.\nPress any key to continue.",
        args.flag, args.arg
    ))
}

pub fn load_desc(cur_builder: &tables::BuildFunction) -> &'static str {
    match DESCRIPTIONS.iter().find(|(func, _)| func == cur_builder) {
        Some(&(_, desc)) => &desc,
        None => "Coming Soon!",
    }
}

pub static DESCRIPTIONS: [(tables::BuildFunction, &'static str); 9] = [
    (
        (
            builders::arena::generate_maze,
            builders::arena::animate_maze,
        ),
        include_str!("../../res/arena.txt"),
    ),
    (
        (
            builders::eller::generate_maze,
            builders::eller::animate_maze,
        ),
        include_str!("../../res/eller.txt"),
    ),
    (
        (builders::grid::generate_maze, builders::grid::animate_maze),
        include_str!("../../res/grid.txt"),
    ),
    (
        (
            builders::kruskal::generate_maze,
            builders::kruskal::animate_maze,
        ),
        include_str!("../../res/kruskal.txt"),
    ),
    (
        (builders::prim::generate_maze, builders::prim::animate_maze),
        include_str!("../../res/prim.txt"),
    ),
    (
        (
            builders::recursive_backtracker::generate_maze,
            builders::recursive_backtracker::animate_maze,
        ),
        include_str!("../../res/recursive_backtracker.txt"),
    ),
    (
        (
            builders::recursive_subdivision::generate_maze,
            builders::recursive_subdivision::animate_maze,
        ),
        include_str!("../../res/recursive_subdivision.txt"),
    ),
    (
        (
            builders::wilson_adder::generate_maze,
            builders::wilson_adder::animate_maze,
        ),
        include_str!("../../res/wilson_adder.txt"),
    ),
    (
        (
            builders::wilson_carver::generate_maze,
            builders::wilson_carver::animate_maze,
        ),
        include_str!("../../res/wilson_carver.txt"),
    ),
];
