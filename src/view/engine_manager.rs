use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use fltk::{prelude::*, *};
use url::Url;

use crate::downloader::{
    create_hide_window_command, get_engine, get_engine_names, get_exe_path, get_plugin_dir,
};

use super::tool_downloader::ToolDownloader;

mod ui {
    fl2rust_macro::include_ui!("./src/ui/engine_manager.fl");
}

pub struct EngineManager {
    engine_manager: ui::UserInterface,
    current_download_asset: Arc<Mutex<Option<HashMap<String, String>>>>,
    tool_downloader: ToolDownloader,
}

impl EngineManager {
    pub fn default() -> Self {
        let tool_downloader = ToolDownloader::default();
        let mut engine_manager = ui::UserInterface::make_window();
        let current_download_asset: Arc<Mutex<Option<HashMap<String, String>>>> =
            Default::default();

        engine_manager
            .choice_engine
            .add_choice(get_engine_names().join("|").as_str());

        engine_manager
            .choice_mirror
            .add_choice("kgithub.com|ghproxy.com");

        engine_manager.btn_detect.set_callback({
            let engine_manager = engine_manager.clone();
            let current_download_asset = current_download_asset.clone();
            move |_| {
                let mut engine_manager = engine_manager.clone();
                let current_download_asset = current_download_asset.clone();

                engine_manager.output_location.set_value("Detecting...");
                engine_manager
                    .output_local_version
                    .set_value("Detecting...");
                engine_manager.output_version.set_value("Detecting...");
                engine_manager.output_count.set_value("-");
                engine_manager.choice_assets.clear();

                {
                    *current_download_asset.lock().unwrap() = None;
                }

                std::thread::spawn(move || -> Result<()> {
                    if let Some(engine_name) = engine_manager.choice_engine.choice().clone() {
                        let (location, version) = get_engine(&engine_name)
                            .ok()
                            .and_then(|x| x.get_program().ok())
                            .map(|(path, version)| (path.to_string_lossy().to_string(), version))
                            .unwrap_or(("Not Found!".to_owned(), "Unknown".to_owned()));
                        engine_manager.output_location.set_value(&location);
                        engine_manager.output_local_version.set_value(&version);
                    }

                    let latest_release = engine_manager
                        .choice_engine
                        .choice()
                        .and_then(|x| GithubLatestRelease::get_from_github_api(&x));

                    if let Some(latest_release) = latest_release {
                        engine_manager
                            .output_version
                            .set_value(&latest_release.version);

                        let download_assets = latest_release.download_assets.to_hashmap();
                        let mut download_arch = Vec::new();
                        for (asset_arch, _) in &download_assets {
                            download_arch.push(asset_arch.clone())
                        }

                        engine_manager
                            .output_count
                            .set_value(&download_assets.len().to_string());

                        engine_manager
                            .choice_assets
                            .add_choice(&download_arch.join("|"));

                        *current_download_asset.lock().unwrap() = Some(download_assets);
                    }

                    Ok(())
                });
            }
        });

        engine_manager.btn_download.set_callback({
            let choice_assets = engine_manager.choice_assets.clone();
            let choice_mirror = engine_manager.choice_mirror.clone();
            let current_download_asset = current_download_asset.clone();
            let mut tool_downloader = tool_downloader.clone();
            move |_| {
                if let Some(engine) = choice_assets.choice() {
                    if let Some(current_download_asset) = &*current_download_asset.lock().unwrap() {
                        if let Some(url) = current_download_asset.get(&engine) {
                            if let Ok(result) = Url::parse(url) {
                                let filename = result
                                    .path_segments()
                                    .and_then(|x| x.last())
                                    .unwrap_or("")
                                    .to_string();

                                let mut dialog =
                                    dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
                                dialog.set_title("Save downloaded assert ...");
                                dialog.set_preset_file(&filename);

                                if let Some(plugins) = get_plugin_dir() {
                                    let _ = dialog.set_directory(&plugins);
                                }

                                dialog.show();
                                let output_path = dialog.filename();

                                let mut url = url.clone();
                                if let Some(choice_mirror) = choice_mirror.choice() {
                                    url = replace_github_download(&url, &choice_mirror);
                                }

                                if output_path.to_string_lossy().len() > 0 {
                                    let title = format!("Downloading {}...", filename);
                                    if let Err(err) = tool_downloader.start_download(
                                        &url,
                                        &output_path.to_string_lossy(),
                                        &title,
                                        &title,
                                    ) {
                                        dialog::alert_default(err.to_string().as_str());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let mut result = Self {
            engine_manager,
            current_download_asset,
            tool_downloader,
        };

        result.set_window_only_callback();

        result
    }

    pub fn show(&mut self) {
        self.engine_manager.window.show();
    }

    #[cfg(target_os = "windows")]
    pub fn set_window_only_callback(&mut self) {
        self.engine_manager.btn_download_extra.set_callback({
            let choice_assets = self.engine_manager.choice_assets.clone();
            let choice_mirror = self.engine_manager.choice_mirror.clone();
            let current_download_asset = self.current_download_asset.clone();
            let mut tool_downloader = self.tool_downloader.clone();
            move |_| {
                if let Some(engine) = choice_assets.choice() {
                    if let Some(current_download_asset) = &*current_download_asset.lock().unwrap() {
                        if let Some(url) = current_download_asset.get(&engine) {
                            if let Ok(result) = Url::parse(url) {
                                let filename = result
                                    .path_segments()
                                    .and_then(|x| x.last())
                                    .unwrap_or("")
                                    .to_string();

                                let output_path = std::env::temp_dir().join(&filename);
                                let mut url = url.clone();
                                if let Some(choice_mirror) = choice_mirror.choice() {
                                    url = replace_github_download(&url, &choice_mirror);
                                }
                                let title = format!("Downloading {}...", filename);
                                if let Err(err) = tool_downloader.start_download(
                                    &url,
                                    &output_path.to_string_lossy(),
                                    &title,
                                    &title,
                                ) {
                                    dialog::alert_default(err.to_string().as_str());
                                    return;
                                }

                                let sz_path = get_exe_path("7z");
                                if sz_path.is_file() == false {
                                    if let Some(plugin_dir) = get_plugin_dir() {
                                        tool_downloader.start_download(
                                            "https://www.7-zip.org/a/7zr.exe",
                                            &plugin_dir.join("7z.exe").to_string_lossy(),
                                            "Downloading 7z...",
                                            "Downloading 7z...",
                                        );
                                    }
                                }

                                if output_path.to_string_lossy().ends_with(".zip") {
                                    extract_file_by_7z(filename);
                                }

                                if output_path.to_string_lossy().ends_with(".exe") {
                                    std::fs::copy(output_path, get_plugin_dir().unwrap());
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    pub fn set_window_only_callback(&self) {}
}

fn get_github_latest(owner: &str, repo: &str) -> Result<serde_json::Value> {
    // https://docs.github.com/en/free-pro-team@latest/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let response: serde_json::Value = ureq::get(&url).call()?.into_json()?;
    Ok(response)
}

struct GithubLatestRelease {
    owner: String,
    repo: String,
    version: String,
    download_assets: DownloadAssets,
}

impl GithubLatestRelease {
    fn get_from_github_api(engine_name: &str) -> Option<Self> {
        match engine_name.to_lowercase().trim() {
            "lux" => get_lux().ok(),
            "youtube-dl" => get_youtubedl().ok(),
            "you-get" => get_youget().ok(),
            _ => None,
        }
    }
}

fn replace_github_download(origin: &str, mirror: &str) -> String {
    match mirror.to_ascii_lowercase().trim() {
        "kgithub" | "kgithub.com" => origin.replace("https://github.com", "https://kgithub.com"),
        "ghproxy" | "ghproxy.com" => {
            origin.replace("https://github.com", "https://ghproxy.com/github.com")
        }
        _ => origin.to_owned(),
    }
}

#[derive(Default)]
struct DownloadAssets {
    windows_x86_64: Option<String>,
    windows_x86: Option<String>,
    darwin_x86_64: Option<String>,
    darwin_arm64: Option<String>,
    linux_x86_64: Option<String>,
}

impl DownloadAssets {
    fn to_hashmap(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        if let Some(windows_x86_64) = &self.windows_x86_64 {
            result.insert("windows_x86_64".to_owned(), windows_x86_64.clone());
        }
        if let Some(windows_x86) = &self.windows_x86 {
            result.insert("windows_x86".to_owned(), windows_x86.clone());
        }
        if let Some(darwin_arm64) = &self.darwin_arm64 {
            result.insert("darwin_arm64".to_owned(), darwin_arm64.clone());
        }
        if let Some(darwin_x86_64) = &self.darwin_x86_64 {
            result.insert("darwin_x86_64".to_owned(), darwin_x86_64.clone());
        }
        if let Some(linux_x86_64) = &self.linux_x86_64 {
            result.insert("linux_x86_64".to_owned(), linux_x86_64.clone());
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
        darwin_x86_64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Darwin_x86_64.zip")),
        darwin_arm64: Some(format!("https://github.com/iawia002/lux/releases/download/v{version}/lux_{version}_Darwin_arm64.zip")),
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
        windows_x86_64: Some(
            format!("https://github.com/ytdl-org/youtube-dl/releases/download/{version}/youtube-dl.exe")
        ),
        windows_x86: Some(
            format!("https://github.com/ytdl-org/youtube-dl/releases/download/{version}/youtube-dl.exe")
        ),
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

fn get_ytdlp() -> Result<GithubLatestRelease> {
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
        darwin_x86_64: Some(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos".to_owned(),
        ),
        darwin_arm64: None,
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

fn extract_file_by_7z<S: AsRef<OsStr>>(file: S) -> Result<()> {
    if let Some(plugin_dir) = get_plugin_dir() {
        create_hide_window_command("7z")
            .arg("x")
            .arg(file)
            .arg(format!("-o{}", plugin_dir.to_string_lossy()))
            .output()?;
    }

    Ok(())
}
