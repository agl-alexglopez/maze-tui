mod run_random;
use builders::arena;
use builders::eller;
use builders::grid;
use builders::kruskal;
use builders::modify;
use builders::prim;
use builders::recursive_backtracker;
use builders::recursive_subdivision;
use builders::wilson_adder;
use builders::wilson_carver;
use crossterm::terminal::EnterAlternateScreen;
use painters::distance;
use painters::runs;
use ratatui::widgets::Borders;
use ratatui::{
    prelude::{Alignment, CrosstermBackend, Terminal},
    widgets::{Block, Paragraph},
};

use solvers::bfs;
use solvers::darkbfs;
use solvers::darkdfs;
use solvers::darkfloodfs;
use solvers::darkrdfs;
use solvers::dfs;
use solvers::floodfs;
use solvers::rdfs;

use std::collections::{HashMap, HashSet};

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;
type BuildFunction = (fn(&mut maze::Maze), fn(&mut maze::Maze, speed::Speed));
type SolveFunction = (fn(maze::BoxMaze), fn(maze::BoxMaze, speed::Speed));

struct FlagArg<'a, 'b> {
    flag: &'a str,
    arg: &'b str,
}

enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

struct MazeRunner {
    args: maze::MazeArgs,
    build_view: ViewingMode,
    build_speed: speed::Speed,
    build: BuildFunction,
    modify: Option<BuildFunction>,
    solve_view: ViewingMode,
    solve_speed: speed::Speed,
    solve: SolveFunction,
    quit: bool,
}

