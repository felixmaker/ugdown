mod utils;
mod mainform;
mod add_url_dialog;
mod task_table;

use fltk::{*, prelude::*};

trait StatusBar {
    fn get_status_bar(&self) -> fltk::frame::Frame;
    fn set_status_bar_message(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_label_color(enums::Color::Blue);
        status_bar.set_label("");
        status_bar.set_label(&format!("[MESSAGE] {}", text));
        status_bar.redraw_label();
        app::redraw();
    }
    fn set_status_bar_success(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_label_color(enums::Color::Green);
        status_bar.set_label("");
        status_bar.set_label(&format!("[SUCCESS] {}", text));
        app::redraw();
    }
    fn set_status_bar_error(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_label_color(enums::Color::Red);
        status_bar.set_label("");
        status_bar.set_label(&format!("[ERROR] {}", text));
        status_bar.redraw_label();
        app::redraw();
    }
}

pub use task_table::TaskTable;
pub use add_url_dialog::AddUrlDialog;
pub use mainform::MainForm;