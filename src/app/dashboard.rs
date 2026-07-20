use std::fmt::Display;

use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::Layout;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};
use ratatui::{Frame, widgets::Block};

use crate::app::chat_box::ChatBox;

const SCROLLVIEW_HEIGHT: u16 = 100;

#[derive(Debug)]
pub struct Dashboard {
    chatbox: ChatBox,
    stage: Stage,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Stage {
    Greeting,
    PuzzleIntro,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            chatbox: ChatBox::default(),
            stage: Stage::Greeting,
        }
    }
}

impl Dashboard {
    pub fn introduce_puzzles(&mut self, name: impl Into<String> + Display, age: u8) {
        self.stage = Stage::PuzzleIntro;
        self.chatbox = ChatBox::new(&[
            format!("Irrashai {name}. Good to meet you."),
            format!("Today, we're going to celebrate you {age}th birthday!"),
            String::from("How are we going to do so, you ask?"),
            String::from("Torture of course. It's only natural."),
            String::from(
                "You will be given a set of puzzles to complete, each one getting you closer to the end... your end that is.",
            ),
            String::from(
                "Each puzzle will give you sufficient instructions for completion. They're so simple you can't fail.",
            ),
            String::from("If you do fail however, I'm not sure I'll be able to let it slide."),
            String::from("In any case. You should get started now. You don't have much time."),
            String::from("You will be taken to puzzle number #1."),
            String::from("準備はできたか？"),
        ]);
    }

    pub fn greet(&mut self) {
        self.stage = Stage::Greeting;
        self.chatbox = ChatBox::new(&[
            String::from("Welcome to Oricrabby."),
            String::from("My name is Ferris."),
            String::from("You may already know me as the Rust mascot."),
            String::from("As you can see, I am a crab."),
            String::from(
                "You shall join our kind soon enough. That however, is a story for another day.",
            ),
            String::from("Let's start simple. Why don't you introduce yourself?"),
        ]);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        let layout = Layout::vertical([Length(3), Min(0)]);
        let [header_area, main_area] = area.layout(&layout);

        Paragraph::new("Oricrabby")
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
