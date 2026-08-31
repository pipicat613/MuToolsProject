use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Seek, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use crate::config::AppState;

#[derive(Deserialize)]
pub struct DownloadResponse {
    pub errcode: i32,
    pub errmsg: String,
    pub data: Option<DownloadData>,
    #[allow(dead_code)]
    pub sign: Option<String>,
}

#[derive(Deserialize)]
pub struct DownloadData {
    #[serde(rename = "mumu")]
    pub mumu: Option<DownloadLink>,
    #[allow(dead_code)]
    pub engine: Option<String>,
    #[allow(dead_code)]
    pub version: Option<String>,
    #[allow(dead_code)]
    pub architecture: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct DownloadLink {
    pub link: String,
    pub checksum: String,
}

// V6 API 响应结构

#[derive(Deserialize, Serialize, Clone)]
pub struct V6DownloadResponse {
    pub errcode: i32,
    pub errmsg: String,
    pub data: Option<V6DownloadData>,
    #[allow(dead_code)]
    pub sign: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct V6DownloadData {
    #[allow(dead_code)]
    pub engine: Option<String>,
    pub version: Option<String>,
    #[allow(dead_code)]
    pub architecture: Option<String>,
    pub components: Option<Vec<V6Component>>,
    pub default_download_engine: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct V6Component {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub link: String,
    pub size: i64,
}

// 全局下载控制标志

static DOWNLOAD_CANCELLED: AtomicBool = AtomicBool::new(false);
static DOWNLOAD_PAUSED: AtomicBool = AtomicBool::new(false);
static ARIA2_PID: Mutex<Option<u32>> = Mutex::new(None);
static PARTIAL_FILES: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(true, Ordering::SeqCst);
    DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    if let Some(pid) = *ARIA2_PID.lock().unwrap() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    // 等待aria2/reqwest进程完全退出释放文件句柄
    std::thread::sleep(std::time::Duration::from_millis(200));
    // 删除所有已下载的部分文件
    let files = PARTIAL_FILES.lock().unwrap().clone();
    for f in &files {
        if f.exists() {
            // 重试
            let mut removed = false;
            for _attempt in 0..3 {
                match fs::remove_file(f) {
                    Ok(()) => {
                        log::info!("已删除取消下载的文件: {}", f.display());
                        removed = true;
                        break;
                    }
                    Err(e) => {
                        log::warn!("删除文件失败(将重试): {} - {}", f.display(), e);
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
            if !removed {
                // 最后一次尝试
                log::error!("无法删除文件: {}", f.display());
            }
        }
    }
    PARTIAL_FILES.lock().unwrap().clear();
    log::info!("下载已取消");
    Ok(())
}

#[tauri::command]
pub fn pause_download() -> Result<(), String> {
    DOWNLOAD_PAUSED.store(true, Ordering::SeqCst);
    if let Some(pid) = *ARIA2_PID.lock().unwrap() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    log::info!("下载已暂停");
    Ok(())
}

#[tauri::command]
pub fn resume_download() -> Result<(), String> {
    DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    log::info!("下载已恢复");
    Ok(())
}

// 版本下载链接获取

#[tauri::command]
pub async fn fetch_download_url(version: String) -> Result<String, String> {
    let link = match version.as_str() {
        "MuMu12-V4" => fetch_download_url_v4_inner().await?,
        "MuMu12-V5" => fetch_download_url_v5_inner().await?,
        "MuMu12-V6" => fetch_download_url_v6_inner().await?,
        _ => return Err("仅支持MuMu-V4V5V6版本获取".to_string()),
    };
    log::info!("获取 {} 下载链接成功: {}", version, link);
    Ok(link)
}

#[tauri::command]
pub async fn browse_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = mpsc::channel();
    app.dialog().file().pick_folder(move |d| {
        let _ = tx.send(d.and_then(|d| d.as_path().map(|p| p.to_string_lossy().to_string())));
    });
    let result = rx.recv().map_err(|e| format!("对话框错误: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub async fn browse_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = mpsc::channel();
    app.dialog()
        .file()
        .add_filter("安装包", &["exe", "7z", "zip", "rar", "mumudata"])
        .pick_file(move |d| {
            let _ = tx.send(d.and_then(|d| d.as_path().map(|p| p.to_string_lossy().to_string())));
        });
    let result = rx.recv().map_err(|e| format!("对话框错误: {}", e))?;
    Ok(result)
}

#[tauri::command]
pub fn start_install(
    version: String,
    method: String,
    path: String,
    install_dir: String,
) -> Result<String, String> {
    let method_str = if method == "local" {
        "本地安装"
    } else {
        "在线安装"
    };
    let msg = format!(
        "{} {} 安装任务已开始\n安装包路径: {}\n安装目录: {}",
        version, method_str, path, install_dir
    );
    log::info!("{}", msg);
    Ok(msg)
}

/// 查找 aria2c 可执行文件路径
fn find_aria2(resource_dir: &PathBuf) -> Option<PathBuf> {
    let mut candidates = vec![resource_dir.join("aria2")];

    if let Ok(entries) = fs::read_dir(resource_dir) {
        candidates.extend(entries.flatten().map(|entry| entry.path()).filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase().contains("aria2"))
                    == Some(true)
        }));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("aria2"));
            candidates.push(exe_dir.join("testdata").join("aria2"));
        }
    }

