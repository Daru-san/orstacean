use std::time::{Duration, Instant};

use crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout, Size};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget, Wrap};
use ratatui::{Frame, layout::Rect, widgets::Block};
use tui_scrollview::{ScrollView, ScrollViewState};

const SCROLLVIEW_HEIGHT: u16 = 100;

#[derive(Debug)]
pub struct ChatBox {
    messages: Vec<String>,
    current_message: usize,
    revealed_chars: usize,
    last_tick: Instant,
    char_delay: Duration,
    msg_pause: Duration,
    pause_start: Option<Instant>,
    scroll_state: ScrollViewState,
}

impl Default for ChatBox {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            current_message: 0,
            revealed_chars: 0,
            last_tick: Instant::now(),
            char_delay: Duration::from_millis(50),
            msg_pause: Duration::from_millis(400),
            pause_start: None,
            scroll_state: ScrollViewState::new(),
        }
    }
}

impl ChatBox {
    pub fn new(text: &[String]) -> Self {
        Self {
            messages: text.to_vec(),
            current_message: 0,
            revealed_chars: 0,
            last_tick: Instant::now(),
            char_delay: Duration::from_millis(50),
            msg_pause: Duration::from_millis(400),
            pause_start: None,
            scroll_state: ScrollViewState::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let buf = frame.buffer_mut();

        let mut scroll_view = ScrollView::new(Size::new(buf.area.width, buf.area.height));

        let scroll_buf = scroll_view.buf_mut();

        let paragraphs = self.messages[..=self
            .current_message
            .min(self.messages.len().saturating_sub(1))]
            .iter()
            .enumerate()
            .map(|(index, message)| {
                let text = if index == self.current_message && !self.done() {
                    message
                        .chars()
                        .take(self.revealed_chars)
                        .collect::<String>()
                } else {
                    message.into()
                };

                Paragraph::new(text)
                    .block(block())
                    .wrap(Wrap { trim: false })
            })
            .collect::<Vec<_>>();

        let constraints = paragraphs
            .iter()
            .map(|_| Constraint::Length(3))
            .collect::<Vec<_>>();
        let chunks = Layout::vertical(constraints).split(scroll_buf.area);

        for (paragraph, chunk) in paragraphs.into_iter().zip(chunks.iter()) {
            paragraph.render(*chunk, scroll_buf);
        }

        scroll_view.render(area, buf, &mut self.scroll_state);
    }

    pub fn update(&mut self) -> color_eyre::Result<bool> {
        self.tick();

        Ok(self.done())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.scroll_state.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_state.scroll_up(),
                KeyCode::Char('f') | KeyCode::PageDown => self.scroll_state.scroll_page_down(),
                KeyCode::Char('b') | KeyCode::PageUp => self.scroll_state.scroll_page_up(),
                KeyCode::Char('g') | KeyCode::Home => self.scroll_state.scroll_to_top(),
                KeyCode::Char('G') | KeyCode::End => self.scroll_state.scroll_to_bottom(),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        if self.done() {
            return;
        };

        let Some(current) = self.messages.get(self.current_message) else {
            return;
        };

        let total_chars = current.chars().count();

        if self.revealed_chars >= total_chars {
            match self.pause_start {
                None => {
                    self.pause_start.replace(Instant::now());
                }
                Some(instant) => {
                    if instant.elapsed() >= self.msg_pause {
                        self.current_message += 1;
                        self.revealed_chars = 0;
                        self.pause_start.take();
                    }
                }
            }
            return;
        }

        if self.last_tick.elapsed() >= self.char_delay {
            self.revealed_chars += 1;
            self.last_tick = Instant::now();
        }
    }

    pub fn done(&self) -> bool {
        self.messages.len() <= self.current_message
    }
}

fn block<'a>() -> Block<'a> {
    Block::bordered().title("🦀 Ferris")
}
