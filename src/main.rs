mod downloader;
mod view;

mod alert {
    fn alert_default() {

    }
}

fn main() {
    let app = fltk::app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Metro).apply();
    let _mainform = view::MainForm::default();
    while app.wait() {
        
    };
}
