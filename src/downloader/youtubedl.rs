use std::{collections::HashMap, process::Stdio, path::{Path, PathBuf}};

use anyhow::Result;
use serde::Deserialize;

use super::{DownloadInfo, Downloader};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlNode {
    id: String,
    duration: f64,
    formats: Vec<YoutuledlFormatNode>,
    title: String,
    description: String,
    timestamp: u64,
    uploader: String,
    uploader_id: String,
    extractor: String,
    webpage_url: String,
    webpage_url_basename: String,
    extractor_key: String,
    playlist: Option<YoutuledlPlaylist>,
    playlist_index: Option<String>,
    thumbnails: Vec<YoutuledlThumbnailNode>,
    display_id: String,
    upload_date: String,
    requested_subtitles: Option<Vec<YoutuledlSubtitleNode>>,
    url: String,
    filesize: u64,
    http_headers: HashMap<String, String>,
    ext: String,
    format_id: String,
    format: String,
    protocol: String,
    fulltitle: String,
    _filename: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlFormatNode {
    url: String,
    filesize: usize,
    http_headers: HashMap<String, String>,
    ext: String,
    format_id: String,
    format: String,
    protocol: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlPlaylist {}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlThumbnailNode {
    url: String,
    id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlSubtitleNode {}

pub struct Youtubedl {}

impl Downloader for Youtubedl {
    fn get_downloader_name() -> String {
        "Youtube-dl".to_owned()
    }

    fn get_stream_info(url: &str) -> Result<HashMap<String, DownloadInfo>> {
        let result = std::process::Command::new("youtube-dl")
            .arg("-j")
            .arg(url)
            .output()?;
        let result = String::from_utf8(result.stdout.to_vec())?;
        let result: YoutuledlNode = serde_json::from_str(&result)?;

        let mut info_map = HashMap::new();

        let site = &result.webpage_url;
        let title = &result.title;

        for format_node in &result.formats {
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: format_node.ext.clone(),
                stream_id: format_node.format_id.clone(),
                stream_name: format_node.format.clone(),
                stream_size: format_node.filesize,
                downloader: Self::get_downloader_name(),
                save_option: None,
            };

            info_map.insert(format_node.format_id.clone(), info);
        }

        Ok(info_map)
    }

    fn download<F>(url: &str, output_dir: &str, output_name: &str, callback: F) -> anyhow::Result<()>
    where
        F: Fn(f64) + Clone,
    {
        let output = format!("{}/{}", output_dir, output_name);
        let child = std::process::Command::new("youtube-dl")
            .arg("-o")
            .arg(output)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        super::watch::watch_progress(child.stdout.unwrap(), callback);

        Ok(())
    }

    fn download_by_id<F>(url: &str, id: &str, output_dir: &str, output_name: &str, callback: F) -> anyhow::Result<()>
    where
        F: Fn(f64) + Clone,
    {
        let output = format!("{}/{}", output_dir, output_name);
        let child = std::process::Command::new("youtube-dl")
            .arg("-f")
            .arg(id)
            .arg("-o")
            .arg(output)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        super::watch::watch_progress(child.stdout.unwrap(), callback);

        Ok(())
    }
}
