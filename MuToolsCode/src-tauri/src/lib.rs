use std::fs;
use std::sync::Mutex;
use tauri::Manager;

mod config;
mod downloader;
mod elevation;
mod logger;
mod mumu_info;
mod optimize;
mod packages;

use config::read_config;
use config::resolve_data_dir;
use config::AppState;
use logger::FileLogger;

pub fn run() {
    // 读取配置提权检查
    let config = read_config();
    elevation::check_and_elevate(config.admin_elevation);

    let data_dir = resolve_data_dir(&config);
    let log_dir = data_dir.join("log");
    let resource_dir = data_dir.join("resources");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            fs::create_dir_all(&log_dir).expect("无法创建日志目录");
            fs::create_dir_all(&resource_dir).expect("无法创建资源目录");

            let log_file_path = log_dir.join("app.log");
            let log_file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
                .expect("无法创建日志文件");

            let logger = FileLogger {
                file: Mutex::new(log_file),
            };
            log::set_boxed_logger(Box::new(logger))
                .map(|()| log::set_max_level(log::LevelFilter::Info))
                .expect("无法设置日志器");

            log::info!("MuTools OK!");
            log::info!("data_dir: {}", data_dir.display());
            log::info!("resource_dir: {}", resource_dir.display());
            log::info!("log_dir: {}", log_dir.display());

            app.manage(AppState {
                log_dir,
                resource_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            logger::log_action,
            logger::get_logs,
            logger::clear_logs,
            downloader::fetch_download_url,
            downloader::browse_directory,
            downloader::browse_file,
            downloader::start_install,
            packages::list_resource_packages,
            packages::install_resource_from_path,
            packages::install_resource_from_url,
            packages::delete_resource_package,
            packages::delete_sub_package,
            packages::open_resource_dir,
            packages::open_url,
            downloader::download_v4_installer,
            downloader::install_v4_setup,
            downloader::download_v6_installer,
            downloader::install_v6_setup,
            downloader::cancel_download,
            downloader::pause_download,
            downloader::resume_download,
            config::get_data_dir,
            config::save_data_config,
            config::save_admin_elevation,
            config::check_admin_status,
            config::save_aria2_config,
            config::get_aria2_config,
            config::save_auto_delete_installer,
            config::get_auto_delete_installer,
            mumu_info::get_mumu_info,
            optimize::block_update_domains,
            optimize::unblock_update_domains,
            optimize::disable_startup_image,
            optimize::restore_startup_image,
            optimize::import_data_package,
            optimize::undo_import_data_package,
            optimize::list_data_optimize_packages,
            optimize::disable_updater,
            optimize::restore_updater,
            optimize::apply_overlay_package,
            optimize::undo_overlay_package,
            optimize::list_overlay_optimize_packages,
            optimize::apply_fchannel,
            optimize::undo_fchannel,
            optimize::patch_system_vdi_overseas,
            optimize::undo_patch_system_vdi_overseas,
            optimize::block_report_domains,
            optimize::unblock_report_domains,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
