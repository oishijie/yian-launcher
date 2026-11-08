//! 去网站扒 favicon，统一压成 64×64 的 PNG

use image::ImageFormat;
use std::io::{Cursor, Read};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const READ_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_ICON_BYTES: u64 = 2 * 1024 * 1024;

/// 下下来转 PNG 搞不定就 None，调用方自己兜底
pub fn download_site_icon_png(web_url: &str) -> Option<Vec<u8>> {
    let parsed = url::Url::parse(web_url.trim()).ok()?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let host = parsed.host_str()?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(CONNECT_TIMEOUT)
        .timeout_read(READ_TIMEOUT)
        .timeout(Duration::from_secs(5))
        .build();

    let fav_url = parsed.join("/favicon.ico").ok()?;
    if let Some(raw) = try_fetch_bytes(&agent, fav_url.as_str()) {
        if let Some(png) = normalize_image_bytes_to_png(&raw) {
            return Some(png);
        }
    }

    for candidate in fallback_icon_urls(host) {
        if let Some(raw) = try_fetch_bytes(&agent, &candidate) {
            if let Some(png) = normalize_image_bytes_to_png(&raw) {
                return Some(png);
            }
        }
    }

    None
}

fn fallback_icon_urls(domain: &str) -> [String; 2] {
    [
        format!("https://icons.duckduckgo.com/ip3/{}.ico", domain),
        format!("https://www.google.com/s2/favicons?sz=64&domain={}", domain),
    ]
}

fn try_fetch_bytes(agent: &ureq::Agent, url: &str) -> Option<Vec<u8>> {
    let resp = agent
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) YianLauncher/1.0")
        .call()
        .ok()?;
    let status = resp.status();
    if !(200..400).contains(&status) {
        return None;
    }
    let mut buf = Vec::new();
    resp.into_reader()
        .take(MAX_ICON_BYTES)
        .read_to_end(&mut buf)
        .ok()?;
    if buf.len() < 8 {
        return None;
    }
    Some(buf)
}

fn normalize_image_bytes_to_png(raw: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(raw).ok()?;
    let resized = img.resize_exact(64, 64, image::imageops::FilterType::Triangle);
    let mut cursor = Cursor::new(Vec::new());
    resized.write_to(&mut cursor, ImageFormat::Png).ok()?;
    Some(cursor.into_inner())
}
