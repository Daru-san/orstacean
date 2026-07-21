use std::cell::LazyCell;
use std::io::{BufReader, Cursor};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::Size;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, layout::Rect, widgets::Block};
use rodio::Source;
use rodio::buffer::SamplesBuffer;
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::app::AppState;

thread_local! {
    static SAMPLE: LazyCell<SamplesBuffer> = LazyCell::new(||{
        let data = BufReader::new(Cursor::new(
            include_bytes!("../../assets/word.ogg").as_slice(),
        ));
        let decoder = rodio::Decoder::new_vorbis(data).expect("Failure");
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let samples: Vec<f32> = decoder.collect();

        SamplesBuffer::new(channels, sample_rate, samples)
    });
}

pub struct ChatBox {
    messages: Vec<String>,
    current_message: usize,
    revealed_chars: usize,
    last_tick: Instant,
    char_delay: Duration,
    msg_pause: Duration,
    pause_start: Option<Instant>,
    scroll_state: ScrollViewState,
    playback_sink: rodio::Sink,
    state: AppState,
}

fn message_height(text: &str, inner_width: u16) -> u16 {
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    paragraph.line_count(inner_width) as u16 + 2
}

impl ChatBox {
    pub fn new(text: &[String], state: AppState) -> Self {
        let playback_sink = rodio::Sink::connect_new(state.mixer());

        Self {
            messages: text.to_vec(),
            current_message: 0,
            revealed_chars: 0,
            last_tick: Instant::now(),
            char_delay: Duration::from_millis(60),
            msg_pause: Duration::from_millis(400),
            pause_start: None,
            scroll_state: ScrollViewState::new(),
            playback_sink,
            state,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let width = area.width;
        let inner_width = width.saturating_sub(2);

        let heights: Vec<u16> = self
            .messages
            .iter()
            .map(|m| message_height(m, inner_width))
            .collect();

        let total_height: u16 = heights.iter().sum();
        let mut scroll_view = ScrollView::new(Size::new(width, total_height))
            .scrollbars_visibility(tui_scrollview::ScrollbarVisibility::Automatic);

        let mut y = 0;
        for (index, (msg, height)) in self.messages.iter().zip(&heights).enumerate() {
            if index > self.current_message {
                break;
            }
            let text = if index == self.current_message && !self.done() {
                msg.chars().take(self.revealed_chars).collect::<String>()
            } else {
                msg.to_string()
            };
            let rect = Rect::new(0, y, width, *height);
            scroll_view.render_widget(
                Paragraph::new(text)
                    .block(block())
                    .wrap(Wrap { trim: false }),
                rect,
            );
            y += height;
        }

        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);
    }

    pub fn update(&mut self) -> color_eyre::Result<bool> {
        self.tick();

        self.playback_sink.set_volume(self.state.volume());

        Ok(self.done())
    }

    pub fn handle_events(&mut self, event: Event) {
        if let Some(key) = event.as_key_press_event() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.scroll_state.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_state.scroll_up(),
                KeyCode::Char('f') | KeyCode::PageDown => self.scroll_state.scroll_page_down(),
                KeyCode::Char('b') | KeyCode::PageUp => self.scroll_state.scroll_page_up(),
                KeyCode::Char('g') | KeyCode::Home => self.scroll_state.scroll_to_top(),
                KeyCode::Char('G') | KeyCode::End => self.scroll_state.scroll_to_bottom(),
                KeyCode::Char('s') if key.modifiers.eq(&KeyModifiers::CONTROL) => {
                    self.current_message = self.messages.len() + 1;
                }
                _ => (),
            }
        }
    }

    pub fn tick(&mut self) {
        if self.done() {
            return;
        }

        let Some(current) = self.messages.get(self.current_message) else {
            return;
        };

        let total_chars = current.chars().count();

        if self.revealed_chars >= total_chars {
            match self.pause_start {
                None => {
                    self.playback_sink.stop();
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

            let char = current.chars().nth(self.revealed_chars);

            if char.is_some_and(|char| char != ' ') {
                let buffer = SAMPLE.with(|samples| (*samples).clone());
                self.playback_sink.clear();
                self.playback_sink.append(buffer);
                self.playback_sink.play();
            }
        }
    }

    pub const fn done(&self) -> bool {
        self.messages.len() <= self.current_message
    }
}

fn block<'a>() -> Block<'a> {
    Block::bordered().title("🦀 King")
}
