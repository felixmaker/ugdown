use std::{collections::HashMap, process::Stdio};

use anyhow::Result;
use serde::Deserialize;

use super::{DownloadInfo, Downloader};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YougetNode {
    url: String,
    title: String,
    site: String,
    streams: HashMap<String, YougetStreamsNode>,
    audiolang: Option<String>, // Unknown
    extra: Option<YougetExtra>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YougetExtra {
    referer: String,
    ua: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct YougetStreamsNode {
    container: String,
    quality: String,
    size: usize,
    src: Vec<YougetSrc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum YougetSrc {
    Multi(Vec<String>),
    Single(String),
}

pub struct Youget {}

impl Downloader for Youget {
    fn get_downloader_name() -> String {
        "Youget".to_owned()
    }

    fn get_stream_info(url: &str) -> Result<HashMap<String, DownloadInfo>> {
        let result = std::process::Command::new("you-get")
            .arg("--json")
            .arg(url)
            .output()?;
        let result = String::from_utf8(result.stdout.to_vec())?;
        let result: YougetNode = serde_json::from_str(&result)?;

        let mut info_map = HashMap::new();

        let site = &result.site;
        let title = &result.title;

        for (stream_id, stream_node) in &result.streams {
            
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: stream_node.container.clone(),
                stream_id: stream_id.clone(),
                stream_name: stream_node.quality.clone(),
                stream_size: stream_node.size,
                downloader: Self::get_downloader_name(),
                save_option: None,
            };

            info_map.insert(stream_id.clone(), info);
        }

        Ok(info_map)
    }

    fn download<F>(url: &str, output_dir: &str, output_file: &str, callback: F) -> anyhow::Result<()>
    where
        F: Fn(f64) + Clone,
    {
        let child = std::process::Command::new("you-get")
            .arg("-o")
            .arg(output_dir)
            .arg("-O")
            .arg(output_file)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        super::watch::watch_progress(child.stdout.unwrap(), callback);

        Ok(())
    }

    fn download_by_id<F>(url: &str, id: &str, output_dir: &str, output_file: &str, callback: F) -> anyhow::Result<()>
    where
        F: Fn(f64) + Clone,
    {
        let child = std::process::Command::new("you-get")
            .arg("--format")
            .arg(id)
            .arg("-o")
            .arg(output_dir)
            .arg("-O")
            .arg(output_file)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        super::watch::watch_progress(child.stdout.unwrap(), callback);

        Ok(())
    }
}
