use encoding_rs::GBK;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::AppState;
use crate::mumu_info::{get_mumu_info, MuMuInfo};
use log;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 解码 Windows 命令行输出
fn decode_windows_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _encoding, had_errors) = GBK.decode(bytes);
            if had_errors {
                String::from_utf8_lossy(bytes).to_string()
            } else {
                cow.into_owned()
            }
        }
    }
}

fn get_hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

fn ensure_host_entry(hosts_path: &Path, line: &str) -> Result<(), String> {
    let content = fs::read_to_string(hosts_path)
        .map_err(|e| format!("读取 hosts 文件失败: {}", e))?;
    if !content.contains(line.trim()) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(hosts_path)
            .map_err(|e| format!("打开 hosts 文件失败: {}", e))?;
        // 如果文件末尾没有换行符，先补一个 \n，避免新条目与上一行拼接
        if !content.ends_with('\n') {
            write!(file, "\n").map_err(|e| format!("写入 hosts 文件失败: {}", e))?;
        }
        writeln!(file, "{}", line).map_err(|e| format!("写入 hosts 文件失败: {}", e))?;
    }
    Ok(())
}

fn remove_host_entry(hosts_path: &Path, line: &str) -> Result<(), String> {
    let content = fs::read_to_string(hosts_path)
        .map_err(|e| format!("读取 hosts 文件失败: {}", e))?;
    let trimmed = line.trim();
    let mut new_content = content
        .lines()
        .filter(|l| l.trim() != trimmed)
        .collect::<Vec<_>>()
        .join("\n");
    // 确保末尾有换行符，避免后续 ensure_host_entry 追加时拼接
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    fs::write(hosts_path, new_content)
        .map_err(|e| format!("写入 hosts 文件失败: {}", e))?;
    Ok(())
}

fn flush_dns() -> Result<(), String> {
    let output = Command::new("ipconfig")
        .args(["/flushdns"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 ipconfig /flushdns 失败: {}", e))?;
    if !output.status.success() {
        return Err(format!("刷新 DNS 失败: {}", decode_windows_output(&output.stderr)));
    }
    Ok(())
}

fn check_hosts_blocked(domain: &str) -> Result<bool, String> {
    let output = Command::new("ping")
        .args([domain, "-n", "1"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 ping 失败: {}", e))?;
    let stdout = decode_windows_output(&output.stdout);
    Ok(stdout.contains("127.0.0.1"))
}

fn get_install_dir_from_info(info: &MuMuInfo) -> Result<String, String> {
    info.install_dir
        .clone()
        .ok_or_else(|| "未能获取 MuMu 安装目录".to_string())
}

fn get_appdata_dir_from_info(info: &MuMuInfo) -> Result<String, String> {
    info.appdata_dir
        .clone()
        .ok_or_else(|| "未能获取 APPDATA 目录".to_string())
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(())
}

fn major_version(info: &MuMuInfo) -> String {
    info.major_version.clone().unwrap_or_default()
}

// 屏蔽更新域名

#[tauri::command]
pub async fn block_update_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 mumu.nie.netease.com",
        "127.0.0.1 g.fp.ps.netease.com",
    ];
    for entry in &entries {
        ensure_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if !check_hosts_blocked("g.fp.ps.netease.com")? {
        return Err("hosts 修改失败，仍能正常解析域名".to_string());
    }
    Ok("更新域名已屏蔽".to_string())
}

#[tauri::command]
pub async fn unblock_update_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 mumu.nie.netease.com",
        "127.0.0.1 g.fp.ps.netease.com",
    ];
    for entry in &entries {
        remove_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if check_hosts_blocked("g.fp.ps.netease.com").unwrap_or(false) {
        Err("解析仍未恢复，请检查网络或 hosts 配置".to_string())
    } else {
        Ok("更新域名已恢复".to_string())
    }
}

// 禁用启动图

#[tauri::command]
pub fn disable_startup_image() -> Result<String, String> {
    let info = get_mumu_info()?;
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let netease_appdata = PathBuf::from(&appdata_dir).join("Netease");

    // 兼容各版本启动图存储位置
    let candidates: Vec<PathBuf> = vec![
        netease_appdata.join("MuMuPlayer").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("startupImage"),
        netease_appdata.join("MuMuPlayer").join("data").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("data").join("startupImage"),
    ];

    let mut handled = 0usize;
    for candidate in &candidates {
        if candidate.is_dir() {
            fs::remove_dir_all(candidate).map_err(|e| format!("删除 startupImage 目录失败 ({}): {}", candidate.display(), e))?;
        } else if candidate.is_file() {
            // 已是文件
            handled += 1;
            continue;
        }
        // 目录不存在或已被删除：创建空文件占位
        ensure_parent_dir(candidate)?;
        fs::write(candidate, b"").map_err(|e| format!("创建 startupImage 文件失败 ({}): {}", candidate.display(), e))?;
        handled += 1;
    }

    if handled == 0 {
        return Err("未找到任何 startupImage 目录".to_string());
    }

    Ok("启动图已禁用".to_string())
}

#[tauri::command]
pub fn restore_startup_image() -> Result<String, String> {
    let info = get_mumu_info()?;
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let netease_appdata = PathBuf::from(&appdata_dir).join("Netease");

    let candidates: Vec<PathBuf> = vec![
        netease_appdata.join("MuMuPlayer").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("startupImage"),
        netease_appdata.join("MuMuPlayer").join("data").join("startupImage"),
        netease_appdata.join("MuMuPlayer-12.0").join("data").join("startupImage"),
    ];

    let mut restored = 0usize;
    for candidate in &candidates {
        if candidate.exists() && candidate.is_file() {
            fs::remove_file(candidate).map_err(|e| format!("删除 startupImage 文件失败 ({}): {}", candidate.display(), e))?;
            fs::create_dir_all(candidate).map_err(|e| format!("创建 startupImage 目录失败 ({}): {}", candidate.display(), e))?;
            restored += 1;
        }
    }

    if restored == 0 {
        Err("未找到 startupImage 文件，无需恢复".to_string())
    } else {
        Ok(format!("启动图已恢复，共处理 {} 个位置", restored))
    }
}

// 导入 Data 分区多开优化包

/// 资源包内用于优化的文件类型
fn find_optimize_files(pkg_dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(pkg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                        result.push(path);
                    }
                }
            }
        }
    }
    result
}

