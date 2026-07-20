use std::cell::Cell;
use std::rc::Rc;

use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::Layout;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::{Frame, widgets::Block};

use crate::APP_NAME;
use crate::app::chat_box::ChatBox;

pub struct Dashboard {
    chatbox: ChatBox,
    stage: Stage,
    mixer: rodio::mixer::Mixer,
    volume: Rc<Cell<f32>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Stage {
    Greeting,
    PuzzleIntro,
}

fn age_suffix(age: u8) -> &'static str {
    match age {
        0 => unreachable!(),
        1 => "st",
        2 => "nd",
        3 => "rd",
        4..=20 | 104..=120 | 204..=220 => "th",
        _ => {
            let n = age.to_string();
            let Some(last_digit) = n.chars().last() else {
                return "th";
            };

            match last_digit {
                '1' => "st",
                '2' => "nd",
                '3' => "rd",
                '0' | '4'..='9' => "th",
                _ => unreachable!(),
            }
        }
    }
}

impl Dashboard {
    pub fn new(mixer: rodio::mixer::Mixer, volume: Rc<Cell<f32>>) -> Self {
        Self {
            chatbox: ChatBox::new(&[], mixer.clone(), volume.clone()),
            mixer,
            stage: Stage::Greeting,
            volume,
        }
    }
    pub fn introduce_puzzles(&mut self, name: impl AsRef<str>, age: u8) {
        self.stage = Stage::PuzzleIntro;
        self.chatbox = ChatBox::new(
            &[
                format!("Irrashai {}.", name.as_ref()),
                String::from("Nice to meet you."),
                format!(
                    "Today, we're going to celebrate you {age}{} birthday!",
                    age_suffix(age)
                ),
                String::from("How are we going to do so, you ask?"),
                String::from("Torture of course. It's only natural."),
                String::from(
                    "You will be given a set of puzzles to complete, each one getting you closer to the end.",
                ),
                String::from("..."),
                String::from("Closer to your end that is."),
                String::from("Each puzzle will give you sufficient instructions for completion."),
                String::from("They're so simple you can't fail."),
                String::from("..."),
                String::from("If you do fail however, I'm not sure I'll be able to let it slide."),
                String::from("In any case. You should get started now. You don't have much time."),
                String::from("You will be taken to puzzle number #1."),
                String::from("準備はできたか？"),
            ],
            self.mixer.clone(),
            self.volume.clone(),
        );
    }

    pub fn greet(&mut self) {
        self.stage = Stage::Greeting;
        self.chatbox = ChatBox::new(
            &[
                format!("Welcome to {APP_NAME}."),
                String::from("My name is Ferris."),
                String::from("You may already know me as the Rust mascot."),
                String::from("As you can see, I am a crab."),
                String::from("You shall join our kind soon enough."),
                String::from("Let's start simple. Why don't you introduce yourself?"),
            ],
            self.mixer.clone(),
            self.volume.clone(),
        );
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        let layout = Layout::vertical([Length(3), Min(0)]);
        let [header_area, main_area] = area.layout(&layout);

        Paragraph::new(APP_NAME)
            .bold()
            .centered()
            .block(Block::bordered())
            .slow_blink()
            .render(header_area, buf);

        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = main_area.layout(&layout);

        let help = Line::from(vec![
            Span::styled("-", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("V: {:.2}%  ", self.volume.get() * 100.),
                Style::default()
                    .add_modifier(Modifier::UNDERLINED)
                    .fg(Color::LightBlue),
            ),
            Span::styled("+   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("Ctrl-Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("Ctrl-R", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": reset  "),
            Span::styled("H or ◄", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": left  "),
            Span::styled("J or ▲", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": up  "),
            Span::styled("K or ▼", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
            Span::styled("L or ►", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
        ]);

        help.render(bottom_area, buf);

        self.chatbox.render(frame, main_area);
    }

    pub fn update(&mut self) -> color_eyre::Result<bool> {
        self.chatbox.update()?;

        Ok(self.done())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        self.chatbox.handle_events()?;
        Ok(())
    }

    pub fn done(&self) -> bool {
        self.chatbox.done()
    }
}