    candidates.push(PathBuf::from("testdata").join("aria2"));
    candidates.push(PathBuf::from("aria2"));

    candidates
        .into_iter()
        .flat_map(|dir| {
            [
                dir.join("aria2c.exe"),
                dir.join("aria2c"),
                dir.join("aria2.exe"),
            ]
        })
        .find(|path| path.is_file())
}

/// 发送下载进度事件
fn emit_progress(
    app: &tauri::AppHandle,
    status: &str,
    action: &str,
    percent: u32,
    downloaded: u64,
    total: u64,
) {
    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": status,
            "action": action,
            "percent": percent,
            "downloaded": downloaded,
            "total": total
        }),
    );
}

/// 返回下载后的文件完整路径
async fn download_file(
    app: &tauri::AppHandle,
    state: &AppState,
    url: &str,
    dir: &PathBuf,
    filename: &str,
    label: Option<&str>,
    known_size: u64,
    progress_offset: f64,
    progress_weight: f64,
    max_connections: u32,
    split: u32,
) -> Result<PathBuf, String> {
    // 检查是否已被取消
    if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
        return Err("下载已取消".to_string());
    }
    // 检查是否暂停
    if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
        return Err("下载已暂停".to_string());
    }

    let installer_path = dir.join(filename);
    let display_label = label.unwrap_or(filename);

    // 检查已有文件大小（用于断点续传时保留进度）
    let existing_size = if installer_path.exists() {
        fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    // 获取文件总大小
    let total_size = if known_size > 0 {
        known_size
    } else {
        let client = reqwest::Client::new();
        // 使用 Range 请求替代 HEAD，因为部分 CDN 不支持 HEAD 或
        // HEAD 不返回 Content-Length；Range 请求的 206 响应中
        // Content-Range 头会包含文件总大小
        match client.get(url).header("Range", "bytes=0-0").send().await {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                    resp.headers()
                        .get("content-range")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.split('/').last())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or_else(|| resp.content_length().unwrap_or(0))
                } else {
                    // 服务器不支持 Range，回退到 Content-Length
                    resp.content_length().unwrap_or(0)
                }
            }
            Err(_) => 0,
        }
    };

    log::info!(
        "开始下载: {} (url={}, total_size={} bytes, existing={} bytes)",
        display_label,
        url,
        total_size,
        existing_size
    );

    // 恢复下载时基于已有文件大小计算初始进度，避免进度条归零
    let initial_pct = if total_size > 0 && existing_size > 0 {
        ((progress_offset + progress_weight * (existing_size as f64 / total_size as f64)) * 100.0)
            .min(100.0) as u32
    } else {
        0u32
    };
    emit_progress(
        app,
        "downloading",
        &format!("正在下载 {}...", display_label),
        initial_pct,
        existing_size,
        total_size,
    );

    // 优先使用 aria2 下载
    let aria2 = find_aria2(&state.resource_dir);
    let download_result: Result<(), String> = if let Some(aria2_path) = aria2 {
        log::info!(
            "使用 aria2 下载: {} (max_connections={}, split={})",
            aria2_path.display(),
            max_connections,
            split
        );
        let ok = download_via_aria2(
            app,
            &aria2_path,
            url,
            dir,
            filename,
            &installer_path,
            total_size,
            display_label,
            progress_offset,
            progress_weight,
            max_connections,
            split,
        )
        .await;
        if ok {
            Ok(())
        } else if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            // 取消
            log::info!("下载已取消，不回退到内置下载器: {}", display_label);
            Err("下载已取消".to_string())
        } else if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
            // 暂停
            log::info!("下载已暂停，保留已下载部分: {}", display_label);
            Err("下载已暂停".to_string())
        } else {
            // aria2 自身失败（非暂停/取消）
            log::info!("aria2 失败，回退到内置下载器: {}", display_label);
            download_via_reqwest(
                app,
                url,
                &installer_path,
                total_size,
                display_label,
                progress_offset,
                progress_weight,
            )
            .await
        }
    } else {
        log::info!("未找到 aria2，使用内置下载器");
        download_via_reqwest(
            app,
            url,
            &installer_path,
            total_size,
            display_label,
            progress_offset,
            progress_weight,
        )
        .await
    };

    download_result?;

    let final_size = fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0);
    emit_progress(
        app,
        "downloading",
        &format!("{} 下载完成", display_label),
        100,
        final_size,
        final_size,
    );
    log::info!(
        "下载完成: {} ({} bytes)",
        installer_path.display(),
        final_size
    );

    Ok(installer_path)
}

