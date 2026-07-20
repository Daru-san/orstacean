use std::cell::{Cell, LazyCell};
use std::io::{BufReader, Cursor};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crossterm::event::{self, KeyCode};
use ratatui::layout::Size;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, layout::Rect, widgets::Block};
use rodio::Source;
use rodio::buffer::SamplesBuffer;
use tui_scrollview::{ScrollView, ScrollViewState};

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
    volume: Rc<Cell<f32>>,
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
    pub fn new(text: &[String], mixer: rodio::mixer::Mixer, volume: Rc<Cell<f32>>) -> Self {
        let playback_sink = rodio::Sink::connect_new(&mixer);

        Self {
            messages: text.to_vec(),
            current_message: 0,
            revealed_chars: 0,
            last_tick: Instant::now(),
            char_delay: Duration::from_millis(50),
            msg_pause: Duration::from_millis(400),
            pause_start: None,
            scroll_state: ScrollViewState::new(),
            playback_sink,
            volume,
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
        self.tick()?;

        self.playback_sink.set_volume(self.volume.get());

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

    pub fn tick(&mut self) -> color_eyre::Result<()> {
        if self.done() {
            return Ok(());
        }

        let Some(current) = self.messages.get(self.current_message) else {
            return Ok(());
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
            return Ok(());
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

    pub fn done(&self) -> bool {
        self.messages.len() <= self.current_message
    }
}

fn block<'a>() -> Block<'a> {
    Block::bordered().title("🦀 Ferris")
}
