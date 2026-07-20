use std::io::{BufReader, Cursor};

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

    let start_playback: Box<PlaybackCallback> = Box::new(|sink, mixer, volume, track| {
        let reader = BufReader::new(Cursor::new(match track {
            Track::Lobby => include_bytes!("../assets/slow-walk.ogg").as_slice(),
            Track::Struggle => include_bytes!("../assets/anticipation.ogg").as_slice(),
            Track::Success => include_bytes!("../assets/satisfactory.ogg").as_slice(),
            Track::Failure => include_bytes!("../assets/reconsider.ogg").as_slice(),
        }));

        match sink {
            Some(sink) => {
                *sink = rodio::play(mixer, reader)?;
                sink.set_volume(volume);
            }
            None => {
                sink.replace(rodio::play(mixer, reader)?);
                if let Some(sink) = sink {
                    sink.set_volume(volume);
                }
            }
        }

        Ok(())
    });

    let app = App::new(start_playback, mixer.clone())?;

    app.run(&mut terminal)
}
