mod builders;
mod solvers;
mod utilities;

use builders::recursive_backtracker;
use solvers::dfs_threads;
use utilities::maze;

fn main() {
    println!("Hello, from main.");
    recursive_backtracker::print_from_builder();
    dfs_threads::print_success();
    let args: maze::MazeArgs = Default::default();
    let build_maze_test = maze::Maze::new(args);
    println!(
        "This maze has {} rows and {} cols.",
        build_maze_test.row_size(),
        build_maze_test.col_size(),
    );
    println!("Here are the building blocks selected as your walls.");
    for piece in build_maze_test.wall_style() {
        print!("{} ", piece);
    }
    println!();
    println!("Let's check if ANSI escapes are working.");
    println!(
        "{}, {}, {}, {}",
        maze::FROM_NORTH_MARK,
        maze::FROM_EAST_MARK,
        maze::FROM_SOUTH_MARK,
        maze::FROM_WEST_MARK
    );
}
