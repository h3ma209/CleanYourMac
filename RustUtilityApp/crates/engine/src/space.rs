use std::collections::HashSet;
use std::os::darwin::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::sync::Semaphore;

use crate::access::{folder_access, AccessStatus};
use crate::cleanup::finder_trash;

/// `SF_DATALESS`. The scan tests this bit of `st_flags` after `lstat`.
const SF_DATALESS: u32 = 0x40000000;

/// Top-level directory walks. Stored on the scan task. No second read was found in the scanner slice.
const MAX_CONCURRENT_SCANS: usize = 4;

/// Progress tracker constant stored next to `threshold`. Half a second in nanoseconds.
const PROGRESS_THRESHOLD_NS: u64 = 500_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudStatus {
    NotCloud,
    CloudLocal,
    CloudEvicted,
}

/// `CloudStorageHelper.strategy`. Raw byte `1` is the `lstat` path. Case names were not recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudScannerStrategy {
    UrlUbiquitousKeys,
    DatalessFlag,
}

#[derive(Clone, Debug)]
struct PathFilter {
    custom_ignore_paths: HashSet<PathBuf>,
    /// Present when full-disk access is denied. Paths under it are skipped.
    reminders_path: Option<PathBuf>,
}

struct ScanTask {
    filter: PathFilter,
    strategy: CloudScannerStrategy,
    #[allow(dead_code)]
    progress_threshold_ns: u64,
}

/// One directory entry. `children` is empty on the nodes returned to the UI.
/// Directory `bytes` is still the sum of descendant sizes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceNode {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_directory: bool,
    pub is_purgeable: bool,
    pub cloud: CloudStatus,
    pub leaves: u64,
    /// Seconds since UNIX epoch from the item's metadata, when available.
    pub modified: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceReport {
    pub root: PathBuf,
    pub total: u64,
    pub children: Vec<SpaceNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeletionUnit {
    pub path: PathBuf,
    pub size: u64,
    pub removable_leaves_count: u64,
}

/// Bubbles and ring views hide items below this share of the folder total.
pub const SPACE_LENS_BUBBLE_THRESHOLD: f32 = 0.02;

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
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let task = create_scan_task(&home);
    let root_dev = std::fs::symlink_metadata(&root).map(|meta| meta.st_dev()).unwrap_or(0);
    let seen = Arc::new(Mutex::new(HashSet::new()));
    if let Ok(meta) = std::fs::symlink_metadata(&root) {
        seen.lock().unwrap().insert((meta.st_dev(), meta.st_ino()));
    }
    let entries: Vec<PathBuf> = match std::fs::read_dir(&root) {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => Vec::new(),
    };
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_SCANS));
    let mut joins = Vec::new();
    for path in entries {
        if ignored(&task.filter, &path) {
            continue;
        }
        let permit = semaphore.clone();
        let seen = Arc::clone(&seen);
        let strategy = task.strategy;
        joins.push(tokio::spawn(async move {
            let _permit = permit.acquire_owned().await.ok();
            node_for_path(path, root_dev, strategy, &seen)
        }));
    }
    let mut children = Vec::new();
    for join in joins {
        if let Some(node) = join.await.ok().flatten() {
            children.push(node);
        }
    }
    children.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    children.retain(|node| node.bytes > 0 || node.is_directory);
    let total = sum_bytes(&children);
    SpaceReport { root, total, children }
}

fn create_scan_task(home: &Path) -> ScanTask {
    let reminders = home.join("Library/Reminders");
    let fda_granted = folder_access(&reminders) == AccessStatus::Granted
        || folder_access(&home.join("Library/Mail")) == AccessStatus::Granted;
    ScanTask {
        filter: PathFilter {
            custom_ignore_paths: HashSet::new(),
            reminders_path: if fda_granted { None } else { Some(reminders) },
        },
        strategy: CloudScannerStrategy::UrlUbiquitousKeys,
        progress_threshold_ns: PROGRESS_THRESHOLD_NS,
    }
}

