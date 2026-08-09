use std::time::Duration;

use crossterm::event::Event;
use rand::seq::SliceRandom;
use ratatui::layout::Constraint::{Min, Percentage};
use ratatui::layout::Layout;
use ratatui::style::{Modifier, Style};
use ratatui::text::{self, Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui_form::TextInput;
use ratatui_textarea::TextArea;
use rodio::cpal::FromSample;

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::riddles::list::RIDDLES;
use crate::app::puzzles::timer::Timer;

mod list;

pub struct Riddles {
    entries: Vec<Riddle>,
    timer: Timer,
    correct_entries: usize,
    current_entry: Option<Riddle>,
    current_question: usize,
    text_area: TextArea<'static>,
    state: State,
}

pub enum State {
    Answering,
    Submit,
    Failed,
    Complete,
}

impl Riddles {
    pub fn new(timeout: Duration) -> Self {
        let mut entries = RIDDLES.to_vec();
        let mut rng = rand::rng();
        entries.shuffle(&mut rng);
        let entries = entries.into_iter().take(3).collect::<Vec<_>>();

        let mut text_area = TextArea::default();

        text_area.set_block(Block::default().title("Answer"));

        text_area.set_cursor_line_style(Style::default());

        Self {
            current_entry: entries.last().cloned(),
            entries,
            timer: Timer::new(timeout),
            correct_entries: 0,
            current_question: 0,
            text_area,
            state: State::Answering,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Riddle {
    question: &'static str,
    answer: &'static str,
}

impl IPuzzle for Riddles {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, area: ratatui::prelude::Rect) {
        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = area.layout(&layout);

        let split = Layout::vertical([Percentage(60), Percentage(40)]);

        let [question_area, answer_area] = main_area.layout(&split);
        let Some(ref entry) = self.current_entry else {
            return;
        };

        Paragraph::new(entry.question)
            .block(Block::new().title(format!("Question {}", self.current_question)))
            .render(question_area, frame.buffer_mut());

        self.text_area.render(answer_area, frame.buffer_mut());

        frame.render_widget(&mut self.timer, bottom_area);
    }

    fn instructions(&self) -> Vec<String> {
        vec![
            String::from("Welcome to the Riddled Riddles."),
            String::from("For this puzzle, you are required to answer a set of riddles."),
            String::from("You will solve three riddles."),
            String::from(
                "Get one riddle wrong three times, and the answer will be showed to you for a select period of time.",
            ),
            String::from("After three incorrect attempts, you will move to the next riddle."),
            String::from(
                "If you get two riddles wrong at the end, you will be given one more opportunity.",
            ),
            String::from("If you got one riddle wrong, you will be given two opportunities"),
            String::from(
                "However, if you get the first of the two wrong, you will fail the puzzle.",
            ),
            String::from("Three correct attempts and you pass."),
        ]
    }

    fn update(&mut self) {
        self.timer.update();
        match self.state {
            State::Answering => {
                if self.timer.done() {
                    self.state = State::Failed;
                }
            }
            State::Submit => {
                if let Some(ref riddle) = self.current_entry {
                    let answer = self.text_area.lines().concat().to_lowercase();
                    if answer.as_str().eq(riddle.answer) {
                        self.state = State::Answering;
                        self.correct_entries += 1;
                    } else {
                        if self.current_question < 3 {
                            self.state = State::Answering;
                            self.current_question += 1;
                            self.current_entry = self.entries.get(self.current_question).cloned();
                        } else {
                            // TODO: Add the five max redos failure condition
                            self.state = State::Failed;
                        }
                    }
                }
            }
            State::Failed => {}
            State::Complete => {}
        }
    }

    fn handle_events(&mut self, event: crossterm::event::Event) -> color_eyre::Result<()> {
        if let Event::Key(key) = event {
            if key.modifiers.is_empty() && key.code.is_enter() {
                self.state = State::Submit;
            }
            self.text_area.input(key);
        }

        Ok(())
    }

    fn keys_hints<'a>(&self) -> ratatui::prelude::Line<'a> {
        Line::from_iter([
            Span::styled("<C-S>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": submit  "),
        ])
    }

    fn completed(&self) -> bool {
        self.correct_entries == 3
    }

    fn failed(&self) -> bool {
        false
    }

    fn toggle_pause(&mut self, pause: bool) {
        if pause {
            self.timer.pause();
        } else {
            self.timer.unpause();
        }
    }

    fn is_paused(&self) -> bool {
        self.timer.is_paused()
    }

    fn can_pause(&self) -> bool {
        true
    }
}