impl MazeRunner {
    fn default() -> Self {
        Self {
            args: maze::MazeArgs {
                odd_rows: 33,
                odd_cols: 111,
                style: maze::MazeStyle::Contrast,
            },
            build_view: ViewingMode::AnimatedPlayback,
            build_speed: speed::Speed::Speed4,
            build: (
                recursive_backtracker::generate_maze,
                recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::AnimatedPlayback,
            solve_speed: speed::Speed::Speed4,
            solve: (dfs::hunt, dfs::animate_hunt),
            quit: false,
        }
    }
}

struct LookupTables {
    arg_flags: HashSet<String>,
    build_table: HashMap<String, BuildFunction>,
    mod_table: HashMap<String, BuildFunction>,
    solve_table: HashMap<String, SolveFunction>,
    style_table: HashMap<String, maze::MazeStyle>,
    animation_table: HashMap<String, speed::Speed>,
}

fn main() -> std::io::Result<()> {
    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}

fn run() -> std::io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let tables = LookupTables {
        arg_flags: HashSet::from([
            String::from("-r"),
            String::from("-c"),
            String::from("-b"),
            String::from("-s"),
            String::from("-h"),
            String::from("-d"),
            String::from("-m"),
            String::from("-sa"),
            String::from("-ba"),
        ]),
        build_table: HashMap::from([
            (
                String::from("rdfs"),
                (
                    recursive_backtracker::generate_maze as fn(&mut maze::Maze),
                    recursive_backtracker::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("fractal"),
                (
                    recursive_subdivision::generate_maze as fn(&mut maze::Maze),
                    recursive_subdivision::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("grid"),
                (
                    grid::generate_maze as fn(&mut maze::Maze),
                    grid::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("prim"),
                (
                    prim::generate_maze as fn(&mut maze::Maze),
                    prim::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("kruskal"),
                (
                    kruskal::generate_maze as fn(&mut maze::Maze),
                    kruskal::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("eller"),
                (
                    eller::generate_maze as fn(&mut maze::Maze),
                    eller::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson"),
                (
                    wilson_carver::generate_maze as fn(&mut maze::Maze),
                    wilson_carver::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("wilson-walls"),
                (
                    wilson_adder::generate_maze as fn(&mut maze::Maze),
                    wilson_adder::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("arena"),
                (
                    arena::generate_maze as fn(&mut maze::Maze),
                    arena::animate_maze as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        mod_table: HashMap::from([
            (
                String::from("cross"),
                (
                    modify::add_cross as fn(&mut maze::Maze),
                    modify::add_cross_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
            (
                String::from("x"),
                (
                    modify::add_x as fn(&mut maze::Maze),
                    modify::add_x_animated as fn(&mut maze::Maze, speed::Speed),
                ),
            ),
        ]),
        solve_table: HashMap::from([
            (
                String::from("dfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    dfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    dfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("dfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    dfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    bfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    bfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("bfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    bfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    floodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    floodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("floodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    floodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    rdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    rdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("rdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    rdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-hunt"),
                (
                    dfs::hunt as fn(maze::BoxMaze),
                    darkdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-gather"),
                (
                    dfs::gather as fn(maze::BoxMaze),
                    darkdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkdfs-corners"),
                (
                    dfs::corner as fn(maze::BoxMaze),
                    darkdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-hunt"),
                (
                    floodfs::hunt as fn(maze::BoxMaze),
                    darkfloodfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-gather"),
                (
                    floodfs::gather as fn(maze::BoxMaze),
                    darkfloodfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkfloodfs-corners"),
                (
                    floodfs::corner as fn(maze::BoxMaze),
                    darkfloodfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-hunt"),
                (
                    rdfs::hunt as fn(maze::BoxMaze),
                    darkrdfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-gather"),
                (
                    rdfs::gather as fn(maze::BoxMaze),
                    darkrdfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkrdfs-corners"),
                (
                    rdfs::corner as fn(maze::BoxMaze),
                    darkrdfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-hunt"),
                (
                    bfs::hunt as fn(maze::BoxMaze),
                    darkbfs::animate_hunt as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-gather"),
                (
                    bfs::gather as fn(maze::BoxMaze),
                    darkbfs::animate_gather as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("darkbfs-corners"),
                (
                    bfs::corner as fn(maze::BoxMaze),
                    darkbfs::animate_corner as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("distance"),
                (
                    distance::paint_distance_from_center as fn(maze::BoxMaze),
                    distance::animate_distance_from_center as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
            (
                String::from("runs"),
                (
                    runs::paint_run_lengths as fn(maze::BoxMaze),
                    runs::animate_run_lengths as fn(maze::BoxMaze, speed::Speed),
                ),
            ),
        ]),
        style_table: HashMap::from([
            (String::from("sharp"), maze::MazeStyle::Sharp),
            (String::from("round"), maze::MazeStyle::Round),
            (String::from("doubles"), maze::MazeStyle::Doubles),
            (String::from("bold"), maze::MazeStyle::Bold),
            (String::from("contrast"), maze::MazeStyle::Contrast),
            (String::from("spikes"), maze::MazeStyle::Spikes),
        ]),
        animation_table: HashMap::from([
            (String::from("0"), speed::Speed::Instant),
            (String::from("1"), speed::Speed::Speed1),
            (String::from("2"), speed::Speed::Speed2),
            (String::from("3"), speed::Speed::Speed3),
            (String::from("4"), speed::Speed::Speed4),
            (String::from("5"), speed::Speed::Speed5),
            (String::from("6"), speed::Speed::Speed6),
            (String::from("7"), speed::Speed::Speed7),
        ]),
    };
    let mut run = MazeRunner::default();

    'poll: loop {
        terminal.draw(|f| {
            ui(f);
        })?;
        update(&mut run, &tables)?;
        if run.quit {
            break 'poll;
        }
    }
    Ok(())
}

fn startup() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> std::io::Result<()> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn ui(f: &mut Frame<'_>) {
    f.render_widget(
        Paragraph::new("Welcome to the Maze Runner TUI! Press <r> to run.")
            .block(Block::new().title("Mazes").borders(Borders::ALL))
            .alignment(Alignment::Center),
        f.size(),
    );
}

fn update(run: &mut MazeRunner, tables: &LookupTables) -> std::io::Result<()> {
    if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => run.quit = true,
                    crossterm::event::KeyCode::Char('r') => {
                        run_random::rand(run.args.odd_rows, run.args.odd_cols)
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(())
}
