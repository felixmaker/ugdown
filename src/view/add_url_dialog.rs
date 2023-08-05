use std::{collections::HashMap, sync::Arc};

use crate::{downloader::*, send_message, AppMessage};
use fltk::{prelude::*, *};

use anyhow::{anyhow, Result};

use super::{utils::size_to_string, MainFormMessage, StatusBar};

mod add_url_dialog {
    fl2rust_macro::include_ui!("./src/ui/add_url.fl");
}

#[derive(Clone)]
pub struct AddUrlDialog {
    add_url_dialog: add_url_dialog::UserInterface,
    current_idx: HashMap<i32, DownloadInfo>,
    current_cookies: Option<String>,
}

impl AddUrlDialog {
    pub fn default() -> Self {
        let mut add_url_dialog = add_url_dialog::UserInterface::make_window();

        if let Some(download_dir) = directories::UserDirs::new()
            .and_then(|x| x.download_dir().map(|p| p.to_string_lossy().to_string()))
        {
            add_url_dialog.input_dir.set_value(&download_dir);
        }

        let current_idx: HashMap<i32, DownloadInfo> = Default::default();
        let current_cookies: Option<String> = Default::default();

        add_url_dialog
            .choice_engine
            .add_choice(get_engine_names().join("|").as_str());
        add_url_dialog.choice_engine.set_value(0);

        let mut result = Self {
            add_url_dialog,
            current_idx,
            current_cookies,
        };

        result.bind_message();

        result
    }

    fn bind_message(&mut self) {
        self.add_url_dialog
            .btn_detect
            .set_callback(|_| send_message(AddUrlDialogMessage::Detect));
        self.add_url_dialog
            .btn_submit
            .set_callback(|_| send_message(AddUrlDialogMessage::Submit));

        self.add_url_dialog
            .btn_cancel
            .set_callback(|_| send_message(AddUrlDialogMessage::Hide));

        self.add_url_dialog
            .btn_select_dir
            .set_callback(|_| send_message(AddUrlDialogMessage::SelectDir));

        self.add_url_dialog
            .check_all
            .set_callback(|_| send_message(AddUrlDialogMessage::CheckAll));

        self.add_url_dialog
            .btn_reset
            .set_callback(|_| send_message(AddUrlDialogMessage::Reset));

        self.add_url_dialog
            .btn_set_cookie
            .set_callback(|_| send_message(AddUrlDialogMessage::SetCookies));
    }

    fn detect(&mut self) -> Result<()> {
        let url = self.add_url_dialog.input_url.value().trim().to_string();
        if url.len() == 0 {
            return Err(anyhow!("Url is empty!"));
        }

        let engine = self
            .add_url_dialog
            .choice_engine
            .choice()
            .ok_or_else(|| anyhow!("Please select engine"))?;

        self.add_url_dialog.btn_detect.deactivate();
        let current_cookies = self.current_cookies.clone();
        std::thread::spawn(move || {
            let cookie_file = current_cookies.and_then(|x| store_cookies(&x).ok());
            if let Ok(stream_info) = get_stream_info(&engine, &url, cookie_file.as_deref()) {
                send_message(AddUrlDialogMessage::UpdateInfo(Arc::new(stream_info)))
                // }
                // Err(err) => {
                //     // println!("{}", err);
                //     // add_url_dialog.set_status_bar_error("Failed to detect this url!");
                //     // add_url_dialog.btn_detect.activate()
                // }
            }

            if let Some(cookie_file) = cookie_file {
                let _ = std::fs::remove_file(cookie_file);
            }
        });

        Ok(())
    }

    fn update_with_stream_info(&mut self, stream_info: &HashMap<String, DownloadInfo>) {
        self.current_idx.clear();

        let mut title_updated = false;
        self.add_url_dialog.checkbrowser.clear();

        for (_id, info) in stream_info {
            if title_updated == false {
                self.add_url_dialog.output_title.set_value(&info.title);
                title_updated = true;
            }

            let check_item = format!(
                "{} - {} (size: {})",
                info.ext,
                info.stream_name,
                size_to_string(info.stream_size)
            );
            let idx = self
                .add_url_dialog
                .checkbrowser
                .add(check_item.as_str(), false);

            self.current_idx.insert(idx, info.clone());

            self.add_url_dialog.checkbrowser.redraw();
        }

        if self.add_url_dialog.btn_detect.active() == false {
            self.add_url_dialog.btn_detect.activate()
        }
        self.add_url_dialog
            .set_status_bar_success("Detected successfully!");
    }