/// 列出可用于 Data 优化的已安装资源包
#[tauri::command]
pub fn list_data_optimize_packages(state: tauri::State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let rd = &state.resource_dir;
    if !rd.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(rd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let files = find_optimize_files(&path, &["mumudata"]);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let manifest = crate::packages::read_manifest(&path);
            let display_name = if !manifest.name.is_empty() { manifest.name.clone() } else { name.clone() };
            let descs = crate::packages::read_desc_files(&path);
            let version = descs.first().map(|d| d.display_version.clone()).unwrap_or_default();
            for file_path in files {
                let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                result.push(serde_json::json!({
                    "packageName": name,
                    "displayName": display_name,
                    "description": manifest.description,
                    "version": version,
                    "fileName": file_name,
                    "filePath": file_path.to_string_lossy().to_string(),
                }));
            }
        }
    }
    result.sort_by(|a, b| a["displayName"].as_str().unwrap_or("").to_lowercase().cmp(&b["displayName"].as_str().unwrap_or("").to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn import_data_package(path: String) -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let manager = info.mumu_manager_path.ok_or_else(|| "未找到 MuMuManager".to_string())?;

    if !major.starts_with('5') && !major.starts_with('6') && !major.starts_with('4') {
        return Err("不支持的版本".to_string());
    }

    let output = Command::new(&manager)
        .args(["import", "-p", &path, "-n", "1"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 MuMuManager import 失败: {}", e))?;

    let stdout = decode_windows_output(&output.stdout);
    if !output.status.success() {
        return Err(format!("导入失败: {}", decode_windows_output(&output.stderr)));
    }

    Ok(format!("Data 优化包已导入\n{}", stdout))
}

#[tauri::command]
pub fn undo_import_data_package() -> Result<String, String> {
    Ok("无法自动判断使用了 Data 优化包的多开实例，请手动删除相关实例".to_string())
}

// 禁用/恢复 MuMuPlayerUpdater.exe

#[tauri::command]
pub fn disable_updater() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;
    let major = major_version(&info);

    let updater_path = if major.starts_with('5') || major.starts_with('6') {
        PathBuf::from(&install_dir).join("nx_main").join("MuMuPlayerUpdater.exe")
    } else {
        PathBuf::from(&install_dir).join("shell").join("MuMuPlayerUpdater.exe")
    };

    let bak_path = updater_path.with_extension("exe.bak");

    if !updater_path.exists() && bak_path.exists() {
        // Already disabled
        return Ok("更新程序已处于禁用状态".to_string());
    }

    if updater_path.exists() {
        if bak_path.exists() {
            fs::remove_file(&updater_path).map_err(|e| format!("删除原更新程序失败: {}", e))?;
        } else {
            fs::rename(&updater_path, &bak_path).map_err(|e| format!("重命名更新程序失败: {}", e))?;
        }
    }

    fs::write(&updater_path, b"").map_err(|e| format!("创建空更新程序失败: {}", e))?;
    Ok("更新程序已禁用".to_string())
}

#[tauri::command]
pub fn restore_updater() -> Result<String, String> {
    let info = get_mumu_info()?;
    let install_dir = get_install_dir_from_info(&info)?;
    let major = major_version(&info);

    let updater_path = if major.starts_with('5') || major.starts_with('6') {
        PathBuf::from(&install_dir).join("nx_main").join("MuMuPlayerUpdater.exe")
    } else {
        PathBuf::from(&install_dir).join("shell").join("MuMuPlayerUpdater.exe")
    };

    let bak_path = updater_path.with_extension("exe.bak");

    if bak_path.exists() {
        if updater_path.exists() {
            fs::remove_file(&updater_path).map_err(|e| format!("删除空更新程序失败: {}", e))?;
        }
        fs::rename(&bak_path, &updater_path).map_err(|e| format!("恢复更新程序失败: {}", e))?;
        Ok("更新程序已恢复".to_string())
    } else {
        Err("未找到备份文件 .bak".to_string())
    }
}

// [4.x] 仿专版优化（overlay 替换）

/// 查找已安装的 7z.exe，优先从 AppState.resource_dir 找，回退到 exe 同级 resources
fn find_seven_zip(resource_dir: &Path) -> Result<PathBuf, String> {
    let primary = resource_dir.join("7zip").join("7z.exe");
    if primary.exists() {
        return Ok(primary);
    }
    let fallback = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|d| d.join("resources").join("7zip").join("7z.exe"))
        .unwrap_or_else(|| PathBuf::from("7z.exe"));
    if fallback.exists() {
        Ok(fallback)
    } else {
        Err("7z.exe 未找到，请先安装 7zip 资源包".to_string())
    }
}

/// 列出可用于 4.x 仿专版(overlay)优化的已安装资源包
#[tauri::command]
pub fn list_overlay_optimize_packages(state: tauri::State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let rd = &state.resource_dir;
    if !rd.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(rd) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let files = find_optimize_files(&path, &["7z", "zip"]);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let manifest = crate::packages::read_manifest(&path);
            let descs = crate::packages::read_desc_files(&path);
            let display_name = if !manifest.name.is_empty() { manifest.name.clone() } else { name.clone() };
            let version = descs.first().map(|d| d.display_version.clone()).unwrap_or_default();
            for file_path in files {
                let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                result.push(serde_json::json!({
                    "packageName": name,
                    "displayName": display_name,
                    "description": manifest.description,
                    "version": version,
                    "fileName": file_name,
                    "filePath": file_path.to_string_lossy().to_string(),
                }));
            }
        }
    }
    result.sort_by(|a, b| a["displayName"].as_str().unwrap_or("").to_lowercase().cmp(&b["displayName"].as_str().unwrap_or("").to_lowercase()));
    Ok(result)
}

