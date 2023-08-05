mod add_url_dialog;
mod mainform;
mod task_table;
mod tool_downloader;
mod utils;
mod engine_manager;

use fltk::{prelude::*, *};

trait StatusBar {
    fn get_status_bar(&self) -> output::Output;
    fn set_status_bar_message(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Blue);
        status_bar.set_value(&format!("[MESSAGE] {}", text));
    }
    fn set_status_bar_success(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Green);
        status_bar.set_value(&format!("[SUCCESS] {}", text));
    }
    fn set_status_bar_error(&self, text: &str) {
        let mut status_bar = self.get_status_bar();
        status_bar.set_text_color(enums::Color::Red);
        status_bar.set_value(&format!("[ERROR] {}", text));
    }
}

pub use add_url_dialog::{AddUrlDialog, AddUrlDialogMessage};
pub use mainform::{MainForm, MainFormMessage};
pub use tool_downloader::{ToolDownloader, ToolDownloaderMessage};
pub use engine_manager::{EngineManager, EngineManagerMessage};
pub use task_table::TaskTable;