fn node_for_path(
    path: PathBuf,
    root_dev: u64,
    strategy: CloudScannerStrategy,
    seen: &Mutex<HashSet<(u64, u64)>>,
) -> Option<SpaceNode> {
    let meta = std::fs::symlink_metadata(&path).ok()?;
    let same_volume = meta.st_dev() == root_dev;
    let is_link = meta.file_type().is_symlink();
    let is_directory = same_volume && meta.is_dir() && !is_link;
    let cloud = cloud_status(strategy, path_is_ubiquitous(&path), meta.st_flags() & SF_DATALESS != 0, meta.st_flags());
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    let (bytes, leaves) = if !same_volume {
        (0, 0)
    } else if is_directory {
        let measured = measure_tree(&path, root_dev, strategy, seen);
        (measured.bytes, measured.leaves)
    } else {
        let id = (meta.st_dev(), meta.st_ino());
        if seen.lock().ok()?.insert(id) {
            (allocated_bytes(&meta), leaf_count(false, cloud).0)
        } else {
            (0, 0)
        }
    };
    Some(SpaceNode {
        path,
        bytes,
        is_directory,
        is_purgeable: false,
        cloud,
        leaves,
        modified,
    })
}

struct Measured {
    bytes: u64,
    leaves: u64,
}

/// Directory size is the sum of child sizes. Symlinks are not descended.
/// The inode set stops a firmlink from being added twice.
fn measure_tree(
    path: &Path,
    root_dev: u64,
    strategy: CloudScannerStrategy,
    seen: &Mutex<HashSet<(u64, u64)>>,
) -> Measured {
    let mut bytes = 0u64;
    let mut leaves = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if ignored_nofollow(&path) {
                continue;
            }
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() || meta.st_dev() != root_dev {
                continue;
            }
            let id = (meta.st_dev(), meta.st_ino());
            if !seen.lock().unwrap().insert(id) {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            } else {
                let cloud = cloud_status(
                    strategy,
                    path_is_ubiquitous(&path),
                    meta.st_flags() & SF_DATALESS != 0,
                    meta.st_flags(),
                );
                bytes = bytes.saturating_add(allocated_bytes(&meta));
                leaves = leaves.saturating_add(leaf_count(false, cloud).0);
            }
        }
    }
    Measured { bytes, leaves }
}

/// Purgeable node contributes 0. A file contributes 1. Callers sum directories.
fn leaf_count(is_purgeable: bool, cloud: CloudStatus) -> (u64, u64) {
    if is_purgeable {
        return (0, 0);
    }
    let removable = u64::from(cloud != CloudStatus::CloudEvicted);
    (1, removable)
}

fn sum_bytes(children: &[SpaceNode]) -> u64 {
    children.iter().fold(0u64, |sum, node| sum.saturating_add(node.bytes))
}

/// URL strategy: not ubiquitous → on this Mac, not downloaded → evicted, else local.
/// Dataless strategy: `SF_DATALESS` → evicted, every other file → local.
pub fn cloud_status(
    strategy: CloudScannerStrategy,
    is_ubiquitous: bool,
    not_downloaded: bool,
    st_flags: u32,
) -> CloudStatus {
    if strategy == CloudScannerStrategy::DatalessFlag {
        return if st_flags & SF_DATALESS != 0 {
            CloudStatus::CloudEvicted
        } else {
            CloudStatus::CloudLocal
        };
    }
    if !is_ubiquitous {
        CloudStatus::NotCloud
    } else if not_downloaded || st_flags & SF_DATALESS != 0 {
        CloudStatus::CloudEvicted
    } else {
        CloudStatus::CloudLocal
    }
}

fn path_is_ubiquitous(path: &Path) -> bool {
    path.components().any(|component| component.as_os_str() == "Mobile Documents")
}

fn ignored(filter: &PathFilter, path: &Path) -> bool {
    if filter.custom_ignore_paths.contains(path) {
        return true;
    }
    if filter
        .reminders_path
        .as_ref()
        .is_some_and(|reminders| path.starts_with(reminders))
    {
        return true;
    }
    ignored_nofollow(path)
}

fn ignored_nofollow(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text == "/.nofollow" || text.starts_with("/.nofollow/")
}

