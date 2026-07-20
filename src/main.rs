use std::io::{BufReader, Cursor};


use crate::app::App;

mod app;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let result = run();

    ratatui::restore();

    result
}

pub type PlaybackCallback =
    dyn FnMut(&mut Option<rodio::Sink>, &rodio::mixer::Mixer, f32) -> color_eyre::Result<()>;

pub fn run() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let mixer = stream_handle.mixer();

    let start_playback: Box<PlaybackCallback> = Box::new(|sink, mixer, volume| {
        let reader = BufReader::new(Cursor::new(
            include_bytes!("../assets/crusty.ogg").as_slice(),
        ));

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
