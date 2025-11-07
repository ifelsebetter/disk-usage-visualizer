use iced::{Application, Settings};
mod app;

fn main() -> iced::Result {
    app::DiskVisualizer::run(Settings::default())
}

