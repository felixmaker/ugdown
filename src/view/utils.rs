pub fn size_to_string(size: usize) -> String {
    match size {
        0 => format!("Unknown"),
        gb if gb >= 1000 * 1000 * 1000 => format!("{:.2} GB", gb as f64 / 1000000000.0),
        mb if mb >= 1000 * 1000 => format!("{:.2} MB", mb as f64 / 1000000.0),
        kb if kb >= 1000 => format!("{:.2} KB", kb as f64 / 1000.0),
        b => format!("{} B", b),
    }
}

pub fn speed_to_string(size: usize) -> String {
    match size_to_string(size).as_str() {
        "Unknown" => "---".to_owned(),
        other => format!("{}/s", other),
    }
}

pub fn percent_to_string(percent: f64) -> String {
    format!("{:.1}%", percent * 100.0)
}

pub fn eta_to_string(eta: usize) -> String {
    let days = eta / (60 * 60 * 24);
    let hours = (eta - days * 60 * 60 * 24) / (60 * 60);
    let minutes = (eta - days * 60 * 60 * 24 - hours * 60 * 60) / 60;
    let seconds = eta - days * 60 * 60 * 24 - hours * 60 * 60 - minutes * 60;

    match (days, hours, minutes, seconds) {
        (d, _, _, _) if d != 0 => format!(">= {}d", d),
        (_, h, _, _) if h != 0 => format!(">= {}h", h),
        (_, _, m, s) if m != 0 => format!("{}m {}s", m, s),
        (_, _, _, s) => format!("{}s", s),
    }
}

use anyhow::Result;
use std::collections::HashMap;
use url::Url;



#[allow(unused)]
#[derive(Clone)]
pub struct GithubLatestRelease {
    owner: String,
    repo: String,
    pub version: String,
    pub download_assets: DownloadAssets,
}

impl GithubLatestRelease {
    pub fn get_from_github_api(engine_name: &str) -> Option<Self> {
        match engine_name.to_lowercase().trim() {
            "lux" => get_lux().ok(),
            "youtube-dl" => get_youtubedl().ok(),
            "you-get" => get_youget().ok(),
            _ => None,
        }
    }
}

pub fn get_github_latest(owner: &str, repo: &str) -> Result<serde_json::Value> {
    // https://docs.github.com/en/free-pro-team@latest/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let response: serde_json::Value = ureq::get(&url).call()?.into_json()?;
    Ok(response)
}

pub fn replace_github_download(origin: &str, mirror: &str) -> String {
    match mirror.to_ascii_lowercase().trim() {
        "kgithub" | "kgithub.com" => origin.replace("https://github.com", "https://kgithub.com"),
        "ghproxy" | "ghproxy.com" => {
            origin.replace("https://github.com", "https://ghproxy.com/github.com")
        }
        _ => origin.to_owned(),
    }
}

#[derive(Default, Clone)]
pub struct DownloadAssets {
    windows_x86_64: Option<String>,
    windows_x86: Option<String>,
    macos_x86_64: Option<String>,
    macos_arm64: Option<String>,
    linux_x86_64: Option<String>,
}

impl DownloadAssets {
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        if let Some(windows_x86_64) = &self.windows_x86_64 {
            result.insert("windows_x86_64".to_owned(), windows_x86_64.clone());
        }
        if let Some(windows_x86) = &self.windows_x86 {
            result.insert("windows_x86".to_owned(), windows_x86.clone());
        }
        if let Some(macos_arm64) = &self.macos_arm64 {
            result.insert("macos_arm64".to_owned(), macos_arm64.clone());
        }
        if let Some(macos_x86_64) = &self.macos_x86_64 {
            result.insert("macos_x86_64".to_owned(), macos_x86_64.clone());
        }
        if let Some(linux_x86_64) = &self.linux_x86_64 {
            result.insert("linux_x86_64".to_owned(), linux_x86_64.clone());
        }

        result
    }

    pub fn to_order_vec(&self) -> Vec<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let mut result: Vec<String> = self.to_hashmap().keys().map(|x| x.to_owned()).collect();
        let os_asset = format!("{os}_{arch}");
        
        for i in 0..result.len() {
            if os_asset == result[i] {
                result.swap(0, i);
            }
        }

        result
    }
}

