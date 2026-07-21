use std::time::{Duration, Instant};

use ratatui::style::{Color, Style};
use ratatui::widgets::{StatefulWidget, Widget};
use throbber_widgets_tui::{CLOCK, Throbber, ThrobberState};

pub struct Timer {
    timeout: Duration,
    throbber_state: ThrobberState,
    start: Instant,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            throbber_state: ThrobberState::default(),
            start: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.throbber_state.calc_next();
    }

    pub fn done(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.start) >= self.timeout
    }
}

impl Widget for &mut Timer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start);
        let throbber = Throbber::default()
            .label(format!("{:?}", self.timeout.saturating_sub(elapsed)))
            .throbber_set(CLOCK)
            .style(Style::default().fg(determine_color(self.timeout, elapsed)));
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
