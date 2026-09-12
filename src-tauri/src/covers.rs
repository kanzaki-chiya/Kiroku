use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::redirect::Policy;
use tauri::{AppHandle, Emitter};

use crate::bangumi::is_allowed_cover_url;
use crate::error::AppError;
use crate::models::CoverReadyEvent;

const MAX_BYTES: usize = 5 * 1024 * 1024;

pub fn covers_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("covers")
}

fn cover_http() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .user_agent("Kiroku/0.1.0 (personal desktop client)")
        .timeout(Duration::from_secs(20))
        .redirect(Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                return attempt.error("too many redirects");
            }
            if is_allowed_cover_url(attempt.url().as_str()).is_ok() {
                attempt.follow()
            } else {
                attempt.error("redirect host not allowed")
            }
        }))
        .build()
        .map_err(|err| AppError::internal(format!("无法初始化封面下载：{err}")))
}

pub async fn download_cover(
    data_dir: &Path,
    local_id: i64,
    remote_url: &str,
) -> Result<String, AppError> {
    if remote_url.trim().is_empty() {
        return Err(AppError::validation("没有封面地址"));
    }
    let _checked = is_allowed_cover_url(remote_url)?;
    let response = cover_http()?.get(remote_url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::network(format!(
            "封面下载失败：{}",
            response.status()
        )));
    }
    if let Some(len) = response.content_length() {
        if len as usize > MAX_BYTES {
            return Err(AppError::validation("封面文件过大"));
        }
    }
    let bytes = response.bytes().await?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::validation("封面文件过大"));
    }
    let ext = sniff_extension(&bytes).ok_or_else(|| AppError::validation("封面不是支持的图片格式"))?;
    let dir = covers_dir(data_dir);
    tokio::fs::create_dir_all(&dir).await?;
    let filename = format!("{local_id}.{ext}");
    tokio::fs::write(dir.join(&filename), &bytes).await?;
    Ok(format!("covers/{filename}"))
}

fn sniff_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Some("png")
    } else if bytes.starts_with(b"GIF8") {
        Some("gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

pub async fn clear_cover_cache(data_dir: &Path) -> Result<(), AppError> {
    let dir = covers_dir(data_dir);
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            tokio::fs::remove_file(path).await?;
        }
    }
    Ok(())
}

pub fn emit_cover_ready(app: &AppHandle, bangumi_subject_id: i64, cover_local_path: String) {
    let _ = app.emit(
        "cover-ready",
        CoverReadyEvent {
            bangumi_subject_id,
            cover_local_path,
        },
    );
}