/// 使用 aria2 下载文件，返回是否成功
async fn download_via_aria2(
    app: &tauri::AppHandle,
    aria2_path: &PathBuf,
    url: &str,
    dir: &PathBuf,
    filename: &str,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
    max_connections: u32,
    split: u32,
) -> bool {
    // 启动 aria2 进程
    // --file-allocation=none
    // --continue=true
    // --max-connection-per-server=N
    // --split=N
    // --console-log-level=error
    let mut child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new(aria2_path)
            .args([
                "--allow-overwrite=true",
                "--file-allocation=none",
                "--continue=true",
                "--max-connection-per-server",
                &max_connections.to_string(),
                "--split",
                &split.to_string(),
                "--console-log-level=error",
                "--dir",
                &dir.to_string_lossy(),
                "--out",
                filename,
                url,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    };

    // 保存 aria2 进程 PID 以便暂停/取消时 kill
    if let Ok(ref child) = child {
        *ARIA2_PID.lock().unwrap() = Some(child.id());
    }

    let mut success = false;
    if let Ok(ref mut child) = child {
        // 启动进度监控线程
        let app_clone = app.clone();
        let monitor_path = installer_path.clone();
        let label_owned = label.to_string();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let monitor_handle = std::thread::spawn(move || {
            let mut last_size = 0u64;
            while running_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(300));

                if DOWNLOAD_CANCELLED.load(Ordering::Relaxed)
                    || DOWNLOAD_PAUSED.load(Ordering::Relaxed)
                {
                    break;
                }

                if let Ok(metadata) = fs::metadata(&monitor_path) {
                    let current_size = metadata.len();
                    if current_size != last_size {
                        // 计算组件内进度百分比
                        let inner_pct = if total_size > 0 {
                            ((current_size as f64 / total_size as f64) * 100.0).min(100.0)
                        } else {
                            0.0
                        };
                        // 计算总进度
                        let total_pct = ((progress_offset + progress_weight * inner_pct / 100.0)
                            * 100.0)
                            .min(100.0) as u32;
                        emit_progress(
                            &app_clone,
                            "downloading",
                            &format!(
                                "正在下载 {} ({:.2} MB / {:.2} MB)",
                                label_owned,
                                current_size as f64 / 1024.0 / 1024.0,
                                total_size as f64 / 1024.0 / 1024.0
                            ),
                            total_pct,
                            current_size,
                            total_size,
                        );
                        last_size = current_size;
                    }
                }
            }
        });

        // 等待 aria2 完成
        let exit_status = child.wait();
        running.store(false, Ordering::Relaxed);
        let _ = monitor_handle.join();
        *ARIA2_PID.lock().unwrap() = None;

        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            return false;
        }
        if DOWNLOAD_PAUSED.load(Ordering::SeqCst) {
            return false;
        }

        match exit_status {
            Ok(status) if status.success() && installer_path.exists() => {
                success = true;
            }
            Ok(status) => {
                log::warn!(
                    "aria2 下载 {} 失败 (状态: {}), 回退到内置下载",
                    label,
                    status
                );
            }
            Err(e) => {
                log::warn!("aria2 等待 {} 失败: {}, 回退到内置下载", label, e);
            }
        }
    }

    success
}

