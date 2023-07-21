use std::{
    collections::HashMap,
    process::{Child, Stdio}, path::Path,
};

use anyhow::Result;
use serde::Deserialize;

use super::*;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LuxNode {
    url: String,
    site: String,
    title: String,
    #[serde(rename = "type")]
    type_: String,
    streams: HashMap<String, LuxStreamsNode>,
    caption: Option<LuxCaption>,
    err: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LuxStreamsNode {
    id: String,
    quality: String,
    parts: Vec<LuxInfo>,
    size: usize,
    ext: String,
    #[serde(rename = "NeedMux")]
    need_mux: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LuxCaption {
    subtitle: Option<String>,
    danmaku: LuxInfo,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LuxInfo {
    url: String,
    size: u64,
    ext: String,
}

pub struct Lux {}

impl Downloader for Lux {
    fn get_downloader_name(&self) -> String {
        "Lux".to_owned()
    }

    fn get_stream_info(
        &self,
        url: &str,
        cookie_file: Option<&Path>
    ) -> Result<HashMap<String, DownloadInfo>> {

        let result = match &cookie_file {
            Some(file) => std::process::Command::new("lux")
                .arg("-c")
                .arg(file)
                .arg("-j")
                .arg(url)
                .output()?,
            None => std::process::Command::new("lux")
                .arg("-j")
                .arg(url)
                .output()?,
        };

        let result = String::from_utf8(result.stdout.to_vec())?;
        let result: Vec<LuxNode> = serde_json::from_str(&result)?;

        let node = result
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Wrong at get 0 index ??"))?;

        let mut info_map = HashMap::new();

        let site = &node.site;
        let title = &node.title;

        for (stream_id, stream_node) in &node.streams {
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: stream_node
                    .parts
                    .first()
                    .and_then(|x| Some(x.ext.to_owned()))
                    .unwrap_or(stream_node.ext.clone()),
                stream_id: stream_id.clone(),
                stream_name: stream_node.quality.clone(),
                stream_size: stream_node.size,
                downloader: self.get_downloader_name(),
                ..Default::default()
            };

            info_map.insert(stream_id.clone(), info);
        }

        Ok(info_map)
    }

    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_path: &str,
        output_name: &str,
        cookie_file: Option<&Path>
    ) -> anyhow::Result<Child> {
        
        let child = match &cookie_file {
            Some(cookie_file) => std::process::Command::new("lux")
                .arg("-c")
                .arg(cookie_file)
                .arg("-f")
                .arg(id)
                .arg("-o")
                .arg(output_path)
                .arg("-O")
                .arg(output_name)
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()?,
            None => std::process::Command::new("lux")
                .arg("-f")
                .arg(id)
                .arg("-o")
                .arg(output_path)
                .arg("-O")
                .arg(output_name)
                .arg(url)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()?,
        };

        Ok(child)
    }

    fn is_stderr_output(&self) -> bool {
        true
    }
}
