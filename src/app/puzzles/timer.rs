use std::ops::AddAssign;
use std::time::{Duration, Instant};

use ratatui::style::{Color, Style};
use ratatui::widgets::{StatefulWidget, Widget};
use throbber_widgets_tui::{CLOCK, Throbber, ThrobberState};

pub struct Timer {
    timeout: Duration,
    throbber_state: ThrobberState,
    start: Instant,
    paused_for: Duration,
    last_pause: Option<Instant>,
    last_duration: Duration,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            throbber_state: ThrobberState::default(),
            start: Instant::now(),
            paused_for: Duration::ZERO,
            last_pause: None,
            last_duration: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.throbber_state.calc_next();
        if let Some(pause) = self.last_pause {
            let now = Instant::now();
            self.paused_for.add_assign(now.duration_since(pause));
        } else if !self.done() {
            let now = Instant::now();
            self.last_duration = now.duration_since(self.start);
        }
    }

    pub fn done(&self) -> bool {
        self.last_duration >= self.timeout
    }

    pub fn pause(&mut self) {
        if self.last_pause.is_some() {
            return;
        }

        self.last_pause.replace(Instant::now());
    }

    pub const fn is_paused(&self) -> bool {
        self.last_pause.is_some()
    }

    pub const fn unpause(&mut self) {
        self.last_pause.take();
    }
}

impl Widget for &mut Timer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let throbber = Throbber::default()
            .label(format!("{:?}", self.last_pause))
            .throbber_set(CLOCK)
            .style(Style::default().fg(determine_color(self.timeout, self.last_duration)));
        <Throbber as StatefulWidget>::render(throbber, area, buf, &mut self.throbber_state);
    }
}

fn determine_color(timeout: Duration, elapsed: Duration) -> Color {
    let percentage = timeout.div_duration_f64(elapsed).clamp(0.0, 1.0) * 100.;

    match percentage {
        0_f64..=60_f64 => Color::Green,
        61_f64..=80_f64 => Color::Yellow,
        80_f64..=100_f64 => Color::Red,
        _ => unreachable!(),
    }
}
