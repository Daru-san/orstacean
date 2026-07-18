use ratatui::{Frame, layout::Rect, widgets::Block};

pub struct Dashboard {
    page: usize,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self { page: 0 }
    }
}

impl Dashboard {
    pub fn render(_frame: &mut Frame) {}
}

fn render_border(frame: &mut Frame<'_>, area: Rect) {
    let block = Block::bordered().title("Ferris");
    frame.render_widget(block, area);
}
