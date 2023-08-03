#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod view;

use fltk::app;

#[derive(Debug)]
pub enum AppMessage {
    Finished,
}

lazy_static::lazy_static! {
    pub static ref CHANNEL:(app::Sender<AppMessage>, app::Receiver<AppMessage>) = app::channel();
}

fn main() {
    let app = app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Metro).apply();
    let mut mainform = view::MainForm::default();
    CHANNEL.0.send(crate::AppMessage::Finished);

    while app.wait() {
        if let Some(message) = CHANNEL.1.recv() {
            match message {
                AppMessage::Finished => {
                    mainform.engine_manager.tool_downloader.hide();
                }
            }
        }
    }
}
