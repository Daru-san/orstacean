use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Constraint::{self, Min, Percentage};
use ratatui::layout::{Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::APP_NAME;
use crate::app::AppState;
use crate::app::chat_box::ChatBox;
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
    chatbox: ChatBox,
    state: AppState,
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
    fn instructions(&self) -> Vec<String>;
}

impl PuzzleView {
    pub fn new(state: AppState) -> color_eyre::Result<Self> {
        let grid = PaneGrid::new("CRAB")?;
        let instructions = grid.instructions();
        let puzzle = Rc::new(RefCell::new(grid));
        Ok(Self {
            chatbox: ChatBox::new(&instructions, state.clone()),
            results: Vec::new(),
            active_puzzle: Puzzle::Tile1(puzzle.clone()),
            puzzle,
            state,
        })
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = area.layout(&layout);

        let layout = Layout::vertical([Constraint::Length(3), Min(0)]);
        let [header_area, main_area] = main_area.layout(&layout);

        Paragraph::new(format!("{APP_NAME}: Puzzles"))
            .bold()
            .centered()
            .block(Block::bordered())
            .slow_blink()
            .render(header_area, frame.buffer_mut());

        let mut puzzle = self.puzzle.borrow_mut();

        let volume = self.state.volume_hints();
        let mut help = puzzle.keys_hints();
        help.extend(volume);
        help.extend([
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": help  "),
        ]);

        help.render(bottom_area, frame.buffer_mut());

        if !self.chatbox.done() {
            let layout = Layout::vertical([Percentage(50), Percentage(50)]);
            let [puzzle_area, side_area] = layout.areas(main_area);
            puzzle.render(frame, side_area);
            self.chatbox.render(frame, puzzle_area);
        } else {
            puzzle.render(frame, main_area);
        }
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        if !self.chatbox.done() {
            self.chatbox.handle_events()
        } else {
            let mut puzzle = self.puzzle.borrow_mut();
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('?')) && key.modifiers.is_empty() {
                    let mut instructions = vec![
                        String::from("You need more instructions?"),
                        String::from("I'll start from the top, listen carefully"),
                    ];
                    instructions.extend(puzzle.instructions());
                    self.chatbox = ChatBox::new(&instructions, self.state.clone());
                }
            }
            puzzle.handle_events()
        }
    }

    pub fn update(&mut self) -> color_eyre::Result<()> {
        if !self.chatbox.update()? {
            Ok(())
        } else {
            let mut puzzle = self.puzzle.borrow_mut();
            Ok(puzzle.update())
        }
    }
}
