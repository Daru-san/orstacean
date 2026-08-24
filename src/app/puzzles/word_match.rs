use std::collections::BTreeMap;
use std::ops::Div;
use std::ops::Sub;
use std::time::Duration;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use rand::seq::SliceRandom;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::Min;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};
use ratatui_hypertile::HypertileWidget;
use ratatui_hypertile::{Hypertile, HypertileAction, MoveScope, PaneId, PaneSnapshot, Towards};

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::timer::Timer;

type PaneMap = BTreeMap<PaneId, (char, Color)>;

pub struct WordMatch {
    word: &'static str,
    completed: bool,
    timer: Option<Timer>,
    word_chars: Vec<char>,
}

impl WordMatch {
    pub fn new(word: &'static str, timeout: Option<Duration>) -> color_eyre::Result<Self> {
        let mut layout = Hypertile::new();

        let mut chars: Vec<char> = word.chars().collect();

        let mut rng = rand::rng();
        // shuffle chars
        chars.shuffle(&mut rng);

        Ok(Self {
            word,
            completed: false,
            timer: timeout.map(Timer::new),
            word_chars: chars,
        })
    }

    fn check_sorted(&self) -> bool {
        self.word.chars().eq(self.word_chars.iter().copied())
    }

    fn draw_tiles(&self, frame: &mut Frame, area: Rect) {
        let width = area.width;

        let inner_width = width.saturating_sub(5);

        let individual_size = inner_width
            .div(self.word_chars.len() as u16)
            .min(area.height);

        let mut x = 0;
        for char in self.word_chars.iter().copied() {
            let rect = Rect::new(x, 0, individual_size, individual_size);
            Paragraph::new(char.to_string())
                .block(Block::bordered())
                .render(rect, frame.buffer_mut());
            x += individual_size;
        }
    }
}

impl IPuzzle for WordMatch {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut timer) = self.timer {
            let layout = Layout::vertical([Percentage(100), Min(1)]);
            let [main_area, bottom_area] = area.layout(&layout);
            timer.render(bottom_area, frame.buffer_mut());

            self.draw_tiles(frame, main_area);
        } else {
        }
    }

    fn update(&mut self) {
        if let Some(ref mut timer) = self.timer {
            timer.update();
        }
    }

    fn completed(&self) -> bool {
        self.check_sorted() && self.completed
    }
    fn failed(&self) -> bool {
        self.completed && !self.check_sorted()
    }
    fn handle_events(&mut self, event: Event) -> color_eyre::Result<()> {
        let Event::Key(key) = event else {
            return Ok(());
        };

        let none = key.modifiers == KeyModifiers::NONE;
        let shift = key.modifiers == KeyModifiers::SHIFT;
        let ctrl = key.modifiers == KeyModifiers::CONTROL;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') if none => {
                focus(&mut self.layout, Direction::Horizontal, Towards::Start);
            }
            KeyCode::Right | KeyCode::Char('l') if none => {
                focus(&mut self.layout, Direction::Horizontal, Towards::End);
            }
            KeyCode::Up | KeyCode::Char('k') if none => {
                focus(&mut self.layout, Direction::Vertical, Towards::Start);
            }
            KeyCode::Down | KeyCode::Char('j') if none => {
                focus(&mut self.layout, Direction::Vertical, Towards::End);
            }

            KeyCode::Left | KeyCode::Char('H') if shift => {
                move_pane(&mut self.layout, Direction::Horizontal, Towards::Start);
            }
            KeyCode::Right | KeyCode::Char('L') if shift => {
                move_pane(&mut self.layout, Direction::Horizontal, Towards::End);
            }
            KeyCode::Up | KeyCode::Char('K') if shift => {
                move_pane(&mut self.layout, Direction::Vertical, Towards::Start);
            }
            KeyCode::Down | KeyCode::Char('J') if shift => {
                move_pane(&mut self.layout, Direction::Vertical, Towards::End);
            }
            KeyCode::Char('s') if ctrl => {
                self.completed = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn failure_message(&self) -> Vec<String> {
        match self.word {
            "CRAB" => vec![
                String::from("Spelling."),
                String::from("S..p..e..l..l..i..n..g"),
                String::from("How did you fail to spell such a simple word??"),
                String::from("I won't hold this against you."),
                String::from("No, I will absolutely hold it against you."),
                String::from("Come back when you can spell correctly."),
            ],
            "CRUSTACEAN" => {
                if self.timer.as_ref().is_some_and(|timer| timer.done()) {
                    vec![String::from("")]
                } else {
                    vec![String::from("")]
                }
            }
            _ => unreachable!("The only two words on this puzzle must be 'CRAB' and 'CRUSTACEAN'."),
        }
    }

    fn keys_hints<'a>(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled("C-Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("C-R", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": reset  "),
            Span::styled("C-S", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": submit  "),
            Span::styled("H or ◄", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": left  "),
            Span::styled("J or ▲", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": up  "),
            Span::styled("K or ▼", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
            Span::styled("L or ►", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": right  "),
            Span::styled("S-<Key>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": move  "),
        ])
    }

    fn instructions(&self) -> Vec<String> {
        vec![
            String::from("Welcome to the Word Match Puzzle."),
            String::from("This one is rather simple."),
            String::from(
                "Arrange the letters in the grid until they make up the word in question.",
            ),
            String::from("No time limitation shall be put on you during this puzzle."),
        ]
    }

    fn toggle_pause(&mut self, pause: bool) {
        if let Some(timer) = self.timer.as_mut() {
            if pause {
                timer.pause();
            } else {
                timer.unpause();
            }
        }
    }

    fn is_paused(&self) -> bool {
        self.timer.as_ref().is_some_and(Timer::is_paused)
    }

    fn can_pause(&self) -> bool {
        self.timer.is_some()
    }
}

fn move_pane(layout: &mut Hypertile, direction: Direction, towards: Towards) {
    layout.apply_action(HypertileAction::MoveFocused {
        direction,
        towards,
        scope: MoveScope::Window,
    });
}

fn focus(layout: &mut Hypertile, direction: Direction, towards: Towards) {
    layout.apply_action(HypertileAction::FocusDirection { direction, towards });
}

fn render_pane(pane: PaneSnapshot, buf: &mut Buffer, panes: &PaneMap) {
    let (title, color) = panes
        .get(&pane.id)
        .map(|(t, c)| (*t, *c))
        .unwrap_or(('%', Color::White));

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title.to_string());
    if pane.is_focused {
        block = block
            .border_set(border::THICK)
            .border_style(Style::default().fg(color).bold());
    }
    block.render(pane.rect, buf);
}
