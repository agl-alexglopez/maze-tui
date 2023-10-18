use crate::args;
use crate::tables;
use crate::tui;
use crossbeam_channel::bounded;
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
        Err(_) => {}
    };
    Ok(())
}

pub fn rand_with_channels(tui: &mut tui::Tui) -> tui::Result<()> {
    run_channels(set_random_args(tui), tui)?;
    Ok(())
}

fn run_channels(this_run: args::MazeRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    tui.terminal.clear()?;
    let (impatient_user, worker) = bounded::<bool>(1);
    let (finished_worker, patient_user) = bounded::<bool>(1);
    let mut should_quit = false;
    let worker_thread = thread::spawn(move || {
        let mut maze = maze::Maze::new_channel(&this_run.args, worker);
        this_run.build.1(&mut maze, this_run.build_speed);
        match this_run.modify {
            Some(m) => m.1(&mut maze, this_run.build_speed),
            None => {}
        }
        this_run.solve.1(solve::Solver::new(maze), this_run.solve_speed);
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
        'looking_at_maze: loop {
            match tui.events.next()? {
                tui::Pack::Press(_) | tui::Pack::Resize(_, _) => {
                    break 'looking_at_maze;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn set_command_args(tui: &mut tui::Tui, cmd: &String) -> Result<args::MazeRunner, Quit> {
    let mut run = args::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
    run.args.odd_rows = (dimensions.rows as f64 / 1.3) as i32;
    run.args.odd_cols = dimensions.cols;
    run.args.offset = dimensions.offset;
    let mut prev_flag: &str = "";
    let mut process_current = false;
    for a in cmd.split_whitespace().into_iter() {
        if process_current {
            match set_arg(
                &mut run,
                &args::FlagArg {
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

fn set_arg(run: &mut args::MazeRunner, args: &args::FlagArg) -> Result<(), String> {
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
                run.build_view = args::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        "-sa" => match tables::search_table(args.arg, &tables::SPEEDS) {
            Some(speed) => {
                run.solve_speed = speed;
                run.solve_view = args::ViewingMode::AnimatedPlayback;
                Ok(())
            }
            None => Err(err_string(args)),
        },
        _ => Err(err_string(args)),
    }
}

fn set_random_args(tui: &mut tui::Tui) -> args::MazeRunner {
    let mut rng = thread_rng();
    let mut this_run = args::MazeRunner::new();
    let dimensions = tui.inner_dimensions();
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

pub fn err_string(args: &args::FlagArg) -> String {
    String::from(format!(
        "Invalid Flag[{}] Arg[{}] combo.\nPress any key to continue.",
        args.flag, args.arg
    ))
}
