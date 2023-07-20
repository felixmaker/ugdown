use std::{collections::HashMap, process::Child};

use anyhow::Result;

mod lux;
mod youget;
mod youtubedl;

#[derive(Clone, Debug)]
pub struct DownloadInfo {
    pub url: String,
    pub site: String,
    pub title: String,
    pub ext: String,
    pub stream_id: String,
    pub stream_name: String,
    pub stream_size: usize,
    pub downloader: String,
    pub save_option: Option<SaveOption>,
}

#[derive(Clone, Debug)]
pub struct SaveOption {
    pub output_dir: String,
    pub file_name: String,
}

pub trait Downloader {
    fn get_downloader_name(&self) -> String;
    fn get_stream_info(&self, url: &str) -> Result<HashMap<String, DownloadInfo>>;
    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_dir: &str,
        output_name: &str,
    ) -> Result<Child>;
    fn output_in_stderr(&self) -> bool;
}

use lux::Lux;
use youget::Youget;
use youtubedl::Youtubedl;

pub fn get_engine_names() -> Vec<String> {
    ["lux", "you-get", "youtube-dl"]
        .map(|x| x.to_string())
        .to_vec()
}

pub fn get_engine(engine: &str) -> Result<Box<dyn Downloader>> {
    match engine.to_ascii_lowercase().trim() {
        "lux" => Ok(Box::new(Lux {})),
        "you-get" | "youget" => Ok(Box::new(Youget {})),
        "youtube-dl" | "youtubedl" => Ok(Box::new(Youtubedl {})),
        _ => Err(anyhow::anyhow!("engine are not supported {}", engine)),
    }
}

pub fn get_stream_info(engine: &str, url: &str) -> Result<HashMap<String, DownloadInfo>> {
    let engine = get_engine(engine)?;
    engine.get_stream_info(url)
}

pub fn execute_download_info(download_info: &DownloadInfo) -> Result<(Child, bool)> {
    let download_info = download_info.clone();
    let (output_dir, output_name) = download_info
        .save_option
        .and_then(|x| Some((x.output_dir, x.file_name)))
        .unwrap_or((
            "./".to_owned(),
            format!("{}.{}", download_info.title, download_info.ext),
        ));

    let engine = get_engine(&download_info.downloader)?;
    let url = download_info.url;
    let id = download_info.stream_id;
    Ok((
        engine.execute_download(&url, &id, &output_dir, &output_name)?,
        engine.output_in_stderr(),
    ))
}

