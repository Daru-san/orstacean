use ratatui::{style::Style, text::Line, widgets::Block};
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
    fn can_pause(&self) -> bool {
        false
    }
    fn completed(&self) -> bool {
        false
    }
    fn failed(&self) -> bool {
        false
    }
    fn failure_message(&self) -> Vec<String> {
        vec![]
    }
    fn handle_events(&mut self, _event: crossterm::event::Event) -> color_eyre::Result<()> {
        Ok(())
    }
    fn instructions(&self) -> Vec<String> {
        vec![]
    }
    fn is_paused(&self) -> bool {
        false
    }
    fn keys_hints<'a>(&self) -> ratatui::prelude::Line<'a> {
        Line::default()
    }
    fn render(&mut self, _frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
    fn toggle_pause(&mut self, _pause: bool) {}
}
