mod builders;
mod solvers;
mod utilities;

pub use crate::builders::recursive_backtracker;
pub use crate::solvers::dfs;
pub use crate::utilities::maze;
pub use crate::utilities::maze_util;
pub use crate::utilities::print_util;
pub use crate::utilities::solve_util;


fn main() {
    let args = maze::MazeArgs {odd_rows: 33, odd_cols: 111, style: maze::MazeStyle::Contrast};
    let mut build_maze_test = Box::new(maze::Maze::new(args));
    recursive_backtracker::animate_recursive_backtracker_maze(
        &mut build_maze_test,
        maze_util::BuilderSpeed::Speed4,
    );
    print_util::set_cursor_position(maze::Point {
        row: build_maze_test.row_size(),
        col: build_maze_test.col_size(),
    });
    dfs::animate_with_dfs_thread_hunt(build_maze_test, solve_util::SolverSpeed::Speed4);
}