/// 使用内置 reqwest 下载文件
async fn download_via_reqwest(
    app: &tauri::AppHandle,
    url: &str,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    // 检查已存在的部分下载文件，支持断点续传
    let existing_size = if installer_path.exists() {
        fs::metadata(installer_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let total_size = if total_size > 0 { total_size } else { 0 };

    // 如果已有文件且大小与 total_size 一致，视为已下载完成
    if existing_size > 0 && total_size > 0 && existing_size >= total_size {
        log::info!(
            "文件已存在且大小匹配，跳过下载: {} ({} bytes)",
            label,
            existing_size
        );
        return Ok(());
    }

    let mut request = client.get(url);
    let mut downloaded: u64 = 0;

    if existing_size > 0 {
        // 断点续传
        log::info!(
            "断点续传: {} 已有 {} bytes，从该位置继续下载",
            label,
            existing_size
        );
        request = request.header("Range", format!("bytes={}-", existing_size));
        downloaded = existing_size;
    }

    let resp = request
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    // 检查服务器是否支持断点续传
    let is_partial = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    if existing_size > 0 && !is_partial {
        // 服务器不支持 Range，从头下载
        log::warn!("服务器不支持断点续传，将从头开始下载: {}", label);
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("下载请求失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("下载失败: HTTP {}", resp.status()));
        }
        let final_total = if total_size > 0 {
            total_size
        } else {
            resp.content_length().unwrap_or(0)
        };
        return download_via_reqwest_stream(
            app,
            resp,
            installer_path,
            final_total,
            label,
            progress_offset,
            progress_weight,
            0,
        )
        .await;
    }

    if !resp.status().is_success() && !is_partial {
        return Err(format!("下载失败: HTTP {}", resp.status()));
    }

    let effective_total = if total_size > 0 {
        total_size
    } else if is_partial {
        // 从 Content-Range 头解析总大小
        resp.headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(resp.content_length().unwrap_or(0))
    } else {
        resp.content_length().unwrap_or(0)
    };

    download_via_reqwest_stream(
        app,
        resp,
        installer_path,
        effective_total,
        label,
        progress_offset,
        progress_weight,
        downloaded,
    )
    .await
}

/// reqwest 流式下载执行体
async fn download_via_reqwest_stream(
    app: &tauri::AppHandle,
    resp: reqwest::Response,
    installer_path: &PathBuf,
    total_size: u64,
    label: &str,
    progress_offset: f64,
    progress_weight: f64,
    initial_downloaded: u64,
) -> Result<(), String> {
    // 基于已下载大小计算初始进度，避免进度条归零
    let init_pct = if total_size > 0 {
        ((progress_offset + progress_weight * (initial_downloaded as f64 / total_size as f64))
            * 100.0)
            .min(100.0) as u32
    } else {
        0u32
    };
    emit_progress(
        app,
        "downloading",
        &format!("正在下载 {}...", label),
        init_pct,
        initial_downloaded,
        total_size,
    );

    // 追加模式打开文件（支持断点续传）
    // 使用 truncate=false 的写入模式
    // 避免在取消后重新下载时把已删除文件重建（取消后 initial_downloaded=0 时会新建空文件）
    let mut out = {
        let mut opts = fs::OpenOptions::new();
        opts.create(true).write(true);
        if initial_downloaded > 0 {
            opts.append(true);
        } else {
            opts.truncate(true);
        }
        opts.open(installer_path)
            .map_err(|e| format!("创建文件失败: {}", e))?
    };

    // 确保文件指针在末尾
    if initial_downloaded > 0 {
        out.seek(std::io::SeekFrom::End(0))
            .map_err(|e| format!("定位文件失败: {}", e))?;
    }

    let mut downloaded = initial_downloaded;
    let mut stream = resp.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        // 检查是否取消或暂停
        if DOWNLOAD_CANCELLED.load(Ordering::Relaxed) {
            return Err("下载已取消".to_string());
        }
        if DOWNLOAD_PAUSED.load(Ordering::Relaxed) {
            return Err("下载已暂停".to_string());
        }

        let chunk = chunk.map_err(|e| format!("下载数据失败: {}", e))?;
        out.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 限制进度事件频率
        if last_emit.elapsed() >= std::time::Duration::from_millis(200) {
            let inner_pct = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0).min(100.0)
            } else {
                0.0
            };
            let total_pct =
                ((progress_offset + progress_weight * inner_pct / 100.0) * 100.0).min(100.0) as u32;
            emit_progress(
                app,
                "downloading",
                &format!(
                    "正在下载 {} ({:.2} MB / {:.2} MB)",
                    label,
                    downloaded as f64 / 1024.0 / 1024.0,
                    total_size as f64 / 1024.0 / 1024.0
                ),
                total_pct,
                downloaded,
                total_size,
            );
            last_emit = std::time::Instant::now();
        }
    }

    Ok(())
}

