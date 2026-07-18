use std::time::Duration;

use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget};
use tui_spinner::RectSpinner;

use crate::app::input::InputForm;

mod dashboard;
mod input;
mod puzzles;

#[derive(Debug, PartialEq)]
enum State {
    Loading,
    Input,
    Welcome,
    Ready,
    Quit,
    Reset,
}

struct ProgressForm {
    value: f64,
    columns: u16,
}

pub struct App {
    state: State,
    progress_form: ProgressForm,
    input_form: InputForm,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: State::Loading,
            progress_form: ProgressForm {
                value: 0.,
                columns: 0,
            },
            input_form: InputForm::new(),
        }
    }
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while self.state != State::Quit {
            terminal.draw(|frame| match self.state {
                State::Loading => frame.render_widget(&self.progress_form, frame.area()),
                State::Input => {
                    self.input_form.draw(frame);
                }
                State::Welcome => {}
                State::Ready => {}
                State::Reset => frame.render_widget("Resetting", frame.area()),
                State::Quit => unreachable!(),
            })?;
            self.handle_events()?;
            self.update(terminal.size()?.width);
        }
        Ok(())
    }

    fn update(&mut self, terminal_width: u16) {
        match self.state {
            State::Loading => {
                self.progress_form.columns =
                    (self.progress_form.columns + 1).clamp(0, terminal_width);
                self.progress_form.value =
                    f64::from(self.progress_form.columns) / f64::from(terminal_width);

                if self.progress_form.value >= 1. {
                    self.state = State::Input;
                }
            }
            State::Input => {
                let result = self.input_form.update();
                if let Some(result) = result {
                    self.state = State::Ready;
                }
            }
            State::Ready => {}
            State::Welcome => {}
            State::Quit => {}
            State::Reset => {
                self.input_form = InputForm::new();
                self.state = State::Input;
            }
        }
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        if matches!(self.state, State::Loading) {
            let n = rand::random_range(0.1..5.0);
            let timeout = Duration::from_secs_f32(n / 50.0);
            if !event::poll(timeout)? {
                return Ok(());
            }
        }
        if let Some(key) = event::read()?.as_key_press_event() {
            if (matches!(key.code, KeyCode::Char('q')) || matches!(key.code, KeyCode::Char('c')))
                && key.modifiers.eq(&KeyModifiers::CONTROL)
            {
                self.state = State::Quit;
                return Ok(());
            }

            if matches!(key.code, KeyCode::Char('r'))
                && key.modifiers.eq(&KeyModifiers::CONTROL)
                && !matches!(self.state, State::Loading)
            {
                self.reset();
                return Ok(());
            }
            match self.state {
                State::Loading => {}
                State::Input => {
                    self.input_form.handle_event(event::read()?);
                }
                State::Welcome => {}
                State::Quit => {}
                State::Reset => {}
                State::Ready => {}
            }
        }

        Ok(())
    }

    const fn reset(&mut self) {
        self.progress_form.value = 0.0;
        self.progress_form.columns = 0;
        self.state = State::Reset;
    }
}

impl ProgressForm {
    fn render_progress(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Length(3), Min(0)]);
        let [header_area, main_area] = area.layout(&layout);

        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = main_area.layout(&layout);

        let header_layout = Layout::horizontal([Length(3), Min(0)]);

        let [spinner_area, line_area] = header_area.layout(&header_layout);

        RectSpinner::new(((self.value * 100.0) as u64).clamp(0, 100))
            .spin(tui_spinner::Spin::Clockwise)
            .outer_color(Color::Cyan)
            .render(spinner_area, buf);

        Paragraph::new("Loading Origin.Crab")
            .bold()
            .centered()
            .block(Block::bordered())
            .slow_blink()
            .render(line_area, buf);

        Paragraph::new("(Press 'ctrl+q' or 'ctrl+c' to quit)")
            .centered()
            .render(bottom_area, buf);

        LineGauge::default()
            .filled_symbol("⣿")
            .unfilled_symbol("⣿")
            .filled_style(Style::default().fg(Color::Indexed(149)))
            .unfilled_style(Style::default().fg(Color::Indexed(58)))
            .ratio(self.value)
            .render(main_area, buf);
    }
}

impl Widget for &ProgressForm {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_progress(area, buf);
    }
}
