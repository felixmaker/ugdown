use fltk::prelude::*;

use super::*;

mod ui {
    pub mod main {
        fl2rust_macro::include_ui!("./src/ui/main.fl");
    }
}

pub struct MainForm {
    ui: ui::main::UserInterface,
    add_url_dialog: AddUrlDialog,
}

impl MainForm {
    pub fn default() -> Self {
        let mut ui = ui::main::UserInterface::make_window();
        let add_url_dialog = AddUrlDialog::default();

        ui.table_parent.begin();
        let task_table = TaskTable::default();
        ui.table_parent.end();

        ui.btn_add.set_callback({
            let mut add_url_dialog = add_url_dialog.clone();
            let mut task_table = task_table.clone();
            move |_| {
                if let Some(download_info_vec) = add_url_dialog.request_download_info() {
                    task_table.add_tasks(&download_info_vec);
                }
            }
        });

        ui.btn_stop.set_callback({
            let mut task_table = task_table.clone();
            move |_| {
                task_table.stop_select();
            }
        });

        ui.btn_delete.set_callback({
            let mut task_table = task_table.clone();
            move |_| {
                task_table.remove_select();
            }
        });

        ui.btn_reload.set_callback({
            let mut task_table = task_table.clone();
            move |_| {
                task_table.reload();
            }
        });

        ui.btn_start.set_callback({
            let mut task_table = task_table.clone();
            move |_| {
                task_table.start_select();
            }
        });

        fltk::app::add_timeout3(1.0, {
            let mut task_table = task_table.clone();
            move |handle| {
                task_table.update_rows();
                fltk::app::repeat_timeout3(1.0, handle);
            }
        });

        ui.menubar.set_callback({
            let mut add_url_dialog = add_url_dialog.clone();
            let mut task_table = task_table.clone();
            move |c| match c.choice().unwrap_or("".to_owned()).as_str() {
                "Add Url" => {
                    if let Some(download_info_vec) = add_url_dialog.request_download_info() {
                        task_table.add_tasks(&download_info_vec);
                    }
                }
                "Exit" => app::quit(),
                "README.md" => {
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
                _ => {}
            }
        });

        Self { ui, add_url_dialog }
    }
}

impl StatusBar for ui::main::UserInterface {
    fn get_status_bar(&self) -> fltk::frame::Frame {
        self.label_status.clone()
    }
}
