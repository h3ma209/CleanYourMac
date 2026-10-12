use std::path::{Path, PathBuf};

use crate::cleanup::directory_size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudFolder {
    pub label: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub exists: bool,
}

/// Local cloud folders only. No account login and no delete.
pub fn local_cloud_folders(home: &Path) -> Vec<CloudFolder> {
    [
        ("iCloud Drive", home.join("Library/Mobile Documents")),
        ("Cloud storage", home.join("Library/CloudStorage")),
    ]
    .into_iter()
    .map(|(label, path)| {
        let exists = path.is_dir();
        let bytes = if exists { directory_size(&path) } else { 0 };
        CloudFolder {
            label: label.to_string(),
            path,
            bytes,
            exists,
        }
    })
    .collect()
}
