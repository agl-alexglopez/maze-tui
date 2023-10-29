use maze;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct MaxMap {
    pub max: u64,
    pub distances: HashMap<maze::Point, u64>,
}

impl MaxMap {
    pub fn new(p: maze::Point, m: u64) -> Self {
        Self {
            max: m,
            distances: HashMap::from([(p, m)]),
        }
    }
}

pub struct Solver {
    pub maze: maze::Maze,
    pub win: Option<usize>,
    pub win_path: Vec<(maze::Point, u16)>,
    pub map: MaxMap,
    pub count: usize,
}

impl Solver {
    pub fn new(boxed_maze: maze::Maze) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            maze: boxed_maze,
            win: None,
            win_path: Vec::default(),
            map: MaxMap::default(),
            count: 0,
        }))
    }
}

pub type SolverMonitor = Arc<Mutex<Solver>>;
