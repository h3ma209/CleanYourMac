use std::path::Path;

use async_trait::async_trait;

use crate::error::ScanError;
use crate::event::Detection;

/// Per-file check. Returning `Ok(None)` means the path was clean.
/// Returning `Ok(Some(_))` is a hit. This crate does not ship signature rules.
#[async_trait]
pub trait Detector: Send + Sync {
    async fn check(&self, path: &Path) -> Result<Option<Detection>, ScanError>;
}

/// Reads size and extension and never raises a hit.
pub struct MetadataDetector;

#[async_trait]
impl Detector for MetadataDetector {
    async fn check(&self, path: &Path) -> Result<Option<Detection>, ScanError> {
        let _ = file_meta(path).await?;
        Ok(None)
    }
}

/// Raises a hit when the file extension is in `watch` (compared without the dot, lowercased).
pub struct ExtensionWatchDetector {
    pub watch: Vec<String>,
}

impl ExtensionWatchDetector {
    pub fn new(watch: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            watch: watch
                .into_iter()
                .map(|s| s.as_ref().trim_start_matches('.').to_ascii_lowercase())
                .collect(),
        }
    }
}

#[async_trait]
impl Detector for ExtensionWatchDetector {
    async fn check(&self, path: &Path) -> Result<Option<Detection>, ScanError> {
        let (size, extension) = file_meta(path).await?;
        if self.watch.iter().any(|w| w == &extension) {
            Ok(Some(Detection {
                path: path.to_path_buf(),
                size,
                extension,
                note: "extension watch".into(),
            }))
        } else {
            Ok(None)
        }
    }
}

pub async fn file_meta(path: &Path) -> Result<(u64, String), ScanError> {
    let meta = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|source| ScanError::io(path, source))?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    Ok((meta.len(), extension))
}
