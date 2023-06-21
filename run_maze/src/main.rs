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
    let args = maze::MazeArgs {
        odd_rows: 33,
        odd_cols: 111,
        style: maze::MazeStyle::Round,
    };
    let mut boxed_maze = maze::Maze::new(args);
    recursive_backtracker::animate_recursive_backtracker_maze(
        &mut boxed_maze,
        maze_util::BuilderSpeed::Speed4,
    );
    print_util::set_cursor_position(maze::Point {
        row: boxed_maze.row_size(),
        col: boxed_maze.col_size(),
    });
    dfs::animate_with_dfs_thread_hunt(boxed_maze, solve_util::SolverSpeed::Speed4);
}
