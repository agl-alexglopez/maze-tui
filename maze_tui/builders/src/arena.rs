use crate::build;
use maze;

// Pure data driven algorithm with no display.

pub fn generate_maze(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("uncontested lock failure"),
    };
    build::fill_maze_with_walls(&mut lk.maze);
    for r in 1..lk.maze.rows() - 1 {
        for c in 1..lk.maze.cols() - 1 {
            build::build_path(&mut lk.maze, maze::Point { row: r, col: c });
        }
    }
}

// History tracked for later playback and animation.

pub fn generate_history(monitor: monitor::MazeMonitor) {
    let mut lk = match monitor.lock() {
        Ok(l) => l,
        Err(_) => print::maze_panic!("builder could not take lock"),
    };
    build::fill_maze_history_with_walls(&mut lk.maze);
    for r in 1..lk.maze.rows() - 1 {
        for c in 1..lk.maze.cols() - 1 {
            build::build_path_history(&mut lk.maze, maze::Point { row: r, col: c });
        }
    }
}
