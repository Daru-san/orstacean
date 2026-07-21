use std::cell::{Ref, RefCell};
use std::ops::AddAssign;
use std::rc::Rc;
use std::time::Duration;

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
use crate::app::puzzles::riddles::Riddles;
use crate::app::puzzles::word_match::WordMatch;

mod cipher;
mod maze;
mod riddles;
mod timer;
mod word_match;

pub struct PuzzleView {
    active_puzzle: Puzzle,
    results: Vec<String>,
    puzzle: Rc<RefCell<dyn IPuzzle>>,
    chatbox: ChatBox,
    state: AppState,
    failures: u128,
}

enum Puzzle {
    Tile1,
    Tile2,
    Riddle,
    Maze,
    Cipher,
}

pub trait IPuzzle {
    fn update(&mut self);
    fn handle_events(&mut self, event: Event) -> color_eyre::Result<()>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn keys_hints<'a>(&self) -> Line<'a>;
    fn instructions(&self) -> Vec<String>;
    fn completed(&self) -> bool;
    fn failed(&self) -> bool;
    fn toggle_pause(&mut self, pause: bool);
    fn is_paused(&self) -> bool;
    fn can_pause(&self) -> bool;
}

impl PuzzleView {
    pub fn new(state: AppState) -> color_eyre::Result<Self> {
        let grid = WordMatch::new("CRAB", None)?;
        let instructions = grid.instructions();
        let puzzle = Rc::new(RefCell::new(grid));
        Ok(Self {
            chatbox: ChatBox::new(&instructions, state.clone()),
            results: Vec::new(),
            active_puzzle: Puzzle::Tile1,
            puzzle,
            state,
            failures: 0,
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
            .render(header_area, frame.buffer_mut());

        let mut puzzle = self.puzzle.borrow_mut();

        let mut help = Line::from_iter(self.state.volume_hints());
        {
            let puzzle = self.puzzle.borrow();
            if puzzle.can_pause() && !puzzle.completed() {
                help.extend([
                    Span::styled("C-P", Style::default().add_modifier(Modifier::ITALIC)),
                    Span::raw(if puzzle.is_paused() {
                        "❚❚   "
                    } else {
                        "▶   "
                    }),
                ]);
            }
        }
        help.extend(puzzle.keys_hints());
        help.extend([
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": help  "),
        ]);

        help.render(bottom_area, frame.buffer_mut());

        if self.chatbox.done() {
            puzzle.render(frame, main_area);
        } else {
            let layout = Layout::vertical([Percentage(50), Percentage(50)]);
            let [puzzle_area, side_area] = layout.areas(main_area);
            puzzle.render(frame, side_area);
            self.chatbox.render(frame, puzzle_area);
        }
    }

    pub fn handle_events(&mut self, event: Event) -> color_eyre::Result<()> {
        if self.chatbox.done() {
            let mut puzzle = self.puzzle.borrow_mut();
            if let Event::Key(key) = event
                && matches!(key.code, KeyCode::Char('?'))
                && key.modifiers.is_empty()
            {
                let mut instructions = vec![
                    String::from("You need more instructions?"),
                    String::from("I'll start from the top, listen carefully"),
                ];
                instructions.extend(puzzle.instructions());
                self.chatbox = ChatBox::new(&instructions, self.state.clone());
            }
            puzzle.handle_events(event)
        } else {
            self.chatbox.handle_events(event)
        }
    }

    pub fn update(&mut self) -> color_eyre::Result<()> {
        if self.chatbox.update() {
            let mut puzzle = self.puzzle.borrow_mut();
            puzzle.update();

            if puzzle.completed() {
                drop(puzzle);
                self.advance()?;
            } else if puzzle.failed() {
                drop(puzzle);
                self.demote()?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn advance(&mut self) -> color_eyre::Result<()> {
        match self.active_puzzle {
            Puzzle::Tile1 => {
                let next = WordMatch::new("CRUSTACEAN", Some(Duration::from_mins(5)))?;
                let instructions = next.instructions();
                let puzzle = Rc::new(RefCell::new(next));

                self.active_puzzle = Puzzle::Tile2;
                self.puzzle = puzzle;
                self.chatbox = ChatBox::new(
                    &[
                        vec![
                            String::from("Congratulations on finishing the last puzzle."),
                            String::from("It only gets harder from here."),
                        ],
                        instructions,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                    self.state.clone(),
                );

                self.results.push(String::from("Crab"));
            }
            Puzzle::Tile2 => {
                let next = Riddles::new(Duration::from_mins(10));
                let instructions = next.instructions();
                let puzzle = Rc::new(RefCell::new(next));
                self.active_puzzle = Puzzle::Riddle;
                self.puzzle = puzzle;

                self.chatbox = ChatBox::new(
                    &[
                        vec![
                            String::from("Congratulations on completing the last puzzle."),
                            String::from("Next, you will need to wrack you brain."),
                        ],
                        instructions,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                    self.state.clone(),
                );
                self.results.push(String::from("Crustacean"));
            }
            Puzzle::Riddle => {
                let next = Maze::new(Duration::from_mins(5));
                let instructions = next.instructions();
                let puzzle = Rc::new(RefCell::new(next));

                self.active_puzzle = Puzzle::Tile2;
                self.puzzle = puzzle;
                self.chatbox = ChatBox::new(
                    &[
                        vec![
                            String::from("Congratulations on finishing the last puzzle."),
                            String::from("I will admit that the rules were rather unfair."),
                            String::from("Now we move on to my favorite puzzle."),
                        ],
                        instructions,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                    self.state.clone(),
                );

                // self.results.push(String::from("Crab"));
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    pub fn demote(&mut self) -> color_eyre::Result<()> {
        match self.active_puzzle {
            Puzzle::Tile1 => {
                let next = WordMatch::new("CRAB", Some(Duration::from_mins(2)))?;
                let instructions = next.instructions();
                let puzzle = Rc::new(RefCell::new(next));

                self.active_puzzle = Puzzle::Tile1;
                self.puzzle = puzzle;
                self.chatbox = ChatBox::new(
                    &[
                        vec![
                            String::from(
                                "Unfortunately, you've failed the first and simplest puzzle.",
                            ),
                            String::from("I am incredibly disappointed."),
                            String::from("Luckily for you, I am a benevolent crab"),
                            String::from("Thus, we will start from the beginning."),
                        ],
                        instructions,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                    self.state.clone(),
                );
            }
            Puzzle::Tile2 => {}
            Puzzle::Riddle => {}
            Puzzle::Maze => {
                let next = WordMatch::new("CRAB", Some(Duration::from_mins(1)))?;
                let instructions = next.instructions();
                let puzzle = Rc::new(RefCell::new(next));

                self.active_puzzle = Puzzle::Tile1;
                self.puzzle = puzzle;
                self.chatbox = ChatBox::new(
                    &[
                        vec![
                            String::from("Unfortunately, we were unable to reach the end in time."),
                            String::from("I am incredibly disappointed."),
                            String::from("Then again, I am proud of your effort."),
                            String::from("Luckily for you, I am a benevolent crab"),
                            String::from("Thus, we will start from the beginning."),
                        ],
                        instructions,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                    self.state.clone(),
                );
                self.results.clear();
            }
            Puzzle::Cipher => {}
        }

        self.failures.add_assign(1);

        Ok(())
    }
}