#[tauri::command]
pub fn apply_overlay_package(state: tauri::State<AppState>, path: String) -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('4') {
        return Err("请使用其他方案".to_string());
    }
    let install_dir = get_install_dir_from_info(&info)?;
    let overlay_dir = PathBuf::from(&install_dir).join("overlay");
    let backup_dir = PathBuf::from(&install_dir).join("overlay_backup");

    if overlay_dir.exists() && !backup_dir.exists() {
        fs::rename(&overlay_dir, &backup_dir).map_err(|e| format!("备份 overlay 失败: {}", e))?;
        fs::create_dir_all(&overlay_dir).map_err(|e| format!("创建 overlay 目录失败: {}", e))?;
    } else if overlay_dir.exists() {
        fs::remove_dir_all(&overlay_dir).map_err(|e| format!("删除旧 overlay 失败: {}", e))?;
    }

    let seven_zip = find_seven_zip(&state.resource_dir)?;

    let output = Command::new(&seven_zip)
        .args(["x", "-y", &format!("-o{}", install_dir)])
        .arg(&path)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("解压 overlay 包失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("解压 overlay 包失败: {}", decode_windows_output(&output.stderr)));
    }

    Ok("仿专版 overlay 已应用".to_string())
}

#[tauri::command]
pub fn undo_overlay_package() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('4') {
        return Err("请使用其他方案".to_string());
    }
    let install_dir = get_install_dir_from_info(&info)?;
    let overlay_dir = PathBuf::from(&install_dir).join("overlay");
    let backup_dir = PathBuf::from(&install_dir).join("overlay_backup");

    if overlay_dir.exists() {
        fs::remove_dir_all(&overlay_dir).map_err(|e| format!("删除 overlay 失败: {}", e))?;
    }

    if backup_dir.exists() {
        fs::rename(&backup_dir, &overlay_dir).map_err(|e| format!("恢复 overlay 失败: {}", e))?;
    } else {
        fs::create_dir_all(&overlay_dir).map_err(|e| format!("创建空 overlay 失败: {}", e))?;
    }

    Ok("仿专版 overlay 已恢复".to_string())
}

