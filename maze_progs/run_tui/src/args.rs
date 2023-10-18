use crate::tables;

pub struct FlagArg<'a, 'b> {
    pub flag: &'a str,
    pub arg: &'b str,
}

#[derive(Clone, Copy)]
pub enum ViewingMode {
    StaticImage,
    AnimatedPlayback,
}

#[derive(Clone, Copy)]
pub struct MazeRunner {
    pub args: maze::MazeArgs,
    pub build_view: ViewingMode,
    pub build_speed: speed::Speed,
    pub build: tables::BuildFunction,
    pub modify: Option<tables::BuildFunction>,
    pub solve_view: ViewingMode,
    pub solve_speed: speed::Speed,
    pub solve: tables::SolveFunction,
}

impl MazeRunner {
    pub fn new() -> Self {
        Self {
            args: maze::MazeArgs {
                odd_rows: 33,
                odd_cols: 111,
                offset: maze::Offset::default(),
                style: maze::MazeStyle::Sharp,
            },
            build_view: ViewingMode::AnimatedPlayback,
            build_speed: speed::Speed::Speed4,
            build: (
                tables::recursive_backtracker::generate_maze,
                tables::recursive_backtracker::animate_maze,
            ),
            modify: None,
            solve_view: ViewingMode::AnimatedPlayback,
            solve_speed: speed::Speed::Speed4,
            solve: (tables::dfs::hunt, tables::dfs::animate_hunt),
        }
    }
}
