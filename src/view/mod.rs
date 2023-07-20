mod utils;
mod mainform;
mod add_url_dialog;
mod task_table;

use fltk::{*, prelude::*};

trait StatusBar {
    fn get_status_bar(&self) -> fltk::frame::Frame;
    fn set_status_bar_message(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_label_color(enums::Color::Gray0);
        status_bar.set_label(text);
    }
    fn set_status_bar_error(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_label_color(enums::Color::Red);
        status_bar.set_label(&format!("ERROR: {}", text));
    }
}

pub use task_table::TaskTable;
pub use add_url_dialog::AddUrlDialog;
pub use mainform::MainForm;