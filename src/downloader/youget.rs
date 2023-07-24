use std::{
    collections::HashMap,
    path::Path,
    process::{Child, Stdio},
};

use anyhow::Result;

use super::*;

#[derive(Debug)]
struct YougetStreamsNode {
    format: String,
    container: String,
    quality: String,
    size: usize,
}

pub struct Youget {}

impl Downloader for Youget {
    fn get_downloader_name(&self) -> String {
        "Youget".to_owned()
    }

    fn get_stream_info(
        &self,
        url: &str,
        cookie_file: Option<&Path>,
    ) -> Result<HashMap<String, DownloadInfo>> {
        let result = match &cookie_file {
            Some(file) => create_hide_window_command("you-get")
                .arg("-c")
                .arg(file)
                .arg("-i")
                .arg(url)
                .output()?,
            None => create_hide_window_command("you-get")
                .arg("-i")
                .arg(url)
                .output()?,
        };

        let result = String::from_utf8(result.stdout.to_vec())?;

        let mut site: Option<String> = None;
        let mut title: Option<String> = None;
        let mut streams: Vec<YougetStreamsNode> = Default::default();

        let mut format: Option<String> = None;
        let mut container: Option<String> = None;
        let mut quality: Option<String> = None;
        let mut size: Option<usize> = None;

        let re_size = regex::Regex::new(r"\(([0-9]*) bytes\)").unwrap();

        for line in result.lines() {
            match line.trim() {
                lsite if lsite.starts_with("site:") => site = Some(lsite[5..].trim().to_string()),
                ltitle if ltitle.starts_with("title:") => {
                    title = Some(ltitle[6..].trim().to_string())
                }
                lformat if lformat.starts_with("- format:") => {
                    format = Some(lformat[9..].trim().to_string())
                }
                lcontainer if lcontainer.starts_with("container:") => {
                    container = Some(lcontainer[10..].trim().to_string())
                }
                lquality if lquality.starts_with("quality:") => {
                    quality = Some(lquality[8..].trim().to_string())
                }
                lsize if lsize.starts_with("size:") => {
                    let lsize = lsize[5..].trim();
                    if let Some(size_found) = re_size.captures(lsize) {
                        let size_found: usize = size_found
                            .get(1)
                            .map(|x| x.as_str().parse::<usize>().unwrap_or(0))
                            .unwrap_or(0);
                        size = Some(size_found);
                    }
                }
                lformat_end if lformat_end.starts_with("# download-with") => {
                    streams.push(YougetStreamsNode {
                        format: format.take().unwrap_or("__dafault__".to_string()),
                        container: container.take().unwrap_or("Unknown".to_owned()),
                        quality: quality.take().unwrap_or("Unknown".to_owned()),
                        size: size.take().unwrap_or(0),
                    })
                }
                _ => {}
            }
        }

        let mut info_map = HashMap::new();

        let site = site.unwrap_or("Unknown".to_owned());
        let title = title.unwrap_or("Unknown".to_owned());

        for stream_node in streams {
            let info = DownloadInfo {
                url: url.to_string(),
                site: site.clone(),
                title: title.clone(),
                ext: stream_node.container.clone(),
                stream_id: stream_node.format.clone(),
                stream_name: stream_node.quality.clone(),
                stream_size: stream_node.size,
                downloader: self.get_downloader_name(),
                ..Default::default()
            };

            info_map.insert(stream_node.format.clone(), info);
        }

        Ok(info_map)
    }

    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_dir: &str,
        output_file: &str,
        cookie_file: Option<&Path>,
    ) -> anyhow::Result<Child> {
        let child = match &cookie_file {
            Some(cookie_file) => create_hide_window_command("you-get")
                .arg("-c")
                .arg(cookie_file)
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
                .spawn()?,
            None => create_hide_window_command("you-get")
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
                .spawn()?,
        };

        Ok(child)
    }

    fn is_stderr_output(&self) -> bool {
        false
    }
}