/// 下载安装包到指定目录
#[tauri::command]
pub async fn download_v4_installer(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    save_dir: String,
    url: String,
    version: String,
    resume: bool,
) -> Result<String, String> {
    // 重置下载控制标志
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);
    if !resume {
        DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    }

    let url = if url.is_empty() {
        return Err("安装包下载链接不能为空".to_string());
    } else {
        url
    };
    log::info!("下载链接: {}", url);

    let dir = PathBuf::from(&save_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("无法创建下载目录: {}", e))?;

    let config = crate::config::read_config();

    // 使用版本特定文件名，避免 V4/V5 互相覆盖导致断点续传误判
    let filename = match version.as_str() {
        "MuMu12-V5" => "MuMu_Setup_V5.exe",
        _ => "MuMu_Setup_V4.exe",
    };

    let save_path = dir.join(filename);

    // 非 resume 时删除冲突文件，确保从零开始下载
    if !resume && save_path.exists() {
        let _ = fs::remove_file(&save_path);
        log::info!("已删除冲突文件: {}", save_path.display());
    }

    // 跟踪部分下载文件，供取消时删除
    PARTIAL_FILES.lock().unwrap().push(save_path.clone());

    let installer_path = download_file(
        &app,
        &state,
        &url,
        &dir,
        filename,
        None,
        0,
        0.0,
        1.0,
        config.aria2_max_connections,
        config.aria2_split,
    )
    .await;

    // 下载完成后从跟踪列表移除
    PARTIAL_FILES.lock().unwrap().retain(|f| f != &save_path);

    match installer_path {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => {
            // 取消下载时删除已下载的不完整文件
            if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) && save_path.exists() {
                match fs::remove_file(&save_path) {
                    Ok(()) => log::info!("取消下载，已删除不完整文件: {}", save_path.display()),
                    Err(err) => {
                        log::warn!("取消下载，删除文件失败: {} - {}", save_path.display(), err)
                    }
                }
            }
            Err(e)
        }
    }
}

/// V6 下载所有组件安装包到指定目录
#[tauri::command]
pub async fn download_v6_installer(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    save_dir: String,
    components_json: String,
    resume: bool,
) -> Result<String, String> {
    // 重置下载控制标志
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);
    if !resume {
        DOWNLOAD_PAUSED.store(false, Ordering::SeqCst);
    }

    let components: Vec<V6Component> =
        serde_json::from_str(&components_json).map_err(|e| format!("解析组件列表失败: {}", e))?;

    let dir = PathBuf::from(&save_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("无法创建下载目录: {}", e))?;

    let config = crate::config::read_config();

    let total_components = components.len();
    let mut downloaded_files = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        // 检查是否取消
        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            return Err("下载已取消".to_string());
        }

        let url = &comp.link;
        let filename = url
            .split('?')
            .next()
            .and_then(|v| v.rsplit('/').next())
            .unwrap_or(&comp.name)
            .to_string();
        let installer_path = dir.join(&filename);

        log::info!(
            "下载 V6 组件 [{}/{}]: {} -> {}",
            i + 1,
            total_components,
            comp.name,
            filename
        );

        // 非 resume 时，已存在的文件也要删除
        if !resume && installer_path.exists() {
            let _ = fs::remove_file(&installer_path);
            log::info!("已删除冲突文件: {}", installer_path.display());
        }

        // resume 时，已存在的文件跳过下载
        if resume && installer_path.exists() {
            let existing_size = fs::metadata(&installer_path).map(|m| m.len()).unwrap_or(0);
            if existing_size > 0 && comp.size > 0 && existing_size >= comp.size as u64 {
                log::info!(
                    "组件 {} 已存在且大小匹配，跳过下载: {}",
                    comp.name,
                    installer_path.display()
                );
                downloaded_files.push(installer_path.to_string_lossy().to_string());
                continue;
            }
            // 大小不匹配，继续下载（断点续传）
            log::info!(
                "组件 {} 部分下载，继续断点续传: {} (已有 {} bytes)",
                comp.name,
                installer_path.display(),
                existing_size
            );
        }

        // 跟踪部分下载文件，供取消时删除
        PARTIAL_FILES.lock().unwrap().push(installer_path.clone());

        // 每个组件占总进度的 1/total_components
        let weight = 1.0 / total_components as f64;
        let offset = i as f64 / total_components as f64;

        let path = download_file(
            &app,
            &state,
            url,
            &dir,
            &filename,
            Some(&comp.name),
            comp.size as u64,
            offset,
            weight,
            config.aria2_max_connections,
            config.aria2_split,
        )
        .await;

        match path {
            Ok(p) => {
                // 下载完成后从跟踪列表移除
                PARTIAL_FILES
                    .lock()
                    .unwrap()
                    .retain(|f| f != &installer_path);
                downloaded_files.push(p.to_string_lossy().to_string());
                log::info!("组件 {} 下载完成: {}", comp.name, p.display());
            }
            Err(e) => {
                // 取消下载时删除所有已下载的不完整文件
                if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
                    let files = PARTIAL_FILES.lock().unwrap().clone();
                    for f in &files {
                        if f.exists() {
                            match fs::remove_file(f) {
                                Ok(()) => log::info!("取消下载，已删除不完整文件: {}", f.display()),
                                Err(err) => {
                                    log::warn!("取消下载，删除文件失败: {} - {}", f.display(), err)
                                }
                            }
                        }
                    }
                    PARTIAL_FILES.lock().unwrap().clear();
                }
                return Err(e);
            }
        }
    }

    emit_progress(&app, "downloading", "所有组件下载完成", 100, 0, 0);

    Ok(serde_json::to_string(&downloaded_files).unwrap_or_default())
}