// 修改 fchannel 渠道标识

#[tauri::command]
pub fn apply_fchannel() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('5') && !major.starts_with('6') {
        return Err("该功能仅支持 5.x 或 6.x 版本".to_string());
    }
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let config_path = PathBuf::from(&appdata_dir)
        .join("Netease")
        .join("MuMuPlayer")
        .join("install_config.json");

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 install_config.json 失败: {}", e))?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 install_config.json 失败: {}", e))?;

    let product = json
        .get_mut("product")
        .ok_or_else(|| "install_config.json 中缺少 product 节点".to_string())?;

    let current = product.get("fchannel").and_then(|v| v.as_str()).unwrap_or("");
    if current == "yx-yys" {
        return Ok("fchannel 已经是 yx-yys".to_string());
    }

    product["fchannel"] = serde_json::Value::String("yx-yys".to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&json).map_err(|e| format!("序列化失败: {}", e))?)
        .map_err(|e| format!("写入 install_config.json 失败: {}", e))?;

    Ok("fchannel 已修改为 yx-yys".to_string())
}

#[tauri::command]
pub fn undo_fchannel() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    if !major.starts_with('5') && !major.starts_with('6') {
        return Err("该功能仅支持 5.x 或 6.x 版本".to_string());
    }
    let appdata_dir = get_appdata_dir_from_info(&info)?;
    let config_path = PathBuf::from(&appdata_dir)
        .join("Netease")
        .join("MuMuPlayer")
        .join("install_config.json");

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 install_config.json 失败: {}", e))?;
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 install_config.json 失败: {}", e))?;

    let product = json
        .get_mut("product")
        .ok_or_else(|| "install_config.json 中缺少 product 节点".to_string())?;

    product["fchannel"] = serde_json::Value::String("nochannel-mumu12".to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&json).map_err(|e| format!("序列化失败: {}", e))?)
        .map_err(|e| format!("写入 install_config.json 失败: {}", e))?;

    Ok("fchannel 已恢复为 nochannel-mumu12".to_string())
}

// 修改 system.vdi 添加海外版标识

/// 根据大版本返回 system.vdi 路径列表
fn get_system_vdi_paths(install_dir: &str, major: &str) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    if major.starts_with('5') {
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("12.0")
            .join("vms")
            .join("MuMuPlayer-12.0-base")
            .join("system.vdi"));
    } else if major.starts_with('6') {
        // v6 同时处理 12.0 和 15.0 两个版本
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("12.0")
            .join("vms")
            .join("MuMuPlayer-12.0-base")
            .join("system.vdi"));
        paths.push(PathBuf::from(install_dir)
            .join("nx_device")
            .join("15.0")
            .join("vms")
            .join("MuMuPlayer-15.0-base")
            .join("system.vdi"));
    } else {
        return Err("该功能仅支持 5.x 或 6.x 版本".to_string());
    }
    Ok(paths)
}

