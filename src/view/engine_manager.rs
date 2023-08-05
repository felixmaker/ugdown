use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use fltk::{prelude::*, *};

use crate::{downloader::*, send_message, AppMessage};

use super::{tool_downloader::Task, utils::*, ToolDownloaderMessage};

mod ui {
    fl2rust_macro::include_ui!("./src/ui/engine_manager.fl");
}

#[derive(Clone)]
pub struct EngineManager {
    pub engine_manager: ui::UserInterface,
    current_download_asset: Option<HashMap<String, String>>,
}

impl EngineManager {
    pub fn default() -> Self {
        let mut engine_manager = ui::UserInterface::make_window();

        let current_download_asset: Option<HashMap<String, String>> = Default::default();

        engine_manager
            .choice_engine
            .add_choice(get_engine_names().join("|").as_str());

        engine_manager.choice_engine.set_value(0);

        engine_manager
            .choice_mirror
            .add_choice("github.com|ghproxy.com|kgithub.com");

        engine_manager.choice_mirror.set_value(0);

        let mut result = Self {
            engine_manager,
            current_download_asset,
        };

        result.bind_message();
        result
    }

    pub fn bind_message(&mut self) {
        self.engine_manager
            .btn_detect
            .set_callback(|_| send_message(EngineManagerMessage::Detect));
        self.engine_manager
            .btn_download
            .set_callback(|_| send_message(EngineManagerMessage::Download));
        self.engine_manager
            .btn_download_extra
            .set_callback(|_| send_message(EngineManagerMessage::DownloadExtra));
    }

    pub fn prepar_detecting(&mut self) {
        self.engine_manager
            .output_location
            .set_value("Detecting...");

        self.engine_manager
            .output_local_version
            .set_value("Detecting...");

        self.engine_manager.output_version.set_value("Detecting...");
        self.engine_manager.output_count.set_value("-");
        self.engine_manager.choice_assets.clear();
    }

    pub fn update_program_local_info(&mut self, location: &str, version: &str) {
        self.engine_manager.output_location.set_value(location);
        self.engine_manager.output_local_version.set_value(version);
    }

    pub fn update_program_info(&mut self, latest_release: &GithubLatestRelease) {
        self.engine_manager
            .output_version
            .set_value(&latest_release.version);

        let download_assets = latest_release.download_assets.to_order_vec();

        self.engine_manager
            .output_count
            .set_value(&download_assets.len().to_string());

        self.engine_manager.choice_assets.clear();
        self.engine_manager
            .choice_assets
            .add_choice(&download_assets.join("|"));

        self.engine_manager.choice_assets.set_value(0);

        self.current_download_asset = Some(latest_release.download_assets.to_hashmap());
    }

    pub fn detect(&mut self) {
        self.prepar_detecting();

        let engine_name = self.engine_manager.choice_engine.choice();
        if engine_name.is_none() {
            return;
        }
        let engine_name = engine_name.unwrap();

        std::thread::spawn(move || {
            let (location, version) = get_engine(&engine_name)
                .ok()
                .and_then(|x| x.get_program().ok())
                .map(|(path, version)| (path.to_string_lossy().to_string(), version))
                .unwrap_or(("Not Found!".to_owned(), "Unknown".to_owned()));

            send_message(EngineManagerMessage::UpdateLocal(location, version));

            if let Some(latest_release) = GithubLatestRelease::get_from_github_api(&engine_name) {
                send_message(EngineManagerMessage::UpdateOnline(latest_release));
            }
        });
    }

    fn replace_download_url(&self, url: &str) -> String {
        let mut url = url.to_owned();
        if let Some(choice_mirror) = self.engine_manager.choice_mirror.choice() {
            url = replace_github_download(&url, &choice_mirror);
        }
        url
    }

    fn get_asset_name(&self) -> String {
        self.engine_manager
            .choice_assets
            .choice()
            .unwrap_or("".to_owned())
    }

    fn get_asset_url(&self) -> Result<String> {
        let asset_name = self.get_asset_name();
        let url = self
            .current_download_asset
            .as_ref()
            .and_then(|x| x.get(&asset_name).cloned())
            .ok_or_else(|| anyhow::anyhow!("Not found in asset map"))?;
        Ok(url)
    }

    pub fn download(&self) -> Result<()> {
        let url = self.get_asset_url()?;
        let filename = get_filename_from_url(&url).unwrap_or("".to_owned());
        let output_path = select_store_path(&filename)?;

        let filename = &output_path
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or("%%%".to_owned());

        send_message(ToolDownloaderMessage::StartDownload(Task {
            url: self.replace_download_url(&url),
            output_path: output_path.to_string_lossy().to_string(),
            message: format!("Downloading {filename}..."),
            to_plugin: false,
        }));

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn download_to_plugins(&self) -> Result<()> {
        let url = self.get_asset_url()?;
        let filename = get_filename_from_url(&url).unwrap_or("".to_owned());

        let output_path = std::env::temp_dir().join(&filename);
        let filename = &output_path
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to download as filename unknown"))?;

        send_message(ToolDownloaderMessage::StartDownload(Task {
            url: self.replace_download_url(&url),
            output_path: output_path.to_string_lossy().to_string(),
            message: format!("Downloading {}...", filename),
            to_plugin: true,
        }));

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn download_and_extract(&self) -> Result<()> {
        self.download_to_plugins()?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn download_and_extract(&self) {}

    pub fn handle_message(&mut self, message: EngineManagerMessage) {
        match message {
            EngineManagerMessage::UpdateLocal(location, version) => {
                self.update_program_local_info(&location, &version)
            }
            EngineManagerMessage::UpdateOnline(latest_release) => {
                self.update_program_info(&latest_release)
            }
            EngineManagerMessage::Detect => self.detect(),
            EngineManagerMessage::Download => {
                let _ = self.download();
            }
            EngineManagerMessage::DownloadExtra => {
                let _ = self.download_and_extract();
            }
            EngineManagerMessage::Show => self.engine_manager.window.show(),
            EngineManagerMessage::Hide => self.engine_manager.window.hide(),
        }
    }
}

fn select_store_path(default_filename: &str) -> Result<PathBuf> {
    let mut dialog = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
    dialog.set_title("Save downloaded assert ...");
    dialog.set_preset_file(default_filename);

    if let Ok(plugins) = get_plugin_dir() {
        let _ = dialog.set_directory(&plugins);
    }

    dialog.show();
    let output_path = dialog.filename();

    if output_path.to_string_lossy().len() > 0 {
        Ok(output_path)
    } else {
        Err(anyhow::anyhow!("Output path should be set."))
    }
}

#[derive(Clone)]
pub enum EngineManagerMessage {
    UpdateLocal(String, String),
    UpdateOnline(GithubLatestRelease),
    Detect,
    Download,
    DownloadExtra,
    Show,
    Hide,
}

impl From<EngineManagerMessage> for AppMessage {
    fn from(value: EngineManagerMessage) -> Self {
        Self::EngineManager(value)
    }
}
