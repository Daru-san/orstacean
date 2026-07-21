use std::time::Duration;

use rand::seq::SliceRandom;
use ratatui::layout::Constraint::{Min, Percentage};
use ratatui::layout::Layout;
use ratatui::text::Line;
use rodio::cpal::FromSample;

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::riddles::list::RIDDLES;
use crate::app::puzzles::timer::Timer;

mod list;

pub struct Riddles {
    entries: Vec<Riddle>,
    timer: Timer,
}

impl Riddles {
    pub fn new(timeout: Duration) -> Self {
        let mut entries = RIDDLES.to_vec();
        let mut rng = rand::rng();
        entries.shuffle(&mut rng);
        let entries = entries.into_iter().take(3).collect();

        Self {
            entries,
            timer: Timer::new(timeout),
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

    fn update(&mut self) {}

    fn handle_events(&mut self, event: crossterm::event::Event) -> color_eyre::Result<()> {
        Ok(())
    }

    fn keys_hints<'a>(&self) -> ratatui::prelude::Line<'a> {
        Line::from_iter([String::from("Hello")])
    }

    fn completed(&self) -> bool {
        false
    }

    fn failed(&self) -> bool {
        false
    }
}
