use std::sync::Arc;

use fltk::prelude::*;

use crate::{downloader::DownloadInfo, send_message, AppMessage};

use super::*;

mod mainform_ui {
    fl2rust_macro::include_ui!("./src/ui/main.fl");
}

use mainform_ui::UserInterface;

pub struct MainForm {
    ui: UserInterface,
    task_table: TaskTable,
}

impl MainForm {
    pub fn default() -> Self {
        let mut ui = UserInterface::make_window();

        let task_table = TaskTable::default();
        ui.table_parent.add_resizable(&**task_table);
        let task_table = task_table.size_of_parent().center_of_parent();

        fltk::app::add_timeout3(1.0, {
            let mut task_table = task_table.clone();
            move |handle| {
                task_table.update_rows();
                fltk::app::repeat_timeout3(1.0, handle);
            }
        });

        let mut result = Self { ui, task_table };

        result.bind_message();

        return result;
    }

    fn bind_message(&mut self) {
        self.ui
            .btn_add
            .set_callback(|_| send_message(AddUrlDialogMessage::Show));

        self.ui
            .btn_stop
            .set_callback(|_| send_message(MainFormMessage::StopTask));

        self.ui
            .btn_delete
            .set_callback(|_| send_message(MainFormMessage::DeleteTask));

        self.ui
            .btn_start
            .set_callback(|_| send_message(MainFormMessage::StartTask));

        self.ui
            .btn_reload
            .set_callback(|_| send_message(MainFormMessage::ReloadTask));

        self.ui.menubar.set_callback(
            move |c| match c.choice().unwrap_or("".to_owned()).as_str() {
                "Add Url" => send_message(AddUrlDialogMessage::Show),
                "Exit" => app::quit(),
                "README.md" => send_message(MainFormMessage::ShowReadme),
                "About" => send_message(MainFormMessage::ShowVersion),
                "Engine Manager" => send_message(EngineManagerMessage::Show),
                _ => {}
            },
        );
    }

    fn check_task(&mut self, count: usize, success: &str, failure: &str) {
        if count > 0 {
            self.ui.set_status_bar_success(success);
        } else if failure.trim().len() > 0 {
            self.ui.set_status_bar_error(failure);
        }
    }

    fn add_task(&mut self, download_info: &Vec<DownloadInfo>) {
        let count = self.task_table.add_tasks(&download_info);
        self.check_task(
            count,
            &format!("{} task(s) added to task table!", count),
            "No tasks are added to task table!",
        );
    }

    fn start_task(&mut self) {
        if let Ok(count) = self.task_table.start_select() {
            self.check_task(
                count,
                &format!("The selected {} task(s) started.", count),
                "",
            );
        }
    }

    fn stop_task(&mut self) {
        if let Ok(count) = self.task_table.stop_select() {
            self.check_task(
                count,
                &format!("The selected {} task(s) stopped.", count),
                "",
            );
        }
    }

    fn delete_task(&mut self) {
        if let Ok(count) = self.task_table.remove_select() {
            self.check_task(
                count,
                &format!("The selected {} task(s) removed.", count),
                "",
            );
        }
    }

    fn reload_task(&mut self) {
        self.task_table.reload();
        self.ui.set_status_bar_message("Task table reloaded.");
    }

    pub fn handle_message(&mut self, message: MainFormMessage) {
        match message {
            MainFormMessage::AddTask(info) => self.add_task(&*info),
            MainFormMessage::StartTask => self.start_task(),
            MainFormMessage::StopTask => self.stop_task(),
            MainFormMessage::DeleteTask => self.delete_task(),
            MainFormMessage::ReloadTask => self.reload_task(),
            MainFormMessage::ShowReadme => show_readme(),
            MainFormMessage::ShowVersion => show_about(),
        }
    }
}

fn show_readme() {
    let mut help = dialog::HelpDialog::default();
    help.set_text_size(18);
    let readme = include_str!("../../README.md");
    let mut result = Vec::new();
    for line in readme.lines() {
        if line.trim().len() > 0 {
            result.push(format!("<p>{}</p>", line));
        }
    }
    help.set_value(&result.join("\n"));
    help.show();
    while help.shown() {
        app::wait();
    }
}

fn show_about() {
    dialog::message_title("About");
    dialog::message_default(&format!(
        "UgDown\nVersion: {}\nOS: {}({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    ));
}

#[derive(Clone)]
pub enum MainFormMessage {
    AddTask(Arc<Vec<DownloadInfo>>),
    StartTask,
    StopTask,
    DeleteTask,
    ReloadTask,
    ShowReadme,
    ShowVersion,
}

impl From<MainFormMessage> for AppMessage {
    fn from(value: MainFormMessage) -> Self {
        AppMessage::MainForm(value)
    }
}

impl StatusBar for UserInterface {
    fn get_status_bar(&self) -> output::Output {
        self.output_status.clone()
    }
}
