
use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::process::exit;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// 将 &str 转为以 null 结尾的 UTF-16 宽字符串
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
}

/// 检测管理员权限
pub fn is_admin() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);

        ok != 0 && elevation.TokenIsElevated != 0
    }
}

/// 管理员身份重启
pub fn relaunch_as_admin(args: &[String]) -> bool {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();
    if exe.is_empty() {
        return false;
    }

    let verb = to_wide("runas");
    let file = to_wide(&exe);

    // 构造命令行参数字符串
    let mut params = String::new();
    for a in args {
        if !params.is_empty() {
            params.push(' ');
        }
        if a.contains(' ') {
            params.push('"');
            params.push_str(a);
            params.push('"');
        } else {
            params.push_str(a);
        }
    }
    let params_wide = if params.is_empty() {
        Vec::new()
    } else {
        to_wide(&params)
    };
    let params_ptr = if params_wide.is_empty() {
        std::ptr::null()
    } else {
        params_wide.as_ptr()
    };

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            params_ptr,
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW 返回值 > 32 表示成功
    let success = result as usize > 32;
    if success {
        exit(0);
    }
    success
}

/// 供子进程检测是否是提权重启场景的标记参数
pub const ELEVATION_FLAG: &str = "--elevated";

/// 是否在命令行中携带了提权标记
pub fn has_elevation_flag() -> bool {
    std::env::args().any(|a| a == ELEVATION_FLAG)
}

/// 防止无限循环的安全检查
pub fn get_elevation_attempt() -> u32 {
    std::env::var("MUTOOLS_ELEVATION_ATTEMPT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// 设置提权尝试次数到环境变量，供子进程读取
pub fn set_elevation_attempt_env(n: u32) {
    std::env::set_var("MUTOOLS_ELEVATION_ATTEMPT", n.to_string());
}

/// 启动时执行提权检查
pub fn check_and_elevate(admin_elevation: bool) {
    if !admin_elevation {
        return;
    }

    // 已经是管理员权限，无需提权
    if is_admin() {
        if has_elevation_flag() {
            log::info!("以管理员权限重新启动成功");
        }
        return;
    }

    // dev 环境下跳过提权重启
    if is_dev_env() {
        log::info!("处于 dev 环境，非管理员权限，跳过自动提权");
        return;
    }

    // 防止无限重启循环
    let attempt = get_elevation_attempt();
    if attempt >= 3 {
        log::warn!("已尝试 {} 次，放弃提权", attempt);
        return;
    }

    // 构造子进程参数
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // 过滤掉可能已有的标记
    args.retain(|a| a != ELEVATION_FLAG);
    args.push(ELEVATION_FLAG.to_string());

    // 设置环境变量传递提权尝试次数
    set_elevation_attempt_env(attempt + 1);

    log::info!("第 {} 尝试以管理员权限重启", attempt + 1);

    // 尝试提权重启
    if !relaunch_as_admin(&args) {
        log::warn!("自动提权失败，将以普通权限继续运行");
    }
}

/// 检测是否处于dev
fn is_dev_env() -> bool {
    if cfg!(debug_assertions) {
        return true;
    }
    false
}

/// 供命令行调用
pub fn current_is_admin() -> bool {
    is_admin()
}
