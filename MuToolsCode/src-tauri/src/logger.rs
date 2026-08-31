use chrono::Local;
use std::fs;
use std::io::Write;
use std::sync::Mutex;

use crate::config::AppState;

pub struct FileLogger {
    pub file: Mutex<fs::File>,
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Ok(mut file) = self.file.lock() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let line = format!("[{}] [{}] {}\n", timestamp, record.level(), record.args());
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

#[tauri::command]
pub fn log_action(state: tauri::State<AppState>, message: String) -> Result<(), String> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_line = format!("[{}] {}\n", timestamp, message);
    let log_file = state.log_dir.join("app.log");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .map_err(|e| format!("无法打开日志文件: {}", e))?;
    file.write_all(log_line.as_bytes())
        .map_err(|e| format!("无法写入日志: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_logs(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let log_file = state.log_dir.join("app.log");
    if !log_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&log_file).map_err(|e| format!("无法读取日志: {}", e))?;
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    Ok(lines)
}

#[tauri::command]
pub fn clear_logs(state: tauri::State<AppState>) -> Result<(), String> {
    let log_file = state.log_dir.join("app.log");
    fs::write(&log_file, "").map_err(|e| format!("无法清空日志: {}", e))?;
    Ok(())
}
