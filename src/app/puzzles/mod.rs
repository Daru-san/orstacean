use std::cell::RefCell;
use std::rc::Rc;

use ratatui::Frame;
use ratatui::layout::Constraint::{Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Widget;

use crate::app::puzzles::maze::Maze;
use crate::app::puzzles::word_match::PaneGrid;

mod cipher;
mod maze;
mod riddles;
mod word_match;

pub struct PuzzleView {
    active_puzzle: Puzzle,
    results: Vec<String>,
    puzzle: Rc<RefCell<dyn IPuzzle>>,
}

enum Puzzle {
    Tile1(Rc<RefCell<PaneGrid>>),
    Tile2(Rc<RefCell<PaneGrid>>),
    Riddle,
    Maze(Rc<RefCell<Maze>>),
    Cipher,
}

pub trait IPuzzle {
    fn update(&mut self);
    fn handle_events(&mut self) -> color_eyre::Result<()>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn keys_hints<'a>(&self) -> Line<'a>;
}

impl PuzzleView {
    pub fn new() -> Self {
        let grid = Rc::new(RefCell::new(PaneGrid::new("CRAB")));
        let puzzle = Puzzle::Tile1(grid.clone());
        Self {
            active_puzzle: puzzle,
            results: Vec::new(),
            puzzle: grid,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = area.layout(&layout);

        let mut puzzle = self.puzzle.borrow_mut();

        let help = puzzle.keys_hints();

        help.render(bottom_area, frame.buffer_mut());
    }
}
