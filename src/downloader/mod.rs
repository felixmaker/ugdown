use std::collections::HashMap;

use anyhow::Result;

mod watch;

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

trait Downloader {
    fn get_downloader_name() -> String;
    fn get_stream_info(url: &str) -> Result<HashMap<String, DownloadInfo>>;
    fn download<F>(url: &str, output_path: &str, output_name: &str, callback: F) -> Result<()>
    where
        F: Fn(f64) + Clone;
    fn download_by_id<F>(
        url: &str,
        id: &str,
        output_dir: &str,
        output_name: &str,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(f64) + Clone;
}

use lux::Lux;
use youget::Youget;
use youtubedl::Youtubedl;

pub fn get_engine_names() -> Vec<String> {
    ["lux", "you-get", "youtube-dl"]
        .map(|x| x.to_string())
        .to_vec()
}

pub fn get_stream_info(engine: &str, url: &str) -> Result<HashMap<String, DownloadInfo>> {
    match engine.to_ascii_lowercase().trim() {
        "lux" => Lux::get_stream_info(url),
        "you-get" | "youget" => Youget::get_stream_info(url),
        "youtube-dl" | "youtubedl" => Youtubedl::get_stream_info(url),
        _ => Err(anyhow::anyhow!("engine are not supported {}", engine)),
    }
}

pub fn download<F>(
    engine: &str,
    url: &str,
    output_dir: &str,
    output_name: &str,
    callback: F,
) -> Result<()>
where
    F: Fn(f64) + Clone,
{
    match engine.to_ascii_lowercase().trim() {
        "lux" => Lux::download(url, output_dir, output_name, callback),
        "you-get" | "youget" => Youget::download(url, output_dir, output_name, callback),
        "youtube-dl" | "youtubedl" => Youtubedl::download(url, output_dir, output_name, callback),
        _ => Err(anyhow::anyhow!("engine are not supported {}", engine)),
    }
}

pub fn download_by_id<F>(
    engine: &str,
    url: &str,
    id: &str,
    output_dir: &str,
    output_name: &str,
    callback: F,
) -> Result<()>
where
    F: Fn(f64) + Clone,
{
    match engine.to_ascii_lowercase().trim() {
        "lux" => Lux::download_by_id(url, id, output_dir, output_name, callback),
        "you-get" | "youget" => Youget::download_by_id(url, id, output_dir, output_name, callback),
        "youtube-dl" | "youtubedl" => {
            Youtubedl::download_by_id(url, id, output_dir, output_name, callback)
        }
        _ => Err(anyhow::anyhow!("engine are not supported {}", engine)),
    }
}
