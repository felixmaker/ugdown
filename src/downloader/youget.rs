use std::{
    collections::HashMap,
    process::{Child, Stdio},
};

use anyhow::Result;
use serde::Deserialize;

use super::{DownloadInfo, Downloader};

#[derive(Debug, Deserialize)]
struct YougetNode {
    title: String,
    site: String,
    streams: HashMap<String, YougetStreamsNode>,
}

#[derive(Debug, Deserialize)]
struct YougetStreamsNode {
    container: Option<String>,
    quality: Option<String>,
    size: usize,
    // src: Vec<YougetSrc>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum YougetSrc {
    Multi(Vec<String>),
    Single(String),
}

pub struct Youget {}

impl Downloader for Youget {
    fn get_downloader_name(&self) -> String {
        "Youget".to_owned()
    }

    fn get_stream_info(&self, url: &str) -> Result<HashMap<String, DownloadInfo>> {
        let result = std::process::Command::new("you-get")
            .arg("--json")
            .arg(url)
            .output()?;
        let result = String::from_utf8(result.stdout.to_vec())?;
        let fixed_re = regex::Regex::new(r"(?s).*?(\{.*\})").unwrap();
        let result = fixed_re
            .find(&result)
            .ok_or_else(|| anyhow::anyhow!("Unknown format"))?
            .as_str()
            .to_owned();

        let result: YougetNode = serde_json::from_str(&result)?;

        let mut info_map = HashMap::new();

        let site = &result.site;
        let title = &result.title;

        for (stream_id, stream_node) in &result.streams {
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: stream_node.container.clone().unwrap_or("Unknown".to_owned()),
                stream_id: stream_id.clone(),
                stream_name: stream_node.quality.clone().unwrap_or("Unknown".to_owned()),
                stream_size: stream_node.size,
                downloader: self.get_downloader_name(),
                save_option: None,
            };

            info_map.insert(stream_id.clone(), info);
        }

        Ok(info_map)
    }

    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_dir: &str,
        output_file: &str,
    ) -> anyhow::Result<Child> {
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

        Ok(child)
    }

    fn output_in_stderr(&self) -> bool {
        false
    }
}
