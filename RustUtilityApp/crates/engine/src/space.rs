use std::collections::HashSet;
use std::os::darwin::fs::MetadataExt;
use std::path::{Path, PathBuf};

use tokio::sync::Semaphore;

use crate::cleanup::finder_trash;

/// `SF_DATALESS` from the Space Lens string `dataless`. A dataless file is cloud-evicted.
const SF_DATALESS: u32 = 0x40000000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudStatus {
    NotCloud,
    CloudLocal,
    CloudEvicted,
}

/// One level of a size tree. `bytes` includes descendants. Concurrency cap is 4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceNode {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_directory: bool,
    pub is_purgeable: bool,
    pub cloud: CloudStatus,
    pub leaves: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceReport {
    pub root: PathBuf,
    pub total: u64,
    pub children: Vec<SpaceNode>,
}

/// APFS data volume named in Space Lens (`/System/Volumes/Data/`). Falls back to `/`.
pub fn computer_root() -> PathBuf {
    let data = PathBuf::from("/System/Volumes/Data");
    if data.is_dir() {
        data
    } else {
        PathBuf::from("/")
    }
}

pub async fn scan_space(root: PathBuf) -> SpaceReport {
    let entries: Vec<PathBuf> = match std::fs::read_dir(&root) {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => Vec::new(),
    };
    let semaphore = std::sync::Arc::new(Semaphore::new(4));
    let mut joins = Vec::new();
    for path in entries {
        let permit = semaphore.clone();
        joins.push(tokio::spawn(async move {
            let _permit = permit.acquire_owned().await.ok();
            let meta = std::fs::symlink_metadata(&path).ok();
            let is_directory = meta.as_ref().is_some_and(|meta| meta.is_dir())
                && !meta.as_ref().is_some_and(|meta| meta.file_type().is_symlink());
            let flags = meta.as_ref().map(|meta| meta.st_flags()).unwrap_or(0);
            let cloud = cloud_status(&path, flags);
            let bytes = if is_directory {
                let path = path.clone();
                tokio::task::spawn_blocking(move || measure_tree(&path))
                    .await
                    .unwrap_or(0)
            } else {
                meta.map(|meta| meta.len()).unwrap_or(0)
            };
            SpaceNode {
                path,
                bytes,
                is_directory,
                is_purgeable: false,
                cloud,
                leaves: u64::from(!is_directory),
            }
        }));
    }
    let mut children = Vec::new();
    for join in joins {
        if let Ok(node) = join.await {
            children.push(node);
        }
    }
    children.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total = children.iter().fold(0u64, |sum, node| sum.saturating_add(node.bytes));
    SpaceReport {
        root,
        total,
        children,
    }
}

/// Sizes a directory without the junk-scan entry cap, and without counting the same inode twice.
fn measure_tree(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    let mut seen = HashSet::new();
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            let id = (meta.st_dev(), meta.st_ino());
            if !seen.insert(id) {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            } else {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

fn cloud_status(path: &Path, flags: u32) -> CloudStatus {
    if flags & SF_DATALESS != 0 {
        return CloudStatus::CloudEvicted;
    }
    let text = path.to_string_lossy();
    if text.contains("Mobile Documents") || text.contains("/CloudStorage/") {
        CloudStatus::CloudLocal
    } else {
        CloudStatus::NotCloud
    }
}

/// System locations and directories are spared. The original `shouldSpare` closure is supplied by the caller.
pub fn is_spared(path: &Path, home: &Path) -> bool {
    if path.is_dir() {
        return true;
    }
    let text = path.to_string_lossy();
    text.starts_with("/System")
        || text.starts_with("/usr")
        || text.starts_with("/bin")
        || text.starts_with("/sbin")
        || text.starts_with("/Library")
        || path.starts_with(home.join("Library"))
}

/// Finder trash for selected files inside the measured folder. Spared paths are skipped.
pub fn trash_selected(root: &Path, home: &Path, paths: &[PathBuf]) -> Result<usize, String> {
    let mut moved = 0usize;
    for path in paths {
        if is_spared(path, home) || !path.starts_with(root) || path == root || !path.is_file() {
            continue;
        }
        finder_trash(path)?;
        moved += 1;
    }
    if moved == 0 {
        Err("nothing removable was selected".into())
    } else {
        Ok(moved)
    }
}
