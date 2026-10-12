use std::path::{Path, PathBuf};
use std::process::Command;

pub fn junk_candidates(home: &Path) -> Vec<(&'static str, PathBuf)> {
    vec![
        ("User caches", home.join("Library/Caches")),
        ("User logs", home.join("Library/Logs")),
        ("Diagnostic reports", home.join("Library/Logs/DiagnosticReports")),
        (
            "Spotify cache",
            home.join("Library/Application Support/Spotify/PersistentCache/Storage"),
        ),
        (
            "Xcode derived data",
            home.join("Library/Developer/Xcode/DerivedData"),
        ),
    ]
}

/// User-level cache and log roots named in the cleanup notes.
/// Survey is read-only. Trash is a separate call and only accepts these roots.
pub fn junk_roots(home: &Path) -> Vec<JunkRoot> {
    junk_candidates(home)
        .into_iter()
        .map(|(label, path)| {
            let exists = path.is_dir();
            let bytes = if exists { directory_size(&path) } else { 0 };
            JunkRoot {
                label: label.to_string(),
                path,
                bytes,
                exists,
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JunkRoot {
    pub label: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub exists: bool,
}

pub fn directory_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    let mut seen = 0usize;
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > 80_000 {
                return total;
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
                stack.push(path);
            } else {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

/// Finder trash, matching `trashItemAtURL`. Refuses anything outside the surveyed roots.
pub fn move_root_to_trash(home: &Path, path: &Path) -> Result<(), String> {
    let allowed = junk_candidates(home)
        .into_iter()
        .any(|(_, root)| root == path && path.is_dir());
    if !allowed {
        return Err("only a surveyed cache or log root can be moved to the Trash".into());
    }
    let text = path.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(r#"tell application "Finder" to delete POSIX file "{text}""#);
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
