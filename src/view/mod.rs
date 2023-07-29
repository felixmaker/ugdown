mod add_url_dialog;
mod mainform;
mod task_table;
mod tool_downloader;
mod utils;

use fltk::{prelude::*, *};

trait StatusBar {
    fn get_status_bar(&self) -> output::Output;
    fn set_status_bar_message(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Blue);
        status_bar.set_value(&format!("[MESSAGE] {}", text));
        app::redraw();
    }
    fn set_status_bar_success(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Green);
        status_bar.set_value(&format!("[SUCCESS] {}", text));
        app::redraw();
    }
    fn set_status_bar_error(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Red);
        status_bar.set_value(&format!("[ERROR] {}", text));
        app::redraw();
    }
}

pub use add_url_dialog::AddUrlDialog;
pub use mainform::MainForm;
pub use task_table::TaskTable;
