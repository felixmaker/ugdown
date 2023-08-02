use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use anyhow::Result;

mod lux;
mod youget;
mod youtubedl;

#[derive(Clone, Debug, Default)]
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
    pub cookies: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SaveOption {
    pub output_dir: String,
    pub file_name: String,
}

pub trait Downloader {
    fn get_downloader_name(&self) -> String;
    fn get_stream_info(
        &self,
        url: &str,
        cookie_file: Option<&Path>,
    ) -> Result<HashMap<String, DownloadInfo>>;
    fn execute_download(
        &self,
        url: &str,
        id: &str,
        output_dir: &str,
        output_name: &str,
        cookie_file: Option<&Path>,
    ) -> Result<Child>;
    fn is_stderr_output(&self) -> bool;
    fn get_program(&self) -> Result<(PathBuf, String)>;
}

pub fn store_cookies(cookies: &str) -> Result<PathBuf> {
    let cookie_id = uuid::Uuid::new_v4();
    let cookie_file = std::env::temp_dir().join(format!("cookie_{}.txt", cookie_id.to_string()));
    std::fs::write(&cookie_file, cookies)?;
    Ok(cookie_file)
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

pub fn get_stream_info(
    engine: &str,
    url: &str,
    cookie_file: Option<&Path>,
) -> Result<HashMap<String, DownloadInfo>> {
    let engine = get_engine(engine)?;
    engine.get_stream_info(url, cookie_file)
}

pub fn execute_download_info(
    download_info: &DownloadInfo,
) -> Result<(Child, Option<PathBuf>, bool)> {
    let download_info = download_info.clone();
    let (output_dir, output_name) = download_info
        .save_option
        .and_then(|x| Some((x.output_dir, x.file_name)))
        .unwrap_or((
            "./".to_owned(),
            format!("{}.{}", download_info.title, download_info.ext),
        ));

    let cookie_file = match download_info.cookies {
        Some(cookies) => Some(store_cookies(&cookies)?),
        None => None,
    };

    let engine = get_engine(&download_info.downloader)?;
    let url = download_info.url;
    let id = download_info.stream_id;
    Ok((
        engine.execute_download(&url, &id, &output_dir, &output_name, cookie_file.as_deref())?,
        cookie_file,
        engine.is_stderr_output(),
    ))
}

#[cfg(target_os = "windows")]
pub fn create_hide_window_command<S: AsRef<OsStr>>(program: S) -> Command {
    use std::os::windows::process::CommandExt;
    let program = get_exe_path(program);
    let mut command = Command::new(program);
    command.creation_flags(0x08000000);
    command
}

#[cfg(not(target_os = "windows"))]
pub fn create_hide_window_command<S: AsRef<OsStr>>(program: S) -> Command {
    let program = get_exe_path(program);
    Command::new(program)
}

pub fn get_exe_path<S: AsRef<OsStr>>(program: S) -> PathBuf {
    let mut program: PathBuf = program.as_ref().into();

    if let Some(plugin_dir) = get_plugin_dir() {
        let plugin = plugin_dir.join(&program);
        if let Ok(plugin) = which::which(plugin) {
            program = plugin
        }
    }

    program
}

pub fn get_plugin_dir() -> Option<PathBuf> {
    if let Some(user_dir) = directories::UserDirs::new() {
        if let Some(document_dir) = user_dir.document_dir() {
            let path = document_dir.join("ugdown/plugins");
            if path.is_dir() == false {
                let _ = std::fs::create_dir_all(&path);
            }
            return Some(path);
        }
    }
    None
}
