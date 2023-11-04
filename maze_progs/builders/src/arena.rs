use crate::build;
use maze;
use speed;

pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("builder could not take lock"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    for r in 1..lk.maze.row_size() - 1 {
        for c in 1..lk.maze.col_size() - 1 {
            build::build_path_history(&mut lk.maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn generate_maze(monitor: monitor::MazeReceiver) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    for r in 1..lk.maze.row_size() - 1 {
        for c in 1..lk.maze.col_size() - 1 {
            build::build_path(&mut lk.maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn animate_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    if lk.maze.is_mini() {
        drop(lk);
        animate_mini_maze(monitor, speed);
        return;
    }
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    for r in 1..lk.maze.row_size() - 1 {
        if monitor.exit() {
            return;
        }
        for c in 1..lk.maze.col_size() - 1 {
            build::carve_path_walls_animated(
                &mut lk.maze,
                maze::Point { row: r, col: c },
                animation,
            )
        }
    }
}

fn animate_mini_maze(monitor: monitor::MazeReceiver, speed: speed::Speed) {
    let mut lk = match monitor.solver.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(&mut lk.maze);
    build::flush_grid(&lk.maze);
    build::print_overlap_key_animated(&lk.maze);
    for r in 1..lk.maze.row_size() - 1 {
        if monitor.exit() {
            return;
        }
        for c in 1..lk.maze.col_size() - 1 {
            build::carve_mini_walls_animated(
                &mut lk.maze,
                maze::Point { row: r, col: c },
                animation,
            )
        }
    }
}