/// V6 静默安装
#[tauri::command]
pub async fn install_v6_setup(
    app: tauri::AppHandle,
    components_json: String,
    downloaded_files_json: String,
    install_dir: String,
    product_version: String,
) -> Result<String, String> {
    let components: Vec<V6Component> =
        serde_json::from_str(&components_json).map_err(|e| format!("解析组件列表失败: {}", e))?;
    let downloaded_files: Vec<String> = serde_json::from_str(&downloaded_files_json)
        .map_err(|e| format!("解析文件列表失败: {}", e))?;

    fs::create_dir_all(&install_dir).map_err(|e| format!("无法创建安装目录: {}", e))?;

    // 生成统一的 txn_id
    let txn_id = uuid::Uuid::new_v4().to_string();

    log::info!(
        "V6 安装开始: txn_id={}, product_version={}, install_dir={}",
        txn_id,
        product_version,
        install_dir
    );
    log::info!(
        "V6 组件数量: {}, 文件数量: {}",
        components.len(),
        downloaded_files.len()
    );

    // 将组件名匹配到下载的文件路径
    let mut component_file_map: Vec<(&V6Component, &String)> = Vec::new();
    for comp in &components {
        let comp_name_lower = comp.name.to_lowercase();
        let matched = downloaded_files.iter().find(|f| {
            let f_lower = f.to_lowercase();
            // 匹配文件名中包含组件名（小写）
            f_lower.contains(&comp_name_lower) || f_lower.contains(&format!("-{}", comp_name_lower))
        });
        if let Some(file_path) = matched {
            component_file_map.push((comp, file_path));
        } else {
            log::warn!("未找到组件 {} 的安装文件，跳过", comp.name);
        }
    }

    if component_file_map.is_empty() {
        return Err("没有找到任何可用的组件安装文件".to_string());
    }

    let total_steps = component_file_map.len() * 2; // install_prepare + commit 各一次
    let mut current_step = 0u32;

    // install_prepare
    for (comp, file_path) in &component_file_map {
        let install_dir_arg = install_dir.clone();
        let product_version_arg = product_version.clone();
        let txn_id_arg = txn_id.clone();

        current_step += 1;
        let action_msg = format!(
            "安装组件 {}/{}: {} (install_prepare)",
            current_step, total_steps, comp.name
        );
        log::info!("{}", action_msg);
        log::info!("命令行: \"{}\" /from_orchestrator=1 /action=install_prepare /txn_id={} /silent=1 /product_version={} /auto_start=false /fchannel=nochannel-mumu12 /auto_run=false /D={}", file_path, txn_id_arg, product_version_arg, install_dir_arg);

        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "installing",
                "action": action_msg,
                "percent": ((current_step as f64 / total_steps as f64) * 100.0) as u32,
                "downloaded": 0,
                "total": 0
            }),
        );

        let output = {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(file_path)
                .arg("/from_orchestrator=1")
                .arg("/action=install_prepare")
                .arg(format!("/txn_id={}", txn_id_arg))
                .arg("/silent=1")
                .arg(format!("/product_version={}", product_version_arg))
                .arg("/auto_start=false")
                .arg("/fchannel=nochannel-mumu12")
                .arg("/auto_run=false")
                .raw_arg(format!("/D={}", install_dir_arg))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("执行安装程序失败: {}", e))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            log::error!(
                "组件 {} install_prepare 失败 (exit: {}): stdout={}, stderr={}",
                comp.name,
                exit_code,
                stdout,
                stderr
            );
            return Err(format!(
                "组件 {} 安装失败 (exit: {})\n{}\n{}",
                comp.name, exit_code, stdout, stderr
            ));
        }
        log::info!("组件 {} install_prepare 成功", comp.name);
    }

    // commit
    for (comp, file_path) in component_file_map.iter().rev() {
        let install_dir_arg = install_dir.clone();
        let product_version_arg = product_version.clone();
        let txn_id_arg = txn_id.clone();

        current_step += 1;
        let action_msg = format!(
            "安装组件 {}/{}: {} (commit)",
            current_step, total_steps, comp.name
        );
        log::info!("{}", action_msg);
        log::info!("命令行: \"{}\" /from_orchestrator=1 /action=commit /txn_id={} /silent=1 /product_version={} /auto_start=false /fchannel=nochannel-mumu12 /auto_run=false /D={}", file_path, txn_id_arg, product_version_arg, install_dir_arg);

        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "installing",
                "action": action_msg,
                "percent": ((current_step as f64 / total_steps as f64) * 100.0) as u32,
                "downloaded": 0,
                "total": 0
            }),
        );

        let output = {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(file_path)
                .arg("/from_orchestrator=1")
                .arg("/action=commit")
                .arg(format!("/txn_id={}", txn_id_arg))
                .arg("/silent=1")
                .arg(format!("/product_version={}", product_version_arg))
                .arg("/auto_start=false")
                .arg("/fchannel=nochannel-mumu12")
                .arg("/auto_run=false")
                .raw_arg(format!("/D={}", install_dir_arg))
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("执行安装程序失败: {}", e))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            log::error!(
                "组件 {} commit 失败 (exit: {}): stdout={}, stderr={}",
                comp.name,
                exit_code,
                stdout,
                stderr
            );
            return Err(format!(
                "组件 {} commit 失败 (exit: {})\n{}\n{}",
                comp.name, exit_code, stdout, stderr
            ));
        }
        log::info!("组件 {} commit 成功", comp.name);
    }

    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": "completed",
            "action": "安装完成",
            "percent": 100,
            "downloaded": 0,
            "total": 0
        }),
    );

    log::info!("V6 安装完成: {}", install_dir);

    // 删除安装包
    let config = crate::config::read_config();
    if config.auto_delete_installer {
        for file_path in &downloaded_files {
            let p = PathBuf::from(file_path);
            if p.exists() {
                match fs::remove_file(&p) {
                    Ok(()) => log::info!("安装包已删除: {}", file_path),
                    Err(e) => log::warn!("删除安装包失败: {} - {}", file_path, e),
                }
            }
        }
    }

    Ok(format!("V6 安装成功\n安装目录: {}", install_dir))
}

