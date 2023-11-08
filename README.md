# Maze TUI

![demo](/images/demo.gif)

> **For a complete breakdown of all maze generation algorithms, pseudocode, and many other important details please read the [wiki](https://github.com/agl-alexglopez/maze-tui/wiki) This is just the quick start guide to refer to for usage instructions.**

## Requirements

- Rustc >= 1.73.0 or as new as possible `rustup update`. An earlier version may work but if you receive an error regarding the equality operator `==` a newer rustc version fixes that issue. 
- A terminal with full font support for special Unicode characters. This project leverages Unicode Box-Drawing characters.
- A terminal with support for RGB colors as defined by Crossterm's compatibility specifications. ANSI color codes are also used.
- Cargo to build and run the project.   

## Quick Start Guide

This project is a terminal user interface application powered by [crossterm](https://github.com/crossterm-rs/crossterm), [ratatui.rs](https://github.com/ratatui-org/ratatui), and [tui-textarea](https://github.com/rhysd/tui-textarea) that can be run with various combinations of commands. The goal of the program is to explore various maze building, solving, and measurement algorithms while visualizing them in interesting ways. The basic principle behind the program is that you can ask for any combination of settings, include any settings, exclude any settings, and the program will just work. There are sensible defaults for every flag so experiment with different combinations and tweaks until you get what you are looking for. In addition, you can learn about each maze algorithm you observe with informational popups (they are a work in progress right now!).

If you just want to get started follow these steps.

```zsh
$ git clone https://github.com/agl-alexglopez/maze-tui.git
$ cd maze_tui/maze_progs/
$ cargo run --release --bin run_tui
```

You will be greeted by the home page. Read the directions or if you just want to see some cool maze animations right away press `<Enter>`. If you choose to specify arguments and wish to see maze building or solving algorithms animated, be sure to enter the animation speed with the `-ba` or `-sa` flags. See the introduction below.

## Run TUI Program

Here are the instructions that greet you upon running the application.

```txt
███╗   ███╗ █████╗ ███████╗███████╗    ████████╗██╗   ██╗██╗
████╗ ████║██╔══██╗╚══███╔╝██╔════╝    ╚══██╔══╝██║   ██║██║
██╔████╔██║███████║  ███╔╝ █████╗         ██║   ██║   ██║██║
██║╚██╔╝██║██╔══██║ ███╔╝  ██╔══╝         ██║   ██║   ██║██║
██║ ╚═╝ ██║██║  ██║███████╗███████╗       ██║   ╚██████╔╝██║
╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝       ╚═╝    ╚═════╝ ╚═╝

Use flags, followed by arguments, in any order.
Press <ENTER> to confirm your flag choices.

(scroll with <↓>/<↑>, exit with <ESC>)

BUILDER FLAG[-b] Set maze building algorithm.
    [rdfs] - Randomized depth first search.
    [hunt-kill] - Randomized walks and scans.
    [kruskal] - Randomized Kruskal's algorithm.
    [prim] - Randomized Prim's algorithm.
    [eller] - Randomized Eller's algorithm.
    [wilson] - Loop-erased random path carver.
    [wilson-walls] - Loop-erased random wall adder.
    [fractal] - Randomized recursive subdivision.
    [grid] - A random grid pattern.
    [arena] - Open floor with no walls.

MODIFICATION FLAG[-m] Add shortcuts to the maze.
    [cross]- Add crossroads through the center.
    [x]- Add an x of crossing paths through center.

SOLVER FLAG[-s] Set maze solving algorithm.
    [dfs-hunt] - Depth First Search
    [dfs-gather] - Depth First Search
    [dfs-corner] - Depth First Search
    [floodfs-hunt] - Depth First Search
    [floodfs-gather] - Depth First Search
    [floodfs-corner] - Depth First Search
    [rdfs-hunt] - Randomized Depth First Search
    [rdfs-gather] - Randomized Depth First Search
    [rdfs-corner] - Randomized Depth First Search
    [bfs-hunt] - Breadth First Search
    [bfs-gather] - Breadth First Search
    [bfs-corner] - Breadth First Search

WALL FLAG[-w] Set the wall style for the maze.
    [mini] - Half size walls and paths.
    [sharp] - The default straight lines.
    [round] - Rounded corners.
    [doubles] - Sharp double lines.
    [bold] - Thicker straight lines.
    [contrast] - Full block width and height walls.
    [half] - Half wall height and full size paths.
    [spikes] - Connected lines with spikes.

Animations can play forward or reversed.
Cancel any animation by pressing <escape>.
Pause/Play an animation with <space>
Speed up or slow down the animation with <⮝/⮟>
Step next/previous or change the play direction with <⮜/⮞>
Zoom out/in with <Ctrl-[-]>/<Ctrl-[+]>
If any flags are omitted, defaults are used.
An empty command line will create a random maze.

EXAMPLES:

-b rdfs -s bfs-hunt
-s bfs-gather -b prim
-s bfs-corner -w mini -b fractal

ASCII lettering for this title and algorithm
descriptions are templates I used from
patorjk.com and modified to use box-drawing
characters.

Enjoy!
```

## Details

The underlying principles for this program are as follows.

1. Build a maze by representing paths and walls in a `u16` integer.
2. Use logical and visually pleasing unicode characters to represent the walls and features of the maze.
3. Solve the maze with various algorithms, both single and multi-threaded, so that we can observe important features of the mazes.

I included as many **interesting** maze building algorithms as I could. The solving algorithms are multi-threaded in many cases. This is not particularly practical or special on its own. However, multithreading allows for more fun and interesting visualizations and can speed up what can sometimes be a slow solving process to watch for a single thread.

While I have not yet put together a testing suite for performance testing of building and solving the mazes, I will be interested to see the performance implications of the solvers. The [wiki](https://github.com/agl-alexglopez/maze-tui/wiki) is likely where you will find documentation of new features or other testing.

## Wiki

Please read the [wiki](https://github.com/agl-alexglopez/maze-tui/wiki) for more detailed explanation of settings, write-ups for each maze generation algorithm, and much more. Thank you!

## Tips

- Windows is slow, sorry. For some reason the animations are much slower in Windows Terminal in my experience. It's not horrible but try smaller mazes at first.
- Windows Terminal does not render fonts correctly. Depending on your font the Box-Drawing characters may be slightly disconnected or at times completely incorrect. I develop this project on WSL2 sometimes and notice the problems. Try the `-w contrast` or `-w half` option for smoother animations.
- Mac and many Linux distributions render everything beautifully in my experience (as long as fully featured fonts are installed)!

