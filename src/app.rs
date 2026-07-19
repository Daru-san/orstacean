use std::time::Duration;

use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget};
use ratatui_splash_screen::{SplashConfig, SplashScreen};
use tui_popup::Popup;
use tui_spinner::RectSpinner;

use crate::app::dashboard::{Dashboard, Stage};
use crate::app::input::InputForm;
use crate::app::puzzles::PuzzleView;

mod chat_box;
mod dashboard;
mod input;
mod puzzles;

pub static SPLASH_CONFIG: SplashConfig = SplashConfig {
    image_data: include_bytes!("../assets/rustacean-flat-gesture.png"),
    sha256sum: None,
    render_steps: 6,
    use_colors: true,
};

#[derive(Debug, PartialEq, Copy, Clone)]
enum State {
    Loading,
    Input,
    Dashboard(Stage),
    Ready,
    Quit,
    Reset,
}

struct ProgressForm {
    value: f64,
    columns: u16,
    update: bool,
    splash_screen: SplashScreen,
}

pub struct App {
    state: State,
    progress_form: ProgressForm,
    input_form: InputForm,
    confirm_state: Option<State>,
    dashboard: Dashboard,
    puzzle_view: PuzzleView,
}

impl App {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            state: State::Loading,
            progress_form: ProgressForm {
                value: 0.,
                columns: 0,
                update: true,
                splash_screen: SplashScreen::new(SPLASH_CONFIG)?,
            },
            input_form: InputForm::new(),
            confirm_state: None,
            dashboard: Dashboard::default(),
            puzzle_view: PuzzleView::new(),
        })
    }

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while self.state != State::Quit {
            terminal.draw(|frame| {
                match self.state {
                    State::Loading => frame.render_widget(&self.progress_form, frame.area()),
                    State::Input => {
                        self.input_form.draw(frame);
                    }
                    State::Dashboard(_) => {
                        self.dashboard.render(frame);
                    }
                    State::Ready => {
                        self.puzzle_view.render(frame, frame.area());
                    }
                    State::Reset => frame.render_widget("Resetting", frame.area()),
                    State::Quit => unreachable!(),
                }

                if let Some(state) = self.confirm_state {
                    match state {
                        State::Quit => {
                            match self.state {
                                State::Ready => {
                                    let mut text = Text::default();
                                    text.push_line("Are you sure you want to leave? You'll lose all your progress if you do.");
                                    text.push_line("Press Enter to quit. Press Escape to continue.");
                                    text.push_line("I recommend the latter, if you value your sanity.");
                                    let popup = Popup::new(text)
                                        .title("🦀 Ferris");
                                    frame.render_widget(popup, frame.area());
                                }
                                State::Loading => {
                                    let mut text = Text::default();
                                    text.push_line("Are you sure you want to leave? We haven't even started.");
                                    text.push_line("Press Enter to quit. Press Escape to continue.");
                                    let popup = Popup::new(text)
                                        .title("🦀 Ferris");
                                    frame.render_widget(popup, frame.area());
                                }
                                State::Input => {
                                    let mut text = Text::default();
                                    text.push_line("Are you sure you want to leave? I'm still getting to know you.");
                                    text.push_line("Press Enter to quit. Press Escape to continue.");
                                    let popup = Popup::new(text)
                                        .title("🦀 Ferris");
                                    frame.render_widget(popup, frame.area());
                                }
                                State::Dashboard(_) => {
                                    let mut text = Text::default();
                                    text.push_line("Leaving?! I'm talking to you!!");
                                    text.push_line("Press Enter to quit. Press Escape to continue.");
                                    let popup = Popup::new(text)
                                        .title("🦀 Ferris");
                                    frame.render_widget(popup, frame.area());
                                }
                                _ => unreachable!()
                            }
                        }
                        State::Reset => {
                            let popup =
                                Popup::new("Press Enter or Escape to reset").title("Confirmation");
                            frame.render_widget(popup, frame.area());
                        }
                        State::Ready => {
                            if let State::Dashboard(stage) = self.state {
                                match stage {
                                    Stage::Greeting => {
                                        let popup = Popup::new("Press Enter to continue.")
                                            .title("Conformation");
                                        frame.render_widget(popup, frame.area());
                                    }
                                    Stage::PuzzleIntro => {
                                        let popup =
                                            Popup::new("Press Enter to begin with the puzzles")
                                                .title("Conformation");
                                        frame.render_widget(popup, frame.area());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })?;
            self.handle_events()?;
            self.update(terminal.size()?.width)?;
        }
        Ok(())
    }

    fn update(&mut self, terminal_width: u16) -> color_eyre::Result<()> {
        match self.state {
            State::Loading => {
                if self.progress_form.update {
                    self.progress_form.columns =
                        (self.progress_form.columns + 1).clamp(0, terminal_width);
                    self.progress_form.value =
                        f64::from(self.progress_form.columns) / f64::from(terminal_width);
                }

                if self.progress_form.value >= 1. {
                    self.state = State::Dashboard(Stage::Greeting);
                    self.dashboard.greet();
                }
            }
            State::Input => {
                let result = self.input_form.update();
                if let Some(result) = result {
                    self.dashboard.introduce_puzzles(result.name, result.age);
                    self.state = State::Dashboard(Stage::PuzzleIntro);
                }
            }
            State::Ready => {}
            State::Dashboard(stage) => {
                if self.dashboard.update()? {
                    match stage {
                        Stage::Greeting => {
                            self.confirm_state.replace(State::Input);
                        }
                        Stage::PuzzleIntro => {
                            self.confirm_state.replace(State::Ready);
                        }
                    }
                }
            }
            State::Quit => {}
            State::Reset => {
                self.input_form = InputForm::new();
                self.state = State::Input;
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        if matches!(self.state, State::Loading) {
            let n = rand::random_range(0.1..5.0);
            let timeout = Duration::from_secs_f32(n / 50.0);
            self.progress_form.update = !event::poll(timeout)?;
            if self.progress_form.update {
                return Ok(());
            }
        }
        let mut check_state = |key: KeyEvent| -> bool {
            if let Some(state) = self.confirm_state
                && key.modifiers.is_empty()
            {
                if matches!(key.code, KeyCode::Esc) {
                    self.confirm_state.take();
                    return true;
                }
                if matches!(key.code, KeyCode::Enter) {
                    self.confirm_state.take();
                    self.state = state;
                    return true;
                }
            }
            false
        };
        if let Some(key) = event::read()?.as_key_press_event() {
            if check_state(key) {
                return Ok(());
            }
            if (matches!(key.code, KeyCode::Char('q')) || matches!(key.code, KeyCode::Char('c')))
                && key.modifiers.eq(&KeyModifiers::CONTROL)
            {
                self.confirm_state.replace(State::Quit);
                return Ok(());
            }

            if matches!(key.code, KeyCode::Char('r'))
                && key.modifiers.eq(&KeyModifiers::CONTROL)
                && !matches!(self.state, State::Loading)
            {
                self.confirm_state.replace(State::Reset);
                return Ok(());
            }
            match self.state {
                State::Loading => {}
                State::Input => {
                    self.input_form.handle_event(event::read()?);
                }
                State::Dashboard(_) => {
                    self.dashboard.handle_events()?;
                }
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
    fn render_progress(&mut self, area: Rect, buf: &mut Buffer) {
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

        Paragraph::new("Loading Oricrabby")
            .bold()
            .centered()
            .block(Block::bordered())
            .slow_blink()
            .render(line_area, buf);

        let help = Line::from(vec![
            Span::styled("Ctrl-Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
        ]);

        help.render(bottom_area, buf);

        let layout = Layout::vertical([Min(1), Percentage(100)]);
        let [gauge_area, crab_area] = main_area.layout(&layout);

        LineGauge::default()
            .filled_symbol("⣿")
            .unfilled_symbol("⣿")
            .filled_style(Style::default().fg(Color::Indexed(149)))
            .unfilled_style(Style::default().fg(Color::Indexed(58)))
            .ratio(self.value)
            .render(gauge_area, buf);

        self.splash_screen.render(crab_area, buf);
    }
}

impl Widget for &mut ProgressForm {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_progress(area, buf);
    }
}
