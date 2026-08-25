use std::collections::BTreeMap;
use std::collections::VecDeque;
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
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Padding;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui_comfy_toaster::ToastBuilder;
use ratatui_comfy_toaster::ToastType;

use crate::app::AppState;
use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::timer::Timer;

pub struct WordMatch {
    word: Word,
    completed: bool,
    timer: Option<Timer>,
    word_chars: VecDeque<char>,
    hovered_index: usize,
    app_state: AppState,
    remaining_hints: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum Word {
    Crab,
    Crustacean,
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        match self {
            Word::Crab => "CRAB",
            Word::Crustacean => "CRUSTACEAN",
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Crab => f.write_str("CRAB"),
            Self::Crustacean => f.write_str("CRUSTACEAN"),
        }
    }
}

enum Direction {
    Left,
    Right,
}

impl WordMatch {
    pub fn new(
        app_state: AppState,
        word: Word,
        timeout: Option<Duration>,
    ) -> color_eyre::Result<Self> {
        let mut chars: Vec<char> = word.as_ref().chars().collect();

        let mut rng = rand::rng();
        // shuffle chars
        chars.shuffle(&mut rng);

        Ok(Self {
            word,
            completed: false,
            timer: timeout.map(Timer::new),
            word_chars: chars.into(),
            hovered_index: 0,
            app_state,
            remaining_hints: match word {
                Word::Crab => 0,
                Word::Crustacean => 1,
            },
        })
    }

    fn check_sorted(&self) -> bool {
        self.word
            .as_ref()
            .chars()
            .eq(self.word_chars.iter().copied())
    }

    fn draw_tiles(&self, frame: &mut Frame, area: Rect) {
        let width = area.width;

        let inner_width = width.saturating_sub(5);

        let individual_size = inner_width
            .div(self.word_chars.len() as u16)
            .min(area.height);

        let mut x = 0;
        for (index, char) in self.word_chars.iter().copied().enumerate() {
            let rect = Rect::new(x, 0, individual_size, individual_size);
            Paragraph::new(char.to_string())
                .style(Style::default().add_modifier(Modifier::BOLD))
                .centered()
                .block(
                    Block::bordered()
                        .padding(Padding::uniform(4))
                        .border_set(border::THICK)
                        .border_style(Style::default().fg(if index == self.hovered_index {
                            Color::Blue
                        } else {
                            Color::White
                        })),
                )
                .render(rect, frame.buffer_mut());
            x += individual_size;
        }
    }

    fn move_selected_tile(&mut self, direction: Direction) {
        let index = self.hovered_index;

        let max = self.word_chars.len();

        if index < max {
            let new_index = match direction {
                Direction::Left => {
                    if index == 0 {
                        return;
                    } else {
                        index - 1
                    }
                }
                Direction::Right => index + 1,
            };

            if new_index == max {
                return;
            }

            self.word_chars.swap(index, new_index);
            self.hovered_index = new_index;
        }
    }

    fn select_tile(&mut self, direction: Direction) {
        let index = self.hovered_index;

        let max = self.word_chars.len();

        if index < max {
            let new_index = match direction {
                Direction::Left => index.saturating_sub(1),
                Direction::Right => (index + 1).min(max - 1),
            };

            self.hovered_index = new_index;
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
            self.draw_tiles(frame, area);
        }
    }

    fn update(&mut self) {
        if let Some(ref mut timer) = self.timer {
            if matches!(self.word, Word::Crustacean)
                && timer.percentage_done() >= 50.0
                && timer.percentage_done() <= 60.0
                && self.remaining_hints > 0
            {
                self.app_state.show_toast(
                    ToastBuilder::default().title("The word contains the phrase `Rust`."),
                );
            }
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
                self.select_tile(Direction::Left);
            }
            KeyCode::Right | KeyCode::Char('l') if none => {
                self.select_tile(Direction::Right);
            }
            KeyCode::Left | KeyCode::Char('H') if shift => {
                self.move_selected_tile(Direction::Left);
            }
            KeyCode::Right | KeyCode::Char('L') if shift => {
                self.move_selected_tile(Direction::Right)
            }
            KeyCode::Char('s') if ctrl => {
                self.completed = true;
                self.app_state.show_toast(
                    ToastBuilder::default()
                        .duration(Duration::from_secs(1))
                        .toast_type(ToastType::Success)
                        .title(if self.completed() {
                            format!("Word {} was correct!", self.word)
                        } else {
                            format!(
                                "Word {} was incorrect.",
                                self.word_chars.iter().copied().collect::<String>(),
                            )
                        }),
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn failure_message(&self) -> Vec<String> {
        match self.word {
            Word::Crab => vec![
                String::from("Spelling."),
                String::from("S..p..e..l..l..i..n..g"),
                String::from("How did you fail to spell such a simple word??"),
                String::from("I won't hold this against you."),
                String::from("No, I will absolutely hold it against you."),
                String::from("Come back when you can spell correctly."),
            ],
            Word::Crustacean => {
                if self.timer.as_ref().is_some_and(|timer| timer.done()) {
                    vec![String::from("")]
                } else {
                    vec![String::from("")]
                }
            }
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
        if let Some(ref timer) = self.timer {
            vec![
                String::from("Arrange the letters in the grid until they make up a word."),
                String::from(
                    "This word describes a particular creature commonly found on beaches.",
                ),
                String::from("Be mindful of your time."),
                format!(
                    "You have exactly {:?} to complete this challenge.",
                    timer.duration()
                ),
                String::from(
                    "Your time will be shown in the bottom left corner, be sure to pay attention to it.",
                ),
            ]
        } else {
            vec![
                String::from("Welcome to the Word Match Puzzle."),
                String::from("This one is rather simple."),
                String::from(
                    "This word describes a particular creature commonly found on beaches.",
                ),
                String::from(
                    "Arrange the letters in the grid until they make up the word in question.",
                ),
                String::from("No time limitation shall be put on you during this puzzle."),
            ]
        }
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
