use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LargeFile {
    pub path: PathBuf,
    pub bytes: u64,
}

/// Files under `~/Downloads` at or above `min_bytes`. Read-only. Skips symlinks.
pub fn large_downloads(home: &Path, min_bytes: u64) -> Vec<LargeFile> {
    let root = home.join("Downloads");
    let mut found = Vec::new();
    let mut stack = vec![root];
    let mut seen = 0usize;
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > 8_000 || found.len() >= 40 {
                found.sort_by(|a: &LargeFile, b: &LargeFile| b.bytes.cmp(&a.bytes));
                return found;
            }
            let path = entry.path();
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                if seen < 8_000 {
                    stack.push(path);
                }
            } else if meta.len() >= min_bytes {
                found.push(LargeFile {
                    path,
                    bytes: meta.len(),
                });
            }
        }
    }
    found.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    found
}