fn get_lux() -> Result<GithubLatestRelease> {
    let owner = "iawia002".to_owned();
    let repo = "lux".to_owned();

    let response = get_github_latest(&owner, &repo)?;
    let tag_name = jsonpath_lib::select(&response, "$.tag_name")?[0]
        .as_str()
        .unwrap();

    let version = tag_name[1..].to_owned();

    let download_assets = DownloadAssets {
        windows_x86_64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Windows_x86_64.zip")),
        windows_x86: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Windows_i386.zip")),
        macos_x86_64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Darwin_x86_64.zip")),
        macos_arm64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Darwin_arm64.zip")),
        linux_x86_64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Linux_x86_64.zip")),
    };

    let result = GithubLatestRelease {
        owner,
        repo,
        version,
        download_assets,
    };

    Ok(result)
}

fn get_youtubedl() -> Result<GithubLatestRelease> {
    let owner = "ytdl-org".to_owned();
    let repo = "youtube-dl".to_owned();

    let response = get_github_latest(&owner, &repo)?;
    let tag_name = jsonpath_lib::select(&response, "$.tag_name")?[0]
        .as_str()
        .unwrap();

    let version = tag_name.to_owned();

    let download_assets = DownloadAssets {
        windows_x86_64: Some(format!(
            "https://github.com/ytdl-org/youtube-dl/releases/download/{version}/youtube-dl.exe"
        )),
        windows_x86: Some(format!(
            "https://github.com/ytdl-org/youtube-dl/releases/download/{version}/youtube-dl.exe"
        )),
        ..Default::default()
    };

    let result = GithubLatestRelease {
        owner,
        repo,
        version,
        download_assets,
    };

    Ok(result)
}

fn _get_ytdlp() -> Result<GithubLatestRelease> {
    let owner = "yt-dlp".to_owned();
    let repo = "yt-dlp".to_owned();

    let response = get_github_latest(&owner, &repo)?;
    let tag_name = jsonpath_lib::select(&response, "$.tag_name")?[0]
        .as_str()
        .unwrap();

    let version = tag_name.to_owned();

    let download_assets = DownloadAssets {
        windows_x86_64: Some(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe".to_owned(),
        ),
        windows_x86: Some(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_x86.exe".to_owned(),
        ),
        macos_x86_64: Some(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos".to_owned(),
        ),
        macos_arm64: None,
        linux_x86_64: Some(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp".to_owned(),
        ),
    };

    let result = GithubLatestRelease {
        owner,
        repo,
        version,
        download_assets,
    };

    Ok(result)
}

fn get_youget() -> Result<GithubLatestRelease> {
    let owner = "soimort".to_owned();
    let repo = "you-get".to_owned();

    let response = get_github_latest(&owner, &repo)?;
    let tag_name = jsonpath_lib::select(&response, "$.tag_name")?[0]
        .as_str()
        .unwrap();

    let version = tag_name[1..].to_owned();

    let download_assets = DownloadAssets {
        ..Default::default()
    };

    let result = GithubLatestRelease {
        owner,
        repo,
        version,
        download_assets,
    };

    Ok(result)
}

pub fn get_filename_from_url(url: &str) -> Result<String> {
    let url = Url::parse(url)?;

    let filename = url
        .path_segments()
        .and_then(|x| x.last())
        .unwrap_or("")
        .to_string();

    Ok(filename)
}

use std::path::Path;

#[cfg(target_os = "windows")]
pub fn extract_file_to_plugin<S: AsRef<Path>>(file_path: S) -> Result<()> {
    use std::fs::File;
    use crate::downloader::get_plugin_dir;

    let plugin_dir = get_plugin_dir()?;
    let file = File::open(file_path.as_ref())?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_file() {
            let mut outfile = File::create(&plugin_dir.join(file.name()))?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn extract_file_to_plugin<S: AsRef<Path>>(_file: S) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_size_to_string() {
        assert_eq!("1.20 MB", size_to_string(1200000));
        assert_eq!("1.22 GB", size_to_string(1220000170));
        assert_eq!("120 B", size_to_string(120));
    }

    #[test]
    fn test_percent_to_string() {
        assert_eq!("22.4%", percent_to_string(0.2242));
    }

    #[test]
    fn test_eta_to_string() {
        assert_eq!("1m 40s", eta_to_string(100));
        assert_eq!(">= 2h", eta_to_string(2 * 60 * 60 + 500));
        assert_eq!(">= 1d", eta_to_string(24 * 60 * 60 + 666));
    }

    #[test]
    fn test_get_filename_from_url() {
        let url = "https://668000.xyz/sample.html";
        let filename = get_filename_from_url(url).unwrap();
        assert_eq!("sample.html", filename);

        let url = "https://668000.xyz/";
        let filename = get_filename_from_url(url).unwrap();
        assert_eq!("", filename);
    }
}
