use crate::models::{AppInfo, LaunchResult};
use std::path::Path;
use std::process::Command;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct AppLauncher;

impl AppLauncher {
    pub fn new() -> Self { Self }

 /// 普通启动
    pub fn launch(&self, app: &AppInfo) -> LaunchResult {
        if let Some(ref wu) = app.web_url {
            if !wu.trim().is_empty() {
                return match self.launch_via_shell_open(wu) {
                    Ok(()) => LaunchResult::Success,
                    Err(e) => LaunchResult::Failed(e),
                };
            }
        }

        let path = &app.path;
        let path_lossy = path.to_string_lossy();
        if let Ok(u) = url::Url::parse(path_lossy.as_ref()) {
            if matches!(u.scheme(), "http" | "https") {
                return match self.launch_via_shell_open(path_lossy.as_ref()) {
                    Ok(()) => LaunchResult::Success,
                    Err(e) => LaunchResult::Failed(e),
                };
            }
        }

        if !path.exists() {
            return LaunchResult::Failed(format!("路径不存在: {:?}", path));
        }

        let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());
        let result = match ext.as_deref() {
            Some("exe") => self.launch_exe(path, &app.args, app.work_dir.as_deref()),
            _ => self.launch_via_shell(path, &app.args, app.work_dir.as_deref()),
        };
        match result {
            Ok(_) => LaunchResult::Success,
            Err(e) => LaunchResult::Failed(e),
        }
    }

 /// 以管理员身份启动（UAC）
    pub fn launch_as_admin(&self, app: &AppInfo) -> Result<(), String> {
        win_shell::run_as_admin(&app.path, &app.args, app.work_dir.as_deref())
    }

 /// 打开文件所在目录并选中目标
    pub fn open_in_explorer(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(String::from("路径不存在"));
        }
        let arg = format!("/select,\"{}\"", path.to_string_lossy());
        let mut cmd = Command::new("explorer");
        cmd.raw_arg(std::ffi::OsString::from(arg))
            .creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map(|_| ()).map_err(|e| format!("打开位置失败：{}", e))
    }

 /// 显示文件属性对话框
    pub fn show_properties(&self, path: &Path) -> Result<(), String> {
        win_shell::show_properties(path)
    }

    fn launch_exe(&self, path: &Path, args: &[String], work_dir: Option<&Path>) -> Result<(), String> {
        let mut cmd = Command::new(path);
        for a in args { cmd.arg(a); }
        if let Some(dir) = work_dir {
            if dir.exists() { cmd.current_dir(dir); }
        } else if let Some(parent) = path.parent() {
            cmd.current_dir(parent);
        }
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().map(|_| ()).map_err(|e| format!("启动失败：{}", e))
    }

    fn launch_via_shell(&self, path: &Path, args: &[String], work_dir: Option<&Path>) -> Result<(), String> {
        let work = work_dir.or_else(|| path.parent());
        win_shell::shell_open_path(path, args, work)
    }

    fn launch_via_shell_open(&self, url_or_path: &str) -> Result<(), String> {
        win_shell::shell_open_target(url_or_path)
    }
}

// 子模块：Win32 工具
#[cfg(windows)]
pub mod win_shell {
    use std::path::Path;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::UI::Shell::{
        ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_INVOKEIDLIST, SEE_MASK_NOASYNC,
        SEE_MASK_NOCLOSEPROCESS,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    fn to_wide(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn path_to_wide(p: &Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    fn format_shell_args(args: &[String]) -> String {
        args.iter()
            .map(|a| {
                if a.contains(' ') || a.contains('\t') || a.contains('"') {
                    format!("\"{}\"", a.replace('"', "\\\""))
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn shell_open_path(path: &Path, args: &[String], work_dir: Option<&Path>) -> Result<(), String> {
        if !path.exists() { return Err(String::from("路径不存在")); }
        let file = path_to_wide(path);
        let args_str = format_shell_args(args);
        let args_w = if args_str.is_empty() { vec![0u16] } else { to_wide(&args_str) };
        let work_w = work_dir.filter(|p| p.exists()).map(path_to_wide);

        unsafe {
            let mut sei = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOASYNC,
                lpVerb: w!("open"),
                lpFile: PCWSTR(file.as_ptr()),
                lpParameters: if args_str.is_empty() { PCWSTR::null() } else { PCWSTR(args_w.as_ptr()) },
                lpDirectory: work_w.as_ref().map(|w| PCWSTR(w.as_ptr())).unwrap_or(PCWSTR::null()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            ShellExecuteExW(&mut sei).map_err(|e| format!("启动失败：{}", e))
        }
    }

    pub fn shell_open_target(target: &str) -> Result<(), String> {
        let target_w = to_wide(target);
        unsafe {
            let mut sei = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOASYNC,
                lpVerb: w!("open"),
                lpFile: PCWSTR(target_w.as_ptr()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            ShellExecuteExW(&mut sei).map_err(|e| format!("打开链接失败：{}", e))
        }
    }

    pub fn run_as_admin(path: &Path, args: &[String], work_dir: Option<&Path>) -> Result<(), String> {
        if !path.exists() { return Err(String::from("路径不存在")); }
        let file = path_to_wide(path);
        let args_str = format_shell_args(args);
        let args_w = if args_str.is_empty() { vec![0u16] } else { to_wide(&args_str) };
        let work_w = work_dir.filter(|p| p.exists()).map(path_to_wide);

        unsafe {
            let mut sei = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
                lpVerb: w!("runas"),
                lpFile: PCWSTR(file.as_ptr()),
                lpParameters: if args_str.is_empty() { PCWSTR::null() } else { PCWSTR(args_w.as_ptr()) },
                lpDirectory: work_w.as_ref().map(|w| PCWSTR(w.as_ptr())).unwrap_or(PCWSTR::null()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            let res = ShellExecuteExW(&mut sei).map_err(|e| format!("管理员启动失败：{}", e));
            if !sei.hProcess.is_invalid() {
                let _ = CloseHandle(sei.hProcess);
            }
            res
        }
    }

    pub fn show_properties(path: &Path) -> Result<(), String> {
        if !path.exists() { return Err(String::from("路径不存在")); }
        let file = path_to_wide(path);
        unsafe {
            let mut sei = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_INVOKEIDLIST | SEE_MASK_NOASYNC,
                lpVerb: w!("properties"),
                lpFile: PCWSTR(file.as_ptr()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            ShellExecuteExW(&mut sei).map_err(|e| format!("属性打开失败：{}", e))
        }
    }
}

#[cfg(not(windows))]
pub mod win_shell {
    use std::path::Path;
    pub fn shell_open_path(_p: &Path, _a: &[String], _w: Option<&Path>) -> Result<(), String> {
        Err(String::from("仅 Windows 支持"))
    }
    pub fn shell_open_target(_t: &str) -> Result<(), String> {
        Err(String::from("仅 Windows 支持"))
    }
    pub fn run_as_admin(_p: &Path, _a: &[String], _w: Option<&Path>) -> Result<(), String> {
        Err(String::from("仅 Windows 支持"))
    }
    pub fn show_properties(_p: &Path) -> Result<(), String> {
        Err(String::from("仅 Windows 支持"))
    }
}

