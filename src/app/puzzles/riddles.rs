use std::time::Duration;

use rand::seq::SliceRandom;

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::riddles::list::RIDDLES;
use crate::app::puzzles::timer::Timer;

mod list;

pub struct Riddles {
    entries: Vec<Riddle>,
    timer: Timer,
}

impl Riddles {
    pub fn new(timeout: Duration) -> Riddles {
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

impl IPuzzle for Riddles {}
