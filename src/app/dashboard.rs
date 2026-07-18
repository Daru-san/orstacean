use ratatui::Frame;

pub struct Dashboard {
    page: usize,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self { page: 0 }
    }
}

impl Dashboard {
    pub fn render(_frame: &mut Frame) {

    }
}