    fn set_cookies(&mut self) {
        let current_cookies = self.current_cookies.take().unwrap_or("".to_owned());
        if let Some(cookies) = dialog::input_default("Input cookies below:", &current_cookies) {
            self.current_cookies = Some(cookies);
            self.add_url_dialog
                .set_status_bar_success("Cookie has changed!")
        }
    }

    fn reset(&mut self) {
        self.add_url_dialog.input_url.set_value("");
        self.add_url_dialog.output_title.set_value("");
        self.add_url_dialog.checkbrowser.clear();
    }

    fn check_all(&mut self) {
        if self.add_url_dialog.check_all.is_checked() {
            self.add_url_dialog.checkbrowser.check_all();
        } else {
            self.add_url_dialog.checkbrowser.check_none();
        }
    }

    fn select_dir(&mut self) {
        if let Some(dir) = dialog::dir_chooser("Choose dir to save download file", "", false) {
            self.add_url_dialog.input_dir.set_value(&dir);
            self.add_url_dialog
                .set_status_bar_message(&format!("Set ouput dir to {}", dir))
        }
    }

    fn submit(&mut self) {
        let mut current_task: Vec<DownloadInfo> = Vec::new();

        let save_dir = self.add_url_dialog.input_dir.value();
        let save_dir_path = std::path::Path::new(&save_dir);
        if save_dir_path.is_dir() == false {
            self.add_url_dialog
                .set_status_bar_error("You need to set output dir!");
            return;
        }

        for i in 1..=self.add_url_dialog.checkbrowser.nitems() as i32 {
            if self.add_url_dialog.checkbrowser.checked(i) {
                if let Some(info) = self.current_idx.get(&i) {
                    let mut info = info.to_owned();
                    info.save_option = Some(SaveOption {
                        output_dir: save_dir.clone(),
                        file_name: format!("{}[{}]", info.title, info.stream_name),
                    });
                    current_task.push(info);
                }
            }
        }

        let length = current_task.len();

        if length > 0 {
            send_message(MainFormMessage::AddTask(Arc::new(current_task)));
            self.add_url_dialog.set_status_bar_success("Task add");
        }

        self.reset();
        self.add_url_dialog.window.hide();
    }

    pub fn handle_message(&mut self, message: AddUrlDialogMessage) {
        match message {
            AddUrlDialogMessage::UpdateInfo(stream_info) => {
                self.update_with_stream_info(&*stream_info)
            }
            AddUrlDialogMessage::Detect => {
                if let Err(error) = self.detect() {
                    self.add_url_dialog.set_status_bar_error(&error.to_string());
                }
            }
            AddUrlDialogMessage::Show => self.add_url_dialog.window.show(),
            AddUrlDialogMessage::Hide => self.add_url_dialog.window.hide(),
            AddUrlDialogMessage::Submit => self.submit(),
            AddUrlDialogMessage::SelectDir => self.select_dir(),
            AddUrlDialogMessage::CheckAll => self.check_all(),
            AddUrlDialogMessage::Reset => self.reset(),
            AddUrlDialogMessage::SetCookies => self.set_cookies(),
        }
    }
}

#[derive(Clone)]
pub enum AddUrlDialogMessage {
    UpdateInfo(Arc<HashMap<String, DownloadInfo>>),
    Submit,
    Detect,
    SelectDir,
    Show,
    Hide,
    CheckAll,
    Reset,
    SetCookies,
}

impl From<AddUrlDialogMessage> for AppMessage {
    fn from(value: AddUrlDialogMessage) -> Self {
        AppMessage::AddUrlDialog(value)
    }
}

impl StatusBar for add_url_dialog::UserInterface {
    fn get_status_bar(&self) -> output::Output {
        self.output_status.clone()
    }
}
