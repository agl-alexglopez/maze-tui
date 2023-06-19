mod builders;
mod solvers;
mod utilities;

pub use crate::builders::recursive_backtracker;
// pub use crate::solvers::dfs;
pub use crate::utilities::maze;
pub use crate::utilities::util;

fn main() {
    let args: maze::MazeArgs = Default::default();
    let mut build_maze_test = maze::Maze::new(args);
    recursive_backtracker::animate_recursive_backtracker_maze(&mut build_maze_test, util::BuilderSpeed::Speed4);
    util::set_cursor_position( maze::Point {row: build_maze_test.row_size(), col: build_maze_test.col_size()});
    println!();
}
