use std::{
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

        let kill_download = Arc::new(Mutex::new(false));

        downloader.btn_cancel.set_callback({
            let kill_download = kill_download.clone();
            let mut downloader = downloader.clone();
            move |_| {
                *kill_download.lock().unwrap() = true;
                downloader.window.hide();
            }
        });

        Self {
            downloader,
            kill_download,
        }
    }

    pub fn start_download(
        &mut self,
        url: &str,
        output_path: &str,
        title: &str,
        message: &str,
    ) -> Result<()> {
        self.downloader.window.set_label(title);
        self.downloader.lable_message.set_label(message);
        self.downloader.progress.set_value(0.0);
        self.downloader.window.show();

        let url = url.to_owned();

        let temp_path = PathBuf::from(format!("{}.download", output_path));
        let output_path = PathBuf::from(&output_path);

        let kill_download = self.kill_download.clone();
        let mut progress_widget = self.downloader.progress.clone();

        if output_path.is_file() {
            progress_widget.set_value(1.0);
            progress_widget.set_label(&percent_to_string(1.0));
            return Ok(());
        }

        let mut start_bytes = 0;
        if temp_path.is_file() {
            start_bytes = std::fs::metadata(&temp_path)?.len() as usize;
        }

        let mut output_speed = self.downloader.output_speed.clone();
        let mut output_total = self.downloader.output_total.clone();

        std::thread::spawn(move || -> Result<()> {
            let mut response = ureq::get(&url).call()?;

            let content_length = response
                .header("Content-Length")
                .map(|x| x.parse::<usize>().unwrap_or(0))
                .unwrap_or(0);

            output_total.set_value(&size_to_string(content_length));

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

                output_speed.set_value(&speed_to_string(speed as usize));

                if content_length > 0 && length > 0 {
                    let progress = write_length as f64 / content_length as f64;
                    progress_widget.set_value(progress);
                    progress_widget.set_label(&percent_to_string(progress))
                }

                if length == 0 {
                    progress_widget.set_value(1.0);
                    progress_widget.set_label(&percent_to_string(1.0));
                    std::fs::rename(&temp_path, &output_path)?;
                    break;
                }

                if *kill_download.lock().unwrap() {
                    break;
                }
            }

            Ok(())
        });
        Ok(())
    }
}
