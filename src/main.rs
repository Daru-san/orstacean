use crate::app::App;

mod app;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app = App::new()?;
    let result = app.run(&mut terminal);
    ratatui::restore();

    result
}