/// 静默安装 Setup.exe /S /D=install_path
#[tauri::command]
pub async fn install_v4_setup(
    app: tauri::AppHandle,
    installer_path: String,
    install_dir: String,
) -> Result<String, String> {
    let installer = PathBuf::from(&installer_path);
    if !installer.exists() {
        return Err(format!("安装包不存在: {}", installer_path));
    }

    fs::create_dir_all(&install_dir).map_err(|e| format!("无法创建安装目录: {}", e))?;

    log::info!("开始静默安装: {} -> {}", installer_path, install_dir);
    log::info!("安装命令行: \"{}\" /S /D={}", installer_path, install_dir);
    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "status": "installing",
            "action": "正在静默安装，请稍候...",
            "percent": 0,
            "downloaded": 0,
            "total": 0
        }),
    );

    // NSIS 安装器要求 /D= 必须是最后一个参数，且路径不能加引号。
    // std::process::Command::arg() 会自动给含空格的参数加引号，破坏 NSIS 参数解析。
    // 使用 raw_arg() 直接传递原始参数字符串，避免引号问题。
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new(&installer)
            .arg("/S")
            .raw_arg(format!("/D={}", install_dir))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("执行安装程序失败: {}", e))?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "completed",
                "action": "安装完成",
                "percent": 100,
                "downloaded": 0,
                "total": 0
            }),
        );
        log::info!("安装成功: {}", install_dir);
        if !stdout.is_empty() {
            log::info!("安装 stdout: {}", stdout);
        }
        // 安装成功后，根据配置决定是否删除安装包
        let config = crate::config::read_config();
        if config.auto_delete_installer && installer.exists() {
            match fs::remove_file(&installer) {
                Ok(()) => log::info!("安装包已删除: {}", installer_path),
                Err(e) => log::warn!("删除安装包失败: {} - {}", installer_path, e),
            }
        }
        Ok(format!("安装成功\n安装目录: {}\n{}", install_dir, stdout))
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        log::error!(
            "安装失败 (exit: {}): stdout={}, stderr={}",
            exit_code,
            stdout,
            stderr
        );
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "status": "error",
                "action": format!("安装失败 (exit: {})", exit_code),
                "percent": 0,
                "downloaded": 0,
                "total": 0
            }),
        );
        Err(format!(
            "安装失败 (exit: {})\n{}\n{}",
            exit_code, stdout, stderr
        ))
    }
}

/// 获取 V4 下载链接
async fn fetch_download_url_v4_inner() -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4();
    let params = format!(
        "architecture=x86_64&channel=V4.1.31.3724&detectinfo=&downloader_version=3.1.14.0&has_installed=0&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n=MuMuInstaller_3.1.14_hoeCnNC&package=&product=&usage=0&uuid={}",
        uuid
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://mumu.nie.netease.com/api/v1/download/nemux")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    let download_resp: DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("解析响应失败: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "API返回错误: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let link = download_resp
        .data
        .and_then(|d| d.mumu)
        .map(|m| m.link)
        .ok_or_else(|| "响应中未找到下载链接".to_string())?;

    Ok(link)
}

