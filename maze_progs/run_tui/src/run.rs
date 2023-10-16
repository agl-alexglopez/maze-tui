use crate::args;
use crate::tables;
use crate::tui;
use crossbeam_channel::bounded;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
};
use print;
use rand::{
    distributions::{Bernoulli, Distribution},
    seq::SliceRandom,
    thread_rng,
};
use std::error;
use std::fmt;
use std::{
    sync::{Arc, Mutex},
    thread,
};

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

pub fn rand(mut args: maze::MazeArgs) {
    let mut rng = thread_rng();
    let modification_probability = Bernoulli::new(0.2);
    args.style = match tables::WALL_STYLES.choose(&mut rng) {
        Some(&style) => style.1,
        None => print::maze_panic!("Styles not set for loop, broken"),
    };
    let mut maze = maze::Maze::new(args);
    let build_speed = match tables::SPEEDS.choose(&mut rng) {
        Some(&speed) => speed.1,
        None => print::maze_panic!("Build speed array empty."),
    };
    let solve_speed = match tables::SPEEDS.choose(&mut rng) {
        Some(&speed) => speed.1,
        None => print::maze_panic!("Solve speed array empty."),
    };
    let build_algo = match tables::BUILDERS.choose(&mut rng) {
        Some(&algo) => algo.1 .1,
        None => print::maze_panic!("Build algorithm array empty."),
    };
    let solve_algo = match tables::SOLVERS.choose(&mut rng) {
        Some(&algo) => algo.1 .1,
        None => print::maze_panic!("Solve algorithm array empty."),
    };
    build_algo(&mut maze, build_speed);
    if modification_probability
        .expect("Bernoulli innefective")
        .sample(&mut rng)
    {
        match tables::MODIFICATIONS.choose(&mut rng) {
            Some(modder) => {
                modder.1 .1(&mut maze, build_speed);
            }
            None => print::maze_panic!("Empty modification table."),
        }
    }
    solve_algo(maze, solve_speed);
}

pub fn run_with_channels(this_run: args::MazeRunner, tui: &mut tui::Tui) -> tui::Result<()> {
    let (impatient_user, worker) = bounded::<bool>(1);
    let (finished_worker, patient_user) = bounded::<bool>(1);
    let mut should_quit = false;
    let builder_thread = thread::spawn(move || {
        let mut maze = maze::Maze::new_channel(&this_run.args, worker);
        this_run.build.1(&mut maze, this_run.build_speed);
        this_run.solve.1(maze, this_run.solve_speed);
        match finished_worker.send(true) {
            Ok(_) => {}
            Err(_) => print::maze_panic!("Worker sender disconnected."),
        }
    });

    while patient_user.is_empty() {
        match tui.events.next()? {
            tui::Event::Key(key_event) => match key_event.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    match impatient_user.send(true) {
                        Ok(_) => {
                            should_quit = true;
                            break;
                        }
                        Err(_) => print::maze_panic!("User couldn't exit."),
                    }
                }
                _ => {}
            },
            tui::Event::Resize(_, _) => match impatient_user.send(true) {
                Ok(_) => {
                    should_quit = true;
                    break;
                }
                Err(_) => print::maze_panic!("User couldn't resize."),
            },
            _ => {}
        }
    }
    builder_thread.join().unwrap();
    match should_quit {
        true => Err(Box::new(Quit::new())),
        false => Ok(()),
    }
}