/// Both `fileSize` and `totalFileAllocatedSize` are requested by the scanner.
/// The stored key was not recovered. Disk usage here is allocated blocks, 512 bytes each.
fn allocated_bytes(meta: &std::fs::Metadata) -> u64 {
    meta.st_blocks().saturating_mul(512)
}

/// Flat rows for a delete pass. `should_spare` drops a node and everything under it.
pub fn deletion_units(nodes: &[SpaceNode], should_spare: impl Fn(&SpaceNode) -> bool) -> Vec<DeletionUnit> {
    nodes
        .iter()
        .filter(|node| !should_spare(node) && !node.is_directory)
        .map(|node| DeletionUnit {
            path: node.path.clone(),
            size: node.bytes,
            removable_leaves_count: u64::from(node.cloud != CloudStatus::CloudEvicted && !node.is_purgeable),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_strategy_matches_ubiquitous_and_dataless() {
        assert_eq!(
            cloud_status(CloudScannerStrategy::UrlUbiquitousKeys, false, false, 0),
            CloudStatus::NotCloud
        );
        assert_eq!(
            cloud_status(CloudScannerStrategy::UrlUbiquitousKeys, true, true, 0),
            CloudStatus::CloudEvicted
        );
        assert_eq!(
            cloud_status(CloudScannerStrategy::UrlUbiquitousKeys, true, false, 0),
            CloudStatus::CloudLocal
        );
        assert_eq!(
            cloud_status(CloudScannerStrategy::DatalessFlag, false, false, SF_DATALESS),
            CloudStatus::CloudEvicted
        );
        assert_eq!(
            cloud_status(CloudScannerStrategy::DatalessFlag, false, false, 0),
            CloudStatus::CloudLocal
        );
    }

    #[test]
    fn nofollow_and_reminders_are_ignored() {
        let filter = PathFilter {
            custom_ignore_paths: HashSet::from([PathBuf::from("/tmp/skip")]),
            reminders_path: Some(PathBuf::from("/Users/example/Library/Reminders")),
        };
        assert!(ignored(&filter, Path::new("/.nofollow")));
        assert!(ignored(&filter, Path::new("/.nofollow/secret")));
        assert!(ignored(&filter, Path::new("/tmp/skip")));
        assert!(ignored(&filter, Path::new("/Users/example/Library/Reminders/a.ics")));
        assert!(!ignored(&filter, Path::new("/Users/example/Documents/a.txt")));
    }

    #[test]
    fn purgeable_and_evicted_files_do_not_count_as_removable() {
        assert_eq!(leaf_count(true, CloudStatus::NotCloud), (0, 0));
        assert_eq!(leaf_count(false, CloudStatus::NotCloud), (1, 1));
        assert_eq!(leaf_count(false, CloudStatus::CloudEvicted), (1, 0));
    }

    #[test]
    fn deletion_units_drop_spared_directories() {
        let nodes = vec![
            SpaceNode {
                path: PathBuf::from("/Users/example/file.txt"),
                bytes: 10,
                is_directory: false,
                is_purgeable: false,
                cloud: CloudStatus::NotCloud,
                leaves: 1,
                modified: None,
            },
            SpaceNode {
                path: PathBuf::from("/Users/example/Folder"),
                bytes: 40,
                is_directory: true,
                is_purgeable: false,
                cloud: CloudStatus::NotCloud,
                leaves: 2,
                modified: None,
            },
        ];
        let units = deletion_units(&nodes, |node| node.is_directory);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].size, 10);
        assert_eq!(units[0].removable_leaves_count, 1);
    }

    #[tokio::test]
    async fn scan_sums_a_nested_file_and_skips_root_nofollow() {
        let dir = std::env::temp_dir().join(format!("rust_utility_space_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub/b.txt"), vec![b'x'; 8192]).unwrap();
        let report = scan_space(dir.clone()).await;
        let sub = report.children.iter().find(|node| node.path.ends_with("sub")).unwrap();
        assert!(sub.is_directory);
        assert!(sub.bytes >= 8192);
        assert_eq!(sub.leaves, 1);
        assert_eq!(report.total, sub.bytes);
        assert!(ignored_nofollow(Path::new("/.nofollow/item")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
