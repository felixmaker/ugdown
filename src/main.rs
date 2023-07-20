#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod view;

fn main() {
    let app = fltk::app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Metro).apply();
    let _mainform = view::MainForm::default();
    app.run().unwrap();
}
