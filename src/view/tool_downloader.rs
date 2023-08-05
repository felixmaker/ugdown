use std::{
    io::{BufReader, Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use fltk::prelude::*;

use crate::{downloader::get_plugin_dir, send_message, AppMessage};

use super::{
    utils::{extract_file_to_plugin, percent_to_string, size_to_string, speed_to_string},
    EngineManagerMessage,
};

mod ui {
    fl2rust_macro::include_ui!("./src/ui/tool_downloader.fl");
}

#[derive(Clone)]
pub struct Task {
    pub url: String,
    pub output_path: String,
    pub message: String,
    pub to_plugin: bool,
}

#[derive(Clone)]
pub struct ToolDownloader {
    downloader: ui::UserInterface,
    kill_download: Arc<Mutex<bool>>,
}

impl ToolDownloader {
    pub fn default() -> Self {
        let mut downloader = ui::UserInterface::make_window();
        downloader.progress.set_minimum(0.0);
        downloader.progress.set_maximum(1.0);
        downloader
            .btn_cancel
            .set_callback(|_| send_message(ToolDownloaderMessage::CancelCurrent));

        let kill_download = Arc::new(Mutex::new(false));

        Self {
            downloader,
            kill_download,
        }
    }

    pub fn cancel_download(&mut self) {
        *self.kill_download.lock().unwrap() = true;
        self.downloader.window.hide();
    }

    fn get_kill_download(&self, kill: bool) -> Arc<Mutex<bool>> {
        let kill_download = self.kill_download.clone();
        {
            *kill_download.lock().unwrap() = kill;
        }
        kill_download
    }

    pub fn start_download(&mut self, task: Task) {
        let kill_download = self.get_kill_download(false);

        std::thread::spawn(move || -> Result<()> {
            send_message(ToolDownloaderMessage::Show);

            let url = task.url;
            let message = task.message;

            let temp_path = PathBuf::from(format!("{}.download", &task.output_path));
            let output_path = PathBuf::from(&task.output_path);

            if output_path.is_file() {
                send_message(ToolDownloaderMessage::SetStatus(TaskStatus::new(
                    &message, 0, 1.0, 0.0,
                )));
                send_message(ToolDownloaderMessage::Hide);
                if task.to_plugin {
                    put_to_plugin(&output_path)?;
                }
                return Ok(());
            }

            let mut start_bytes = 0;
            if temp_path.is_file() {
                start_bytes = std::fs::metadata(&temp_path)?.len() as usize;
            }

            let mut response = ureq::get(&url).call()?;

            let content_length = response
                .header("Content-Length")
                .map(|x| x.parse::<usize>().unwrap_or(0))
                .unwrap_or(0);

            if start_bytes > 0 || start_bytes < content_length {
                response = ureq::get(&url)
                    .set("Range", &format!("bytes={}-", start_bytes))
                    .call()?;
            }

            let mut write_length = start_bytes;

            let mut output_file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&temp_path)?;

            let mut response = BufReader::new(response.into_reader());
            let mut buf = [0; 1024 * 1024];

            let mut before = std::time::Instant::now();

            loop {
                let length = response.read(&mut buf)?;
                output_file.write(&buf[0..length])?;
                write_length = write_length + length;

                let end = std::time::Instant::now();
                let duration = end - before;
                before = end;
                let speed = length as f64 / 1024.0 / duration.as_secs_f64();

                if content_length > 0 && length > 0 {
                    let progress = write_length as f64 / content_length as f64;
                    send_message(ToolDownloaderMessage::SetStatus(TaskStatus::new(
                        &message,
                        content_length,
                        progress,
                        speed,
                    )));
                }

                if length == 0 {
                    send_message(ToolDownloaderMessage::SetStatus(TaskStatus::new(
                        &message,
                        content_length,
                        1.0,
                        speed,
                    )));
                    std::fs::rename(&temp_path, &output_path)?;
                    send_message(ToolDownloaderMessage::Hide);
                    if task.to_plugin {
                        put_to_plugin(&output_path)?;
                    }
                    break;
                }

                if *kill_download.lock().unwrap() {
                    send_message(ToolDownloaderMessage::Hide);
                    break;
                }
            }

            Ok(())
        });
    }

    fn set_status(&mut self, task_status: TaskStatus) {
        self.downloader.progress.set_value(task_status.progress);
        self.downloader
            .progress
            .set_label(&percent_to_string(task_status.progress));
        self.downloader
            .output_speed
            .set_value(&speed_to_string(task_status.speed as usize));

        self.downloader.window.set_label(&task_status.message);
        self.downloader
            .lable_message
            .set_label(&task_status.message);

        self.downloader
            .output_total
            .set_value(&size_to_string(task_status.content_length));
    }

    pub fn handle_message(&mut self, message: ToolDownloaderMessage) {
        match message {
            ToolDownloaderMessage::StartDownload(task) => self.start_download(task),
            ToolDownloaderMessage::SetStatus(task_status) => self.set_status(task_status),
            ToolDownloaderMessage::Show => self.downloader.window.show(),
            ToolDownloaderMessage::Hide => self.downloader.window.hide(),
            ToolDownloaderMessage::CancelCurrent => self.cancel_download(),
        }
    }
}

#[derive(Clone)]
pub struct TaskStatus {
    progress: f64,
    speed: f64,
    message: String,
    content_length: usize,
}

impl TaskStatus {
    fn new(message: &str, total: usize, progress: f64, speed: f64) -> Self {
        Self {
            progress,
            speed,
            message: message.to_owned(),
            content_length: total,
        }
    }
}

#[derive(Clone)]
pub enum ToolDownloaderMessage {
    StartDownload(Task),
    CancelCurrent,
    SetStatus(TaskStatus),
    Show,
    Hide,
}

impl From<ToolDownloaderMessage> for AppMessage {
    fn from(value: ToolDownloaderMessage) -> Self {
        Self::ToolDownloader(value)
    }
}

fn put_to_plugin(output_path: &PathBuf) -> Result<()> {
    let file_name = output_path
        .file_name()
        .map(|x| x.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Unknown filename"))?;

    if file_name.ends_with(".zip") {
        extract_file_to_plugin(output_path)?;
    } else if file_name.ends_with(".exe") {
        std::fs::copy(output_path, get_plugin_dir()?.join(&file_name))?;
    }
    
    fltk::dialog::message_default(&format!("{} is downloaded and extract to plugin.", &file_name));
    send_message(EngineManagerMessage::Detect);

    Ok(())
}
