use crossterm::event::Event;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::Layout;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::{Frame, widgets::Block};

use crate::APP_NAME;
use crate::app::AppState;
use crate::app::chat_box::ChatBox;

pub struct Dashboard {
    chatbox: ChatBox,
    stage: Stage,
    state: AppState,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
    pub fn new(state: AppState) -> Self {
        Self {
            chatbox: ChatBox::new(&[], state.clone()),
            stage: Stage::Greeting,
            state,
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
            self.state.clone(),
        );
    }

    pub fn greet(&mut self) {
        self.stage = Stage::Greeting;
        self.chatbox = ChatBox::new(
            &[
                format!("Welcome to {APP_NAME}."),
                String::from("My name is King."),
                String::from("As you can see, I am a crab."),
                String::from("You shall join our kind soon enough."),
                String::from("..."),
                String::from("That however, is a story for another day."),
                String::from("..."),
                String::from("Enough about me."),
                String::from("Why don't you introduce yourself?"),
            ],
            self.state.clone(),
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
            .render(header_area, buf);

        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = main_area.layout(&layout);

        let mut parts = Vec::from_iter(self.state.volume_hints());
        parts.extend([
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

        let help = Line::from_iter(parts);

        help.render(bottom_area, buf);

        self.chatbox.render(frame, main_area);
    }

    pub fn update(&mut self) -> color_eyre::Result<bool> {
        self.chatbox.update()?;

        Ok(self.done())
    }

    pub fn handle_events(&mut self, event: Event) -> color_eyre::Result<()> {
        self.chatbox.handle_events(event)?;
        Ok(())
    }

    pub fn done(&self) -> bool {
        self.chatbox.done()
    }
}