/// 获取 V5 下载链接
async fn fetch_download_url_v5_inner() -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4();
    let random_suffix = uuid
        .to_string()
        .split('-')
        .next()
        .unwrap_or("default")
        .to_string();
    let n = format!("MuMu_5.0.2_{}", random_suffix);

    let params = format!(
        "architecture=x86_64&channel=nochannel-mumu12&detectinfo=&downloader_version=5.0.2&has_installed=0&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n={}&package=&product=&usage=0&uuid={}",
        n, uuid
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.mumu.nie.netease.com/api/v1/download/nx")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    let download_resp: DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("解析响应失败: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "API返回错误: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let link = download_resp
        .data
        .and_then(|d| d.mumu)
        .map(|m| m.link)
        .ok_or_else(|| "响应中未找到下载链接".to_string())?;

    Ok(link)
}

/// 内部函数
async fn fetch_download_url_v6_inner() -> Result<String, String> {
    let uuid_val = uuid::Uuid::new_v4();
    let random_suffix = uuid_val
        .to_string()
        .split('-')
        .next()
        .unwrap_or("default")
        .to_string();
    let n = format!("MuMu_6.0.1_{}", random_suffix);

    let params = format!(
        "architecture=x86_64&channel=nochannel-mumu12&detectinfo=&downloader_version=6.0.1&has_installed=1&language=zh-Hans&machine=%7B%0A%20%20%22base_board%22%3A%20%22Manufacturer%3AASUSTeK%20COMPUTER%20INC.%20Product%3ATUF%20GAMING%20B660M-PLUS%20WIFI%20D4%22%2C%0A%20%20%22cpu%22%3A%20%22Intel%20Core%20i5-12400F%206-Core%20Processor%22%2C%0A%20%20%22hard_disk%22%3A%20%5B%0A%20%20%20%20%22DRIVE_FIXED%28C%3A%5C%5C%29%3ATotal%20disk%20space%3A476.9GBFree%20disk%20space%3A198.3GB%22%2C%0A%20%20%20%20%22DRIVE_FIXED%28D%3A%5C%5C%29%3ATotal%20disk%20space%3A931.5GBFree%20disk%20space%3A612.4GB%22%0A%20%20%5D%2C%0A%20%20%22hyperv_opened%22%3A%201%2C%0A%20%20%22ip%22%3A%20%22202.173.8.23%22%2C%0A%20%20%22mac%22%3A%20%22FD%3A87%3A30%3A1F%3AC9%3A0B%22%2C%0A%20%20%22memory%22%3A%2016384%2C%0A%20%20%22os%22%3A%20%22Windows%2010%2064-bit%20Kernel%2010.0.19045%22%2C%0A%20%20%22screen%22%3A%20%7B%0A%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%22width%22%3A%201920%0A%20%20%7D%2C%0A%20%20%22screen_list%22%3A%20%5B%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%22dpr%22%3A%201%2C%0A%20%20%20%20%20%20%22height%22%3A%201080%2C%0A%20%20%20%20%20%20%22is_primary%22%3A%201%2C%0A%20%20%20%20%20%20%22width%22%3A%201920%0A%20%20%20%20%7D%0A%20%20%5D%2C%0A%20%20%22supported_install_arc%22%3A%20%22x86_64%22%2C%0A%20%20%22video%22%3A%20%5B%0A%20%20%20%20%22NVIDIA%20GeForce%20RTX%203060%22%0A%20%20%5D%2C%0A%20%20%22vt%22%3A%20%22%22%2C%0A%20%20%22vt_enabled%22%3A%201%2C%0A%20%20%22vt_supported%22%3A%200%0A%7D&n={}&package=&product=&usage=0&uuid={}",
        n, uuid_val
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.mumu.nie.netease.com/api/v2/download/nx")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(params)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    let download_resp: V6DownloadResponse =
        serde_json::from_str(&body).map_err(|e| format!("解析响应失败: {}", e))?;

    if download_resp.errcode != 100 {
        return Err(format!(
            "API返回错误: {} - {}",
            download_resp.errcode, download_resp.errmsg
        ));
    }

    let data = download_resp
        .data
        .ok_or_else(|| "响应中未找到data".to_string())?;
    let version = data.version.clone().unwrap_or_default();
    let components = data.components.clone().unwrap_or_default();
    let default_engine = data.default_download_engine.clone().unwrap_or_default();

    // 返回组件列表和版本信息
    let result = serde_json::json!({
        "version": version,
        "components": components,
        "defaultDownloadEngine": default_engine,
    });
    Ok(result.to_string())
}
