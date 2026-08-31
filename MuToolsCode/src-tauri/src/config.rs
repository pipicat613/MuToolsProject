use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::elevation;

pub struct AppState {
    pub log_dir: PathBuf,
    pub resource_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataConfig {
    #[serde(default = "default_data_dir_mode")]
    pub data_dir_mode: String,
    #[serde(default)]
    pub data_dir_custom: String,
    #[serde(default = "default_admin_elevation")]
    pub admin_elevation: bool,
    #[serde(default = "default_aria2_max_connections")]
    pub aria2_max_connections: u32,
    #[serde(default = "default_aria2_split")]
    pub aria2_split: u32,
    #[serde(default = "default_auto_delete_installer")]
    pub auto_delete_installer: bool,
}

fn default_data_dir_mode() -> String { "appdata".to_string() }
fn default_admin_elevation() -> bool { true }
fn default_aria2_max_connections() -> u32 { 5 }
fn default_aria2_split() -> u32 { 5 }
fn default_auto_delete_installer() -> bool { false }

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            data_dir_mode: "appdata".to_string(),
            data_dir_custom: String::new(),
            admin_elevation: true,
            aria2_max_connections: 5,
            aria2_split: 5,
            auto_delete_installer: false,
        }
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("无法获取exe路径: {}", e))?
        .parent()
        .ok_or_else(|| "无法获取exe目录".to_string())?
        .to_path_buf();
    Ok(exe_dir.join("mutools_config.json"))
}

pub fn read_config() -> DataConfig {
    let config_path = match get_config_path() {
        Ok(p) => p,
        Err(_) => return DataConfig::default(),
    };
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<DataConfig>(&content) {
                return config;
            }
        }
    }
    DataConfig::default()
}

pub fn resolve_data_dir(config: &DataConfig) -> PathBuf {
    match config.data_dir_mode.as_str() {
        "exe_dir" => {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        }
        "custom" => {
            let custom = config.data_dir_custom.trim();
            if custom.is_empty() {
                // fallback to appdata
                resolve_appdata_dir()
            } else {
                PathBuf::from(custom)
            }
        }
        _ => {
            // "appdata" or default
            resolve_appdata_dir()
        }
    }
}

fn resolve_appdata_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("MuTools")
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        PathBuf::from(home).join(".mutools")
    } else {
        PathBuf::from(".")
    }
}

#[tauri::command]
pub fn get_data_dir() -> Result<String, String> {
    let config = read_config();
    let data_dir = resolve_data_dir(&config);
    Ok(data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_data_config(data_dir_mode: String, data_dir_custom: String) -> Result<(), String> {
    let existing = read_config();
    let config = DataConfig {
        data_dir_mode,
        data_dir_custom,
        admin_elevation: existing.admin_elevation,
        aria2_max_connections: existing.aria2_max_connections,
        aria2_split: existing.aria2_split,
        auto_delete_installer: existing.auto_delete_installer,
    };
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn save_admin_elevation(enabled: bool) -> Result<(), String> {
    let config_path = get_config_path()?;
    let mut config = read_config();
    config.admin_elevation = enabled;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn save_aria2_config(max_connections: u32, split: u32) -> Result<(), String> {
    let config_path = get_config_path()?;
    let mut config = read_config();
    config.aria2_max_connections = max_connections;
    config.aria2_split = split;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_aria2_config() -> Result<serde_json::Value, String> {
    let config = read_config();
    Ok(serde_json::json!({
        "maxConnections": config.aria2_max_connections,
        "split": config.aria2_split,
    }))
}

#[tauri::command]
pub fn check_admin_status() -> Result<bool, String> {
    Ok(elevation::current_is_admin())
}

#[tauri::command]
pub fn save_auto_delete_installer(enabled: bool) -> Result<(), String> {
    let config_path = get_config_path()?;
    let mut config = read_config();
    config.auto_delete_installer = enabled;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_auto_delete_installer() -> Result<bool, String> {
    let config = read_config();
    Ok(config.auto_delete_installer)
}