const OVERSEAS_ORIGINAL: &str = "####################################\n# from generate-common-build-props\n# These properties identify this partition image.\n####################################";
const OVERSEAS_REPLACEMENT: &str = "####################################\n# from generate-common-build-props\n# These properties identify this partition image.\n#####\nro.build.version.overseas=true";

#[derive(Serialize, Deserialize)]
struct SystemVdiPatchRecord {
    start_pos: usize,
    #[serde(alias = "original_base64")]
    original_data: String,
    #[serde(alias = "md5")]
    md5_hash: String,
    timestamp: u64,
}

fn md5_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut context = md5::Context::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("读取文件失败: {}", e))?;
        if n == 0 { break; }
        context.consume(&buf[..n]);
    }
    Ok(format!("{:x}", context.compute()))
}

#[tauri::command]
pub async fn patch_system_vdi_overseas() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let install_dir = get_install_dir_from_info(&info)?;
    let vdi_paths = get_system_vdi_paths(&install_dir, &major)?;

    log::info!("=== 开始添加海外版标识, 版本: {}, 文件数: {} ===", major, vdi_paths.len());

    let mut results = Vec::new();
    for vdi_path in &vdi_paths {
        let vdi_path = vdi_path.clone();
        let record_path = vdi_path.with_extension("vdi.mutools_patch");

        log::info!("处理: {}", vdi_path.display());

        if !vdi_path.exists() {
            log::info!("跳过, 文件不存在: {}", vdi_path.display());
            results.push(format!("跳过 {} (文件不存在)", vdi_path.display()));
            continue;
        }

        // 已有补丁记录则跳过
        if record_path.exists() {
            log::info!("跳过, 已有补丁记录: {}", record_path.display());
            results.push(format!("跳过 {} (已被修改)", vdi_path.display()));
            continue;
        }

        log::info!("开始修改: {}", vdi_path.display());

        // 将大文件 I/O 移至 spawn_blocking 避免阻塞主线程
        let vdi_path_for_block = vdi_path.clone();
        let (md5_result, record) = tokio::task::spawn_blocking(move || -> Result<(String, SystemVdiPatchRecord), String> {
            let content = fs::read(&vdi_path_for_block).map_err(|e| format!("读取 system.vdi 失败: {}", e))?;
            log::info!("文件大小: {} bytes", content.len());

            let original_bytes = OVERSEAS_ORIGINAL.as_bytes();
            let start_pos = content.windows(original_bytes.len())
                .position(|window| window == original_bytes)
                .ok_or_else(|| "未找到匹配内容".to_string())?;

            log::info!("找到匹配位置: {}", start_pos);

            let end_pos = start_pos + original_bytes.len();
            let original_segment = content[start_pos..end_pos].to_vec();
            let replacement_bytes = OVERSEAS_REPLACEMENT.as_bytes();

            // 使用字节拼接方式替换（允许文件大小变化）
            let new_content = [&content[..start_pos], replacement_bytes, &content[end_pos..]].concat();
            fs::write(&vdi_path_for_block, &new_content).map_err(|e| format!("写入 system.vdi 失败: {}", e))?;

            let md5 = md5_file(&vdi_path_for_block)?;
            log::info!("新 MD5: {}", md5);

            let record = SystemVdiPatchRecord {
                start_pos,
                original_data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &original_segment),
                md5_hash: md5.clone(),
                timestamp: fs::metadata(&vdi_path_for_block)
                    .map_err(|e| format!("获取文件元数据失败: {}", e))?
                    .modified()
                    .map_err(|e| format!("获取修改时间失败: {}", e))?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("时间转换失败: {}", e))?
                    .as_secs(),
            };
            Ok((md5, record))
        })
        .await
        .map_err(|e| format!("VDI 修改任务失败: {}", e))??;

        fs::write(&record_path, serde_json::to_string_pretty(&record).map_err(|e| format!("序列化记录失败: {}", e))?)
            .map_err(|e| format!("写入记录文件失败: {}", e))?;

        log::info!("完成: {} (MD5: {})", vdi_path.display(), md5_result);
        results.push(format!("{} 已添加海外版标识, MD5: {}", vdi_path.display(), md5_result));
    }

    log::info!("=== 海外版标识添加完成 ===");
    Ok(results.join("\n"))
}

