use std::{
    collections::HashMap,
    path::Path,
    process::{Child, Stdio},
};

use anyhow::Result;
use serde::Deserialize;

use super::*;

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlNode {
    // id: String,
    // duration: f64,
    formats: Option<Vec<YoutuledlFormatNode>>,
    title: String,
    // description: String,
    // timestamp: u64,
    // uploader: String,
    // uploader_id: String,
    // extractor: String,
    webpage_url: String,
    // webpage_url_basename: String,
    // extractor_key: String,
    // playlist: Option<YoutuledlPlaylist>,
    // playlist_index: Option<String>,
    // thumbnails: Vec<YoutuledlThumbnailNode>,
    // display_id: String,
    // upload_date: String,
    // requested_subtitles: Option<Vec<YoutuledlSubtitleNode>>,
    // url: String,
    filesize: Option<usize>,
    // http_headers: HashMap<String, String>,
    ext: String,
    format_id: String,
    format: String,
    // protocol: String,
    // fulltitle: String,
    _filename: String,
}

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YoutuledlFormatNode {
    // url: String,
    filesize: usize,
    // http_headers: HashMap<String, String>,
    ext: String,
    format_id: String,
    format: String,
    // protocol: String,
}

// #[allow(dead_code)]
// #[derive(Debug, Deserialize)]
// struct YoutuledlPlaylist {}

// #[allow(dead_code)]
// #[derive(Debug, Deserialize)]
// struct YoutuledlThumbnailNode {
//     url: String,
//     id: String,
// }

// #[allow(dead_code)]
// #[derive(Debug, Deserialize)]
// struct YoutuledlSubtitleNode {}

pub struct Youtubedl {}

impl Downloader for Youtubedl {
    fn get_downloader_name(&self) -> String {
        "Youtube-dl".to_owned()
    }

    fn get_stream_info(
        &self,
        url: &str,
        cookie_file: Option<&Path>,
    ) -> Result<HashMap<String, DownloadInfo>> {
        let result = match &cookie_file {
            Some(file) => create_hide_window_command("lux")
                .arg("-c")
                .arg(file)
                .arg("--socket-timeout")
                .arg("4")
                .arg("-j")
                .arg(url)
                .output()?,
            None => create_hide_window_command("youtube-dl")
                .arg("--socket-timeout")
                .arg("4")
                .arg("-j")
                .arg(url)
                .output()?,
        };

        let result = String::from_utf8(result.stdout.to_vec())?;
        let result: YoutuledlNode = serde_json::from_str(&result)?;

        let mut info_map = HashMap::new();

        let site = &result.webpage_url;
        let title = &result.title;

        if let Some(formats) = &result.formats {
            for format_node in formats {
                let info = DownloadInfo {
                    url: url.to_string(),
                    site: site.clone(),
                    title: title.clone(),
                    ext: format_node.ext.clone(),
                    stream_id: format_node.format_id.clone(),
                    stream_name: format_node.format.clone(),
                    stream_size: format_node.filesize,
                    downloader: self.get_downloader_name(),
                    ..Default::default()
                };

                info_map.insert(format_node.format_id.clone(), info);
            }
        } else {
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: result.ext.clone(),
                stream_id: result.format_id.clone(),
                stream_size: result.filesize.unwrap_or(0),
                stream_name: result.format.clone(),
                downloader: self.get_downloader_name(),
                ..Default::default()
            };
            info_map.insert(result.format_id.clone(), info);
        }

        Ok(info_map)
    }

    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_dir: &str,
        output_name: &str,
        cookie_file: Option<&Path>,
    ) -> anyhow::Result<Child> {
        let output = format!("{}/{}", output_dir, output_name);

        let child = match &cookie_file {
            Some(cookie_file) => create_hide_window_command("youtube-dl")
                .arg("-c")
                .arg(cookie_file)
                .arg("-f")
                .arg(id)
                .arg("-o")
                .arg(output)
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?,
            None => create_hide_window_command("youtube-dl")
                .arg("-f")
                .arg(id)
                .arg("-o")
                .arg(output)
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?,
        };

        Ok(child)
    }

    fn is_stderr_output(&self) -> bool {
        false
    }
}
