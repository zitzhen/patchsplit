//! Patch retrieval behind the [`Downloader`] trait.
//!
//! Production code shells out to curl; tests can provide a different
//! implementation without touching the filesystem or PATH.

use std::process::Command;

use crate::error::AppError;

pub(crate) trait Downloader {
    fn fetch(&self, url: &str) -> Result<String, AppError>;
}

pub(crate) struct CurlDownloader;

impl Downloader for CurlDownloader {
    fn fetch(&self, url: &str) -> Result<String, AppError> {
        // Rust's standard library has no HTTPS client; calling curl keeps downloads simple.
        let output = Command::new("curl")
            .arg("--fail")
            .arg("--location")
            .arg("--silent")
            .arg("--show-error")
            .arg("--user-agent")
            .arg(format!("patchsplit/{}", patchsplit::version()))
            .arg(url)
            .output()
            .map_err(AppError::DownloadCommand)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(AppError::DownloadFailed {
                status: output.status.code(),
                message: stderr,
            });
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}
