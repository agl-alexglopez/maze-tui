use crate::maze;
use std::{thread, time};
use std::io::{stdout, Write};
use crossterm::{terminal, ExecutableCommand, QueueableCommand, cursor};
type SpeedUnit = u64;

enum BuilderSpeed {
    Instant = 0,
    Speed1,
    Speed2,
    Speed3,
    Speed4,
    Speed5,
    Speed6,
    Speed7,
}

enum ParityPoint {
    Even,
    Odd
}

const BUILDER_SPEEDS: [SpeedUnit; 8] = [0, 5000, 2500, 1000, 500, 250, 100, 1];

pub fn add_positive_slope(maze: &mut maze::Maze, p: maze::Point) {
    let row_size = maze.row_size() as f32 - 2.0f32;
    let col_size = maze.col_size() as f32 - 2.0f32;
    let cur_row = p.row as f32;
    let slope = (2.0f32 - row_size) / (2.0f32 - col_size);
    let b = 2.0f32 - (2.0f32 * slope);
    let on_slope = ( (cur_row - b) / slope ) as i32;
    if p.col == on_slope && p.col < maze.col_size() - 2 && p.col > 1 {

    }
}

pub fn carve_path_walls(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    if p.row - 1 >= 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        maze[u_row][u_col + 1] &= !maze::WEST_WALL;
    }
    maze[u_row][u_col] |= maze::BUILDER_BIT;
}

pub fn carve_path_walls_animate(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_millis(speed));
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row - 1, col: p.col});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row + 1, col: p.col});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row, col: p.col - 1});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.col + 1 < maze.col_size() && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col + 1] &= !maze::WEST_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row, col: p.col + 1});
        thread::sleep(time::Duration::from_millis(speed));
    }
    maze[u_row][u_col] |= maze::BUILDER_BIT;
}

pub fn build_wall(maze: &mut maze::Maze, p: maze::Point) {
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 {
        wall |= maze::NORTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        wall |= maze::SOUTH_WALL;
    }
    if p.col - 1 >= 0 {
        wall |= maze::WEST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        wall |= maze::EAST_WALL;
    }
    maze[p.row as usize][p.col as usize] |= wall;
    maze[p.row as usize][p.col as usize] &= !maze::PATH_BIT;
}

pub fn build_wall_carefully(maze: &mut maze::Maze, p: maze::Point) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    let mut wall: maze::WallLine = 0b0;
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        wall |= maze::NORTH_WALL;
        maze[u_row - 1][u_col] |= maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() && ( maze[u_row + 1][u_col] & maze::PATH_BIT ) == 0 {
        wall |= maze::SOUTH_WALL;
        maze[u_row + 1][u_col] |= maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 && ( maze[u_row][u_col - 1] & maze::PATH_BIT ) == 0 {
        wall |= maze::WEST_WALL;
        maze[u_row][u_col - 1] |= maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() && ( maze[u_row][u_col + 1] & maze::PATH_BIT ) == 0 {
        wall |= maze::EAST_WALL;
        maze[u_row][u_col + 1] |= maze::WEST_WALL;
    }
    maze[u_row][u_col] |= wall;
    maze[u_row][u_col] &= !maze::PATH_BIT;
}

pub fn build_path(maze: &mut maze::Maze, p: maze::Point) {
    if p.row - 1 >= 0 {
        maze[(p.row - 1) as usize][p.col as usize] &= !maze::SOUTH_WALL;
    }
    if p.row + 1 < maze.row_size() {
        maze[(p.row + 1) as usize][p.col as usize] &= !maze::NORTH_WALL;
    }
    if p.col - 1 >= 0 {
        maze[p.row as usize][( p.col - 1) as usize] &= !maze::EAST_WALL;
    }
    if p.col + 1 < maze.col_size() {
        maze[p.row as usize][( p.col + 1) as usize] &= !maze::WEST_WALL;
    }
    maze[p.row as usize][p.col as usize] |= maze::PATH_BIT;
}

pub fn build_path_animated(maze: &mut maze::Maze, p: maze::Point, speed: SpeedUnit) {
    let u_row = p.row as usize;
    let u_col = p.col as usize;
    maze[u_row][u_col] |= maze::PATH_BIT;
    flush_cursor_maze_coordinate(maze, p);
    thread::sleep(time::Duration::from_millis(speed));
    if p.row - 1 >= 0 && (maze[u_row - 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row - 1][u_col] &= !maze::SOUTH_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row - 1, col: p.col});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.row + 1 < maze.row_size() && (maze[u_row + 1][u_col] & maze::PATH_BIT) == 0 {
        maze[u_row + 1][u_col] &= !maze::NORTH_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row + 1, col: p.col});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.col - 1 >= 0 && (maze[u_row][u_col - 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col - 1] &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row, col: p.col - 1});
        thread::sleep(time::Duration::from_millis(speed));
    }
    if p.col + 1 >= 0 && (maze[u_row][u_col + 1] & maze::PATH_BIT) == 0 {
        maze[u_row][u_col + 1] &= !maze::EAST_WALL;
        flush_cursor_maze_coordinate(maze, maze::Point {row: p.row, col: p.col + 1});
        thread::sleep(time::Duration::from_millis(speed));
    }
}

// Terminal Printing Helpers

pub fn flush_cursor_maze_coordinate(maze: &maze::Maze, p: maze::Point) {
    print_square(maze, p);
    stdout().flush().unwrap();
}

pub fn print_maze_square(maze: &maze::Maze, p: maze::Point) {
    let square = &maze[p.row as usize][p.col as usize];
    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo((p.row + 1) as u16, (p.col + 1) as u16)).unwrap();
    if square & maze::PATH_BIT == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
    } else if square & maze::PATH_BIT != 0 {
        print!(" ");
    } else {
        panic!("Maze square has no category");
    }
}

pub fn print_square(maze: &maze::Maze, p: maze::Point) {
    let square = &maze[p.row as usize][p.col as usize];
    stdout().queue(cursor::MoveTo((p.row + 1) as u16, (p.col + 1) as u16)).unwrap();
    if square & maze::MARKERS_MASK != 0 {
        let mark = (square & maze::MARKERS_MASK) >> maze::MARKER_SHIFT;
        print!("{}", maze::BACKTRACKING_SYMBOLS[mark as usize]);
    } else if square & maze::MARKERS_MASK == 0 {
        print!("{}", maze.wall_style()[(square & maze::WALL_MASK) as usize]);
    } else if square & maze::PATH_BIT != 0 {
        print!(" ");
    } else {
        panic!("Printed a maze square without a category.");
    }
}

pub fn clear_and_flush_grid(maze: &maze::Maze) {
    stdout().queue(terminal::Clear(terminal::ClearType::All)).unwrap();
    for row in 0..maze.row_size() {
        for col in 0..maze.col_size() {
            print_square(maze, maze::Point {row, col});
        }
        print!("\n");
    }
    stdout().flush().unwrap();
}

pub fn print_maze(maze: &maze::Maze) {
    for row in 0..maze.row_size() {
        for col in 0..maze.col_size() {
            print_square(maze, maze::Point {row, col});
        }
        print!("\n");
    }
}
