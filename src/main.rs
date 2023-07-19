mod downloader;
mod view;

mod ui {
    pub mod main {
        fl2rust_macro::include_ui!("./src/ui/main.fl");
    }

    pub mod add_url_dialog {
        fl2rust_macro::include_ui!("./src/ui/add_url.fl");
    }
}

use std::{
    cell::Cell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use downloader::*;
use fltk::{prelude::*, *};
use uuid::Uuid;

#[derive(Clone)]
pub enum Message {
    FromAddUrlDialog(AxgMessage),
    FromMainForm(MxmMessage),
    // FromBackground(BxgMessage),
}

#[derive(Clone)]
pub enum BxgMessage {
    UpdateTable,
    UpdateTaskRow(Uuid, f64),
}

#[derive(Clone)]
pub enum MxmMessage {
    ShowAddUrlDialog,
    RemoveSelect,
    ReloadTable,
    StartSelect,
}

#[derive(Clone)]
pub enum AxgMessage {
    DetectUrl,
    UpdateUI(HashMap<String, DownloadInfo>),
    Submit,
    Cancel,
    CheckAllOrNone(bool),
    AddDownloadInfo(Vec<DownloadInfo>),
    SelectDir,
}

#[derive(Clone)]
enum TaskThreadMessage {
    NeedStart(Uuid),
}

fn main() {
    let app = app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Metro).apply();
    let (sender, receiver) = app::channel::<Message>();

    let mut ui = ui::main::UserInterface::make_window();
    let mut add_url_dialog = ui::add_url_dialog::UserInterface::make_window();

    add_url_dialog
        .choice_engine
        .add_choice(get_engine_names().join("|").as_str());
    add_url_dialog.choice_engine.set_value(0);

    add_url_dialog.btn_detect.emit(
        sender.clone(),
        Message::FromAddUrlDialog(AxgMessage::DetectUrl),
    );

    add_url_dialog.btn_submit.emit(
        sender.clone(),
        Message::FromAddUrlDialog(AxgMessage::Submit),
    );

    add_url_dialog.btn_cancel.emit(
        sender.clone(),
        Message::FromAddUrlDialog(AxgMessage::Cancel),
    );

    add_url_dialog.btn_select_dir.emit(
        sender.clone(),
        Message::FromAddUrlDialog(AxgMessage::SelectDir),
    );

    add_url_dialog.check_all.set_callback({
        let sender = sender.clone();
        move |check| {
            let checked = check.is_checked();
            sender.send(Message::FromAddUrlDialog(AxgMessage::CheckAllOrNone(
                checked,
            )));
        }
    });

    let current_idx: Arc<Mutex<HashMap<i32, DownloadInfo>>> = Default::default();

    ui.table_parent.begin();
    let mut task_table = view::TaskTable::default();
    ui.table_parent.end();

    ui.btn_add.emit(
        sender.clone(),
        Message::FromMainForm(MxmMessage::ShowAddUrlDialog),
    );

    ui.btn_delete.emit(
        sender.clone(),
        Message::FromMainForm(MxmMessage::RemoveSelect),
    );

    ui.btn_start.emit(
        sender.clone(),
        Message::FromMainForm(MxmMessage::StartSelect),
    );

    app::add_timeout3(0.5, {
        let sender = sender.clone();
        move |handle| {
            sender.send(Message::FromMainForm(MxmMessage::ReloadTable));
            app::repeat_timeout3(0.5, handle);
        }
    });

    while app.wait() {
        if let Some(message) = receiver.recv() {
            match message {
                Message::FromAddUrlDialog(message) => match message {
                    AxgMessage::DetectUrl => {
                        let url = add_url_dialog.input_url.value().trim().to_string();
                        if url.len() > 0 {
                            if let Some(engine) = add_url_dialog.choice_engine.choice() {
                                add_url_dialog.btn_detect.deactivate();

                                std::thread::spawn({
                                    let sender = sender.clone();
                                    move || {
                                        if let Ok(info_map) = get_stream_info(&engine, &url) {
                                            sender.send(Message::FromAddUrlDialog(
                                                AxgMessage::UpdateUI(info_map),
                                            ));
                                        }
                                    }
                                });
                            }
                        }
                    }

                    AxgMessage::UpdateUI(info_map) => {
                        let mut current_idx = current_idx.lock().unwrap();
                        current_idx.clear();
                        let mut title_updated = false;
                        add_url_dialog.checkbrowser.clear();

                        for (_id, info) in info_map {
                            if title_updated == false {
                                add_url_dialog.output_title.set_value(&info.title);
                                title_updated = true;
                            }

                            let check_name = format!(
                                "{} - {} (size: {})",
                                info.ext,
                                info.stream_name,
                                view::size_to_string(info.stream_size)
                            );
                            let idx = add_url_dialog.checkbrowser.add(check_name.as_str(), false);
                            current_idx.insert(idx, info);

                            add_url_dialog.checkbrowser.redraw();
                        }

                        if add_url_dialog.btn_detect.active() == false {
                            add_url_dialog.btn_detect.activate()
                        }
                    }

                    AxgMessage::Submit => {
                        let current_idx = current_idx.lock().unwrap();
                        let mut current_task = Vec::new();

                        let save_dir = add_url_dialog.input_dir.value();
                        let save_dir_path = std::path::Path::new(&save_dir);
                        if save_dir_path.is_dir() {
                            for i in 1..=add_url_dialog.checkbrowser.nitems() as i32 {
                                if add_url_dialog.checkbrowser.checked(i) {
                                    if let Some(info) = current_idx.get(&i) {
                                        let mut info = info.to_owned();
                                        info.save_option = Some(SaveOption {
                                            output_dir: save_dir.clone(),
                                            file_name: format!("{}.{}.{}", info.title, info.stream_name, info.ext),
                                        });
                                        current_task.push(info);
                                    }
                                }
                            }

                            add_url_dialog.window.hide();

                            sender.send(Message::FromAddUrlDialog(AxgMessage::AddDownloadInfo(
                                current_task,
                            )));
                        }
                    }

                    AxgMessage::Cancel => add_url_dialog.window.hide(),

                    AxgMessage::CheckAllOrNone(checked) => {
                        if checked {
                            add_url_dialog.checkbrowser.check_all();
                        } else {
                            add_url_dialog.checkbrowser.check_none();
                        }
                    }

                    AxgMessage::AddDownloadInfo(download_info) => {
                        task_table.add_download_info(download_info.as_slice());
                    }

                    AxgMessage::SelectDir => {
                        if let Some(dir) =
                            dialog::dir_chooser("Choose dir to save download file", "", false)
                        {
                            add_url_dialog.input_dir.set_value(&dir);
                        }
                    }
                },

                Message::FromMainForm(message) => match message {
                    MxmMessage::ShowAddUrlDialog => add_url_dialog.window.show(),
                    MxmMessage::RemoveSelect => task_table.remove_select(),
                    MxmMessage::ReloadTable => task_table.reload(),
                    MxmMessage::StartSelect => task_table.start_select(),
                },
            }
        }
    }
}
