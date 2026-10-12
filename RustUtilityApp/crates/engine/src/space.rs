use std::path::PathBuf;

use tokio::sync::Semaphore;

use crate::cleanup::directory_size;

/// One level of a size tree. `bytes` includes descendants. Concurrency cap is 4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceNode {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_directory: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceReport {
    pub root: PathBuf,
    pub total: u64,
    pub children: Vec<SpaceNode>,
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
            let bytes = if is_directory {
                let path = path.clone();
                tokio::task::spawn_blocking(move || directory_size(&path))
                    .await
                    .unwrap_or(0)
            } else {
                meta.map(|meta| meta.len()).unwrap_or(0)
            };
            SpaceNode {
                path,
                bytes,
                is_directory,
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
