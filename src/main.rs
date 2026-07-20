
use crate::app::App;

mod app;

pub const APP_NAME: &'static str = "Orstacean";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let result = run();

    ratatui::restore();

    result
}

pub type PlaybackCallback =
    dyn FnMut(&mut Option<rodio::Sink>, &rodio::mixer::Mixer, f32, Track) -> color_eyre::Result<()>;

#[derive(Debug, Copy, Clone)]
pub enum Track {
    Lobby,
    Struggle,
    Success,
    Failure,
}

pub fn run() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let mixer = stream_handle.mixer();

    let app = App::new(mixer.clone())?;

    app.run(&mut terminal)
}