#[tauri::command]
pub async fn undo_patch_system_vdi_overseas() -> Result<String, String> {
    let info = get_mumu_info()?;
    let major = major_version(&info);
    let install_dir = get_install_dir_from_info(&info)?;
    let vdi_paths = get_system_vdi_paths(&install_dir, &major)?;

    log::info!("=== 开始还原海外版标识, 版本: {}, 文件数: {} ===", major, vdi_paths.len());

    let mut results = Vec::new();
    for vdi_path in &vdi_paths {
        let vdi_path = vdi_path.clone();
        let record_path = vdi_path.with_extension("vdi.mutools_patch");

        log::info!("处理: {}", vdi_path.display());

        if !vdi_path.exists() {
            log::info!("跳过, 文件不存在: {}", vdi_path.display());
            results.push(format!("跳过 {} (文件不存在)", vdi_path.display()));
            continue;
        }

        if !record_path.exists() {
            log::info!("跳过, 未找到补丁记录: {}", record_path.display());
            results.push(format!("跳过 {} (未找到补丁记录)", vdi_path.display()));
            continue;
        }

        log::info!("开始还原: {}", vdi_path.display());

        let record_content = fs::read_to_string(&record_path)
            .map_err(|e| format!("读取记录文件失败: {}", e))?;
        let record: SystemVdiPatchRecord = serde_json::from_str(&record_content)
            .map_err(|e| format!("解析记录文件失败: {}", e))?;

        log::info!("记录位置: {}, 原始 MD5: {}", record.start_pos, record.md5_hash);

        // 将大文件 I/O 移至 spawn_blocking 避免阻塞主线程
        let original_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &record.original_data)
            .map_err(|e| format!("解码原始字节失败: {}", e))?;
        let start_pos = record.start_pos;
        let expected_md5 = record.md5_hash.clone();
        let vdi_path_for_block = vdi_path.clone();
        let record_path_for_block = record_path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let current_md5 = md5_file(&vdi_path_for_block)?;
            log::info!("当前 MD5: {}", current_md5);
            if current_md5 != expected_md5 {
                return Err("文件已被其他程序修改，无法还原".to_string());
            }

            let replacement_bytes = OVERSEAS_REPLACEMENT.as_bytes();

            let content = fs::read(&vdi_path_for_block).map_err(|e| format!("读取 system.vdi 失败: {}", e))?;
            let end_pos = start_pos + replacement_bytes.len();

            // 检查位置是否有效
            if start_pos >= content.len() || end_pos > content.len() {
                return Err(format!("记录位置无效: 位置{} | 文件长度{}", start_pos, content.len()));
            }

            // 使用字节拼接方式还原原始内容
            let new_content = [&content[..start_pos], &original_bytes, &content[end_pos..]].concat();
            fs::write(&vdi_path_for_block, &new_content).map_err(|e| format!("写入 system.vdi 失败: {}", e))?;
            fs::remove_file(&record_path_for_block).map_err(|e| format!("删除记录文件失败: {}", e))?;

            log::info!("还原完成: {}", vdi_path_for_block.display());
            Ok(())
        })
        .await
        .map_err(|e| format!("VDI 还原任务失败: {}", e))??;

        results.push(format!("{} 海外版标识已还原", vdi_path.display()));
    }

    log::info!("=== 海外版标识还原完成 ===");
    Ok(results.join("\n"))
}

// 屏蔽数据上报域名

#[tauri::command]
pub async fn block_report_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 sentry.netease.com",
        "127.0.0.1 report.mumu.nie.netease.com",
    ];
    for entry in &entries {
        ensure_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if !check_hosts_blocked("report.mumu.nie.netease.com")? {
        return Err("hosts 修改失败，仍能正常解析域名".to_string());
    }
    Ok("数据上报域名已屏蔽".to_string())
}

#[tauri::command]
pub async fn unblock_report_domains() -> Result<String, String> {
    let hosts = get_hosts_path();
    let entries = [
        "127.0.0.1 sentry.netease.com",
        "127.0.0.1 report.mumu.nie.netease.com",
    ];
    for entry in &entries {
        remove_host_entry(&hosts, entry)?;
    }
    flush_dns()?;
    if check_hosts_blocked("report.mumu.nie.netease.com").unwrap_or(false) {
        Err("解析仍未恢复，请检查网络或 hosts 配置".to_string())
    } else {
        Ok("数据上报域名已恢复".to_string())
    }
}
