use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::downloader::*;
use fltk::{prelude::*, *};

use super::{utils::size_to_string, StatusBar};

mod add_url_dialog {
    fl2rust_macro::include_ui!("./src/ui/add_url.fl");
}

#[derive(Clone)]
pub struct AddUrlDialog {
    current_select: Rc<RefCell<Option<Vec<DownloadInfo>>>>,
    ui: add_url_dialog::UserInterface,
}

impl AddUrlDialog {
    pub fn default() -> Self {
        let mut ui = add_url_dialog::UserInterface::make_window();
        let current_idx: Arc<Mutex<HashMap<i32, DownloadInfo>>> = Default::default();
        let current_select: Rc<RefCell<Option<Vec<DownloadInfo>>>> = Default::default();

        ui.choice_engine
            .add_choice(get_engine_names().join("|").as_str());
        ui.choice_engine.set_value(0);

        ui.btn_detect.set_callback({
            let current_idx = current_idx.clone();
            let mut add_url_dialog = ui.clone();
            move |_| {
                let url = add_url_dialog.input_url.value().trim().to_string();
                if url.len() > 0 {
                    if let Some(engine) = add_url_dialog.choice_engine.choice() {
                        add_url_dialog.btn_detect.deactivate();

                        std::thread::spawn({
                            let current_idx = current_idx.clone();
                            let mut add_url_dialog = add_url_dialog.clone();
                            move || match get_stream_info(&engine, &url) {
                                Ok(info_map) => {
                                    let mut current_idx = current_idx.lock().unwrap();
                                    current_idx.clear();

                                    let mut title_updated = false;
                                    add_url_dialog.checkbrowser.clear();

                                    for (_id, info) in info_map {
                                        if title_updated == false {
                                            add_url_dialog.output_title.set_value(&info.title);
                                            title_updated = true;
                                        }

                                        let check_item = format!(
                                            "{} - {} (size: {})",
                                            info.ext,
                                            info.stream_name,
                                            size_to_string(info.stream_size)
                                        );
                                        let idx = add_url_dialog
                                            .checkbrowser
                                            .add(check_item.as_str(), false);
                                        current_idx.insert(idx, info);

                                        add_url_dialog.checkbrowser.redraw();
                                    }

                                    if add_url_dialog.btn_detect.active() == false {
                                        add_url_dialog.btn_detect.activate()
                                    }
                                }
                                Err(_) => {
                                    add_url_dialog.set_status_bar_error("Failed to detect this url!");
                                    add_url_dialog.btn_detect.activate()
                                },
                            }
                        });
                    }
                }
            }
        });

        ui.btn_submit.set_callback({
            let mut add_url_dialog = ui.clone();
            let current_select = current_select.clone();
            let current_idx = current_idx.clone();
            move |_| {
                let current_idx = current_idx.lock().unwrap();
                let mut current_task: Vec<DownloadInfo> = Vec::new();

                let save_dir = add_url_dialog.input_dir.value();
                let save_dir_path = std::path::Path::new(&save_dir);
                if save_dir_path.is_dir() {
                    for i in 1..=add_url_dialog.checkbrowser.nitems() as i32 {
                        if add_url_dialog.checkbrowser.checked(i) {
                            if let Some(info) = current_idx.get(&i) {
                                let mut info = info.to_owned();
                                info.save_option = Some(SaveOption {
                                    output_dir: save_dir.clone(),
                                    file_name: format!(
                                        "{}.{}.{}",
                                        info.title, info.stream_name, info.ext
                                    ),
                                });
                                current_task.push(info);
                            }
                        }
                    }

                    *current_select.borrow_mut() = Some(current_task);
                    add_url_dialog.window.hide();
                }
            }
        });

        ui.btn_cancel.set_callback({
            let mut add_url_dialog = ui.clone();
            move |_| add_url_dialog.window.hide()
        });

        ui.btn_select_dir.set_callback({
            let mut add_url_dialog = ui.clone();
            move |_| {
                if let Some(dir) =
                    dialog::dir_chooser("Choose dir to save download file", "", false)
                {
                    add_url_dialog.input_dir.set_value(&dir);
                }
            }
        });

        ui.check_all.set_callback({
            let mut add_url_dialog = ui.clone();
            move |_| {
                if add_url_dialog.check_all.is_checked() {
                    add_url_dialog.checkbrowser.check_all();
                } else {
                    add_url_dialog.checkbrowser.check_none();
                }
            }
        });

        Self { current_select, ui }
    }

    pub fn request_download_info(&mut self) -> Option<Vec<DownloadInfo>> {
        self.ui.window.show();
        while self.ui.window.shown() {
            app::wait();
        }
        self.current_select.borrow_mut().take()
    }
}

impl StatusBar for add_url_dialog::UserInterface {
    fn get_status_bar(&self) -> fltk::frame::Frame {
        self.label_status.clone()
    }
}
