use crate::build;
use maze;
use speed;

pub fn generate_maze(maze: &mut maze::Maze) {
    build::fill_maze_with_walls(maze);
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            build::build_path(maze, maze::Point { row: r, col: c });
        }
    }
}

pub fn animate_maze(maze: &mut maze::Maze, speed: speed::Speed) {
    let animation = build::BUILDER_SPEEDS[speed as usize];
    build::fill_maze_with_walls(maze);
    build::clear_and_flush_grid(maze);
    for r in 1..maze.row_size() - 1 {
        for c in 1..maze.col_size() - 1 {
            build::carve_path_walls_animated(maze, maze::Point { row: r, col: c }, animation);
        }
    }
}