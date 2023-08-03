use std::{
    collections::VecDeque,
    io::{BufReader, Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use fltk::prelude::*;

use super::utils::{percent_to_string, size_to_string, speed_to_string};

mod ui {
    fl2rust_macro::include_ui!("./src/ui/tool_downloader.fl");
}

struct Task {
    url: String,
    output_path: String,
    message: String,
}

#[derive(Clone)]
pub struct ToolDownloader {
    downloader: ui::UserInterface,
    download_queue: Arc<Mutex<VecDeque<Task>>>,
}

impl ToolDownloader {
    pub fn default() -> Self {
        let mut downloader = ui::UserInterface::make_window();
        downloader.progress.set_minimum(0.0);
        downloader.progress.set_maximum(1.0);

        let kill_download = Arc::new(Mutex::new(false));

        let download_queue: Arc<Mutex<VecDeque<Task>>> = Default::default();

        downloader.btn_cancel.set_callback({
            let kill_download = kill_download.clone();
            let mut downloader = downloader.clone();
            move |_| {
                *kill_download.lock().unwrap() = true;
                downloader.window.hide();
            }
        });

        std::thread::spawn({
            let download_queue = download_queue.clone();
            let mut downloader = downloader.clone();
            let kill_download = kill_download.clone();
            let sender = crate::CHANNEL.0.clone();
            move || -> Result<()> {
                loop {
                    if let Some(task) = download_queue.lock().unwrap().pop_front() {
                        downloader.lable_message.set_label(&task.message);
                        downloader.progress.set_value(0.0);

                        let url = task.url;

                        let temp_path = PathBuf::from(format!("{}.download", task.output_path));
                        let output_path = PathBuf::from(&task.output_path);

                        let kill_download = kill_download.clone();

                        if output_path.is_file() {
                            downloader.progress.set_value(1.0);
                            downloader.progress.set_label(&percent_to_string(1.0));
                            sender.send(crate::AppMessage::Finished);
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

                        downloader
                            .output_total
                            .set_value(&size_to_string(content_length));

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

                            downloader
                                .output_speed
                                .set_value(&speed_to_string(speed as usize));

                            if content_length > 0 && length > 0 {
                                let progress = write_length as f64 / content_length as f64;
                                downloader.progress.set_value(progress);
                                downloader.progress.set_label(&percent_to_string(progress))
                            }

                            if length == 0 {
                                downloader.progress.set_value(1.0);
                                downloader.progress.set_label(&percent_to_string(1.0));
                                std::fs::rename(&temp_path, &output_path)?;
                                sender.send(crate::AppMessage::Finished);
                                break;
                            }

                            if *kill_download.lock().unwrap() {
                                break;
                            }
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        });

        Self {
            downloader,
            download_queue,
        }
    }

    pub fn start_download(&mut self, url: &str, output_path: &str, message: &str) {
        let task = Task {
            url: url.to_owned(),
            output_path: output_path.to_owned(),
            message: message.to_owned(),
        };

        self.downloader.window.show();
        self.download_queue.lock().unwrap().push_back(task);
    }

    pub fn hide(&mut self) {
        self.downloader.window.hide();
    }

    pub fn shown(&self) -> bool {
        self.downloader.window.shown()
    }
}
