use ratatui::{style::Style, widgets::Block};
use ratatui_textarea::TextArea;

use crate::app::puzzles::IPuzzle;

pub struct Cipher {
    word: String,
    text_area: TextArea<'static>,
}

impl Cipher {
    pub fn new(word: String) -> Cipher {
        let mut text_area = TextArea::default();

        text_area.set_block(Block::default().title("Answer"));

        text_area.set_cursor_line_style(Style::default());

        Self { word, text_area }
    }
}

impl IPuzzle for Cipher {
    fn update(&mut self) {}
}
