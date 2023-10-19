# Maze TUI

![splash](/images/splash.png)

> **For a complete breakdown of all maze generation algorithms, pseudocode, and many other important details please read the [wiki](https://github.com/agl-alexglopez/multithreading-with-mazes-in-rust/wiki)! This is just the quick start guide to refer to for usage instructions.**

## Quick Start Guide

This project is a command line application that can be run with various combinations of commands. The basic principle behind the commands is that you can ask for any combination of settings, include any settings, exclude any settings, and the program will just work. There are sensible defaults for every flag so experiment with different combinations and tweaks until you get what you are looking for. To start, you should focus mainly on how big you want the maze to be, what algorithm you want to generate it, what algorithm you want to solve it, and if any of these algorithms should be animated in real time. I would reccomend using cargo to build the project.

## Demo

If you would rather just see some cool mazes right away, run the demo I have included. It runs infinite random permutations of maze builder and solver animations so you can see a wide range of what the project has to offer. Stop the loop at any time with `CTRL<C>`.

```zsh
$ cd maze_progs/
$ cargo build --release
$ cargo run --release --bin demo
# Or set the rows and columns to your liking for bigger or smaller demo mazes.
$ cargo run --release --bin demo -- -r 50 -c 50
```

## Run Maze Program

```zsh
$ cd maze_progs/
$ cargo build --release
$ cargo run --release --bin run_maze
```

If you wish to dive into the more specific `run_maze` program, here is the help message that comes with the `-h` flag to get started.

Use flags, followed by arguments, in any order:

- `-r` Rows flag. Set rows for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-c` Columns flag. Set columns for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-b` Builder flag. Set maze building algorithm.
	- `rdfs` - Randomized Depth First Search.
	- `kruskal` - Randomized Kruskal's algorithm.
	- `prim` - Randomized Prim's algorithm.
	- `eller` - Randomized Eller's algorithm.
	- `wilson` - Loop-Erased Random Path Carver.
	- `wilson-walls` - Loop-Erased Random Wall Adder.
	- `fractal` - Randomized recursive subdivision.
	- `grid` - A random grid pattern.
	- `arena` - Open floor with no walls.
- `-m` Modification flag. Add shortcuts to the maze.
	- `cross` - Add crossroads through the center.
	- `x` - Add an x of crossing paths through center.
- `-s` Solver flag. Set maze solving algorithm.
	- `dfs-hunt` - Depth First Search
	- `dfs-gather` - Depth First Search
	- `dfs-corners` - Depth First Search
	- `floodfs-hunt` - Depth First Search
	- `floodfs-gather` - Depth First Search
	- `floodfs-corners` - Depth First Search
	- `rdfs-hunt` - Randomized Depth First Search
	- `rdfs-gather` - Randomized Depth First Search
	- `rdfs-corners` - Randomized Depth First Search
	- `bfs-hunt` - Breadth First Search
	- `bfs-gather` - Breadth First Search
	- `bfs-corners` - Breadth First Search
- `-d` Draw flag. Set the line style for the maze.
	- `sharp` - The default straight lines.
	- `round` - Rounded corners.
	- `doubles` - Sharp double lines.
	- `bold` - Thicker straight lines.
	- `contrast` - Full block width and height walls.
	- `spikes` - Connected lines with spikes.
- `-sa` Solver Animation flag. Watch the maze solution.
	- Any number 1-7. Speed increases with number.
- `-ba` Builder Animation flag. Watch the maze build.
	- Any number 1-7. Speed increases with number.
- `-h` Help flag. Make this prompt appear.

If any flags are omitted, defaults are used.

Examples:

```zsh
cargo run --release --bin run_maze
cargo run --release --bin run_maze -- -r 51 -c 111 -b rdfs -s bfs-hunt
cargo run --release --bin run_maze -- -c 111 -s bfs-gather
cargo run --release --bin run_maze -- -s bfs-corners -d round -b fractal
cargo run --release --bin run_maze -- -s dfs-hunt -ba 4 -sa 5 -b wilson-walls -m x
cargo run --release --bin run_maze -- -h
```

## Maze Measurement Program

This next section is pretty much directly inspired by Jamis Buck's implementation of colorizing his mazes based upon distance from a starting point, most commonly the center. All settings for this section are based on being able to see some aspect of maze quality rated with a color heat map. The program works by painting the maze, starting at a single point, based on some criterion such as distance from that point. This can help us assess the quality of the mazes that we produce. Here are the settings to use the program.

```zsh
$ cd maze_progs/
$ cargo build --release
$ cargo run --release --bin measure
```
Use flags, followed by arguments, in any order:

- `-r` Rows flag. Set rows for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-c` Columns flag. Set columns for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-b` Builder flag. Set maze building algorithm.
	- `rdfs` - Randomized Depth First Search.
	- `kruskal` - Randomized Kruskal's algorithm.
	- `prim` - Randomized Prim's algorithm.
	- `eller` - Randomized Eller's algorithm.
	- `wilson` - Loop-Erased Random Path Carver.
	- `wilson-walls` - Loop-Erased Random Wall Adder.
	- `fractal` - Randomized recursive subdivision.
	- `grid` - A random grid pattern.
	- `arena` - Open floor with no walls.
- `-m` Modification flag. Add shortcuts to the maze.
	- `cross` - Add crossroads through the center.
	- `x` - Add an x of crossing paths through center.
- `-p` Painter flag. Set maze measuring algorithm.
    - `distance` - Distance from the center.
    - `runs` - Run length bias of straight passages.
- `-d` Draw flag. Set the line style for the maze.
	- `sharp` - The default straight lines.
	- `round` - Rounded corners.
	- `doubles` - Sharp double lines.
	- `bold` - Thicker straight lines.
	- `contrast` - Full block width and height walls.
	- `spikes` - Connected lines with spikes.
- `-pa` Painter Animation flag. Watch the maze solution.
	- Any number 1-7. Speed increases with number.
- `-ba` Builder Animation flag. Watch the maze build.
	- Any number 1-7. Speed increases with number.
- `-h` Help flag. Make this prompt appear.

If any flags are omitted, defaults are used.

Examples:

```zsh
cargo run --release --bin measure
cargo run --release --bin measure -- -r 51 -c 111 -b rdfs
cargo run --release --bin measure -- -c 111 -p distance -ba 5 -pa 5
cargo run --release --bin measure -- -h
```

## Wiki

Please read the [wiki](https://github.com/agl-alexglopez/multithreading-with-mazes-in-rust/wiki) for more detailed explanation of settings, write-ups for each maze generation algorithm, and much more. Thank you!
