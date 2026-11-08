//! 启动项路径绝对/相对互转 基准就是配置文件旁边那层目录

use crate::models::UserConfig;
use std::path::{Path, PathBuf};

/// 相对路径拼上 base http(s) / 已经是绝对的直接原样还
pub fn resolve_path(base: &Path, stored: &Path) -> PathBuf {
    let s = stored.to_string_lossy();
    if s.starts_with("http://") || s.starts_with("https://") {
        return stored.to_path_buf();
    }
    if stored.is_absolute() {
        stored.to_path_buf()
    } else {
        base.join(stored)
    }
}

pub fn convert_all_to_absolute(config: &mut UserConfig, base: &Path) {
    for app in &mut config.apps {
        if let Some(ref ip) = app.icon_path {
            if !ip.is_absolute() {
                let joined = base.join(ip);
                if let Ok(canon) = dunce::canonicalize(&joined) {
                    app.icon_path = Some(canon);
                } else if joined.exists() {
                    app.icon_path = Some(joined);
                }
            }
        }
        if let Some(ref wd) = app.work_dir {
            if !wd.is_absolute() {
                let joined = base.join(wd);
                if let Ok(canon) = dunce::canonicalize(&joined) {
                    app.work_dir = Some(canon);
                } else if joined.exists() {
                    app.work_dir = Some(joined);
                }
            }
        }
        if app.web_url.is_some() {
            continue;
        }
        let pstr = app.path.to_string_lossy();
        if pstr.starts_with("http://") || pstr.starts_with("https://") {
            continue;
        }
        let joined = if app.path.is_absolute() {
            app.path.clone()
        } else {
            base.join(&app.path)
        };
        if let Ok(canonical) = dunce::canonicalize(&joined) {
            app.path = canonical;
        } else if joined.exists() {
            app.path = joined;
        }
    }
}

pub fn convert_all_to_relative(config: &mut UserConfig, base: &Path) {
    let base_canon = dunce::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    for app in &mut config.apps {
        if let Some(ref ip) = app.icon_path {
            let abs = if ip.is_absolute() {
                dunce::canonicalize(ip).unwrap_or_else(|_| ip.clone())
            } else {
                dunce::canonicalize(base.join(ip)).unwrap_or_else(|_| base.join(ip))
            };
            if let Ok(rel) = abs.strip_prefix(&base_canon) {
                app.icon_path = Some(rel.to_path_buf());
            }
        }
        if let Some(ref wd) = app.work_dir {
            let abs = if wd.is_absolute() {
                dunce::canonicalize(wd).unwrap_or_else(|_| wd.clone())
            } else {
                dunce::canonicalize(base.join(wd)).unwrap_or_else(|_| base.join(wd))
            };
            if let Ok(rel) = abs.strip_prefix(&base_canon) {
                app.work_dir = Some(rel.to_path_buf());
            }
        }
        if app.web_url.is_some() {
            continue;
        }
        let pstr = app.path.to_string_lossy();
        if pstr.starts_with("http://") || pstr.starts_with("https://") {
            continue;
        }
        let abs = if app.path.is_absolute() {
            dunce::canonicalize(&app.path).unwrap_or_else(|_| app.path.clone())
        } else {
            dunce::canonicalize(base.join(&app.path)).unwrap_or_else(|_| base.join(&app.path))
        };
        if let Ok(rel) = abs.strip_prefix(&base_canon) {
            app.path = rel.to_path_buf();
        }
    }
}
