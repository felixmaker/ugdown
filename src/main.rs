#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod view;

use fltk::app::{channel, App, Receiver, Sender};
use fltk_theme::{ThemeType, WidgetTheme};
use view::*;

lazy_static::lazy_static! {
    pub static ref CHANNEL: (Sender<AppMessage>, Receiver<AppMessage>) = channel();
}

#[derive(Clone)]
pub enum AppMessage {
    MainForm(MainFormMessage),
    AddUrlDialog(AddUrlDialogMessage),
    EngineManager(EngineManagerMessage),
    ToolDownloader(ToolDownloaderMessage)
}

pub fn send_message<T>(message: T)
where
    T: Into<AppMessage>,
{
    CHANNEL.0.clone().send(message.into())
}

fn main() {
    let app = App::default();
    let mut mainform = MainForm::default();
    let mut add_url_dialog = AddUrlDialog::default();
    let mut engine_manager = EngineManager::default();
    let mut tool_downloader = ToolDownloader::default();

    let widget_theme = WidgetTheme::new(ThemeType::Metro);
    widget_theme.apply();

    while app.wait() {
        if let Some(message) = CHANNEL.1.recv() {
            match message {
                AppMessage::MainForm(message) => mainform.handle_message(message),
                AppMessage::AddUrlDialog(message) => add_url_dialog.handle_message(message),
                AppMessage::EngineManager(message) => engine_manager.handle_message(message),
                AppMessage::ToolDownloader(message) => tool_downloader.handle_message(message)
            }
        }
    }
}
