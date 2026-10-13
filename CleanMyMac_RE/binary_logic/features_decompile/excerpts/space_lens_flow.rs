//! Recovered Space Lens control flow from `SpaceLensScanning_arm64`
//! (`sha256 52ebeda49910709c64095f0c19f2e2af5db40b10d12bc573c8d5c867410fabaf`).
//!
//! This is not original source. Each step is static decompiler output.
//! Lines marked `inferred` are not a direct instruction match.
//!
//! Evidence:
//! - directory walk `FUN_00008134` — `ev_ded40391b2dd93a3e0ce3660bc2a9c51e99de05838db8f0ae6675c90e48f54b2`
//! - async caller `FUN_00007b54` — `ev_e6cad4fcc7fe7130df200fc8a92f7773f4dbd57e5ff13e76c3469d7d1fdfce29`
//! - cloud classifier `FUN_0000236c` — `ev_0b5155e5e47707fbe76c95489f9aba39abd97538e7b4e6232bf5cea4f6158386`
//! - `leavesCount` `0x3fe0` — `ev_7b8ee6a6e1c6ccd0426d399575f00e7faf9d5a7a69334afc6003983856d603bf`
//! - `removableLeavesCount` `0x4460` — `ev_105b9f01d7bcfd1d2b9d5a84f8f98741cab2206b55eb0237a65c04435cf8d1bc`

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// `SF_DATALESS`. Walk tests `st_flags` bit 30 after `lstat`.
const SF_DATALESS: u32 = 0x40000000;

/// Reflection order plus classifier returns 0, 1, 2. Number-to-name link is inferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CloudStatus {
    NotCloud = 0,
    CloudEvicted = 1,
    CloudLocal = 2,
}

/// Byte stored on `CloudStorageHelper.strategy`.
/// Raw value `1` selects the `lstat` / `SF_DATALESS` path. Case names were not recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudScannerStrategy {
    UrlUbiquitousKeys,
    /// Observed raw value `1`.
    DatalessFlag,
}

/// Layout observed from the 0x28-byte node written by `FUN_00008134`
/// and read by `leavesCount` / `removableLeavesCount`.
///
/// ```text
/// +0x00 path
/// +0x10 size
/// +0x18 is_directory
/// +0x19 is_purgeable
/// +0x1a cloud_status
/// +0x20 children
/// ```
#[derive(Clone, Debug)]
pub struct Entity {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
    pub is_purgeable: bool,
    pub cloud_status: CloudStatus,
    pub children: Vec<Entity>,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub path: PathBuf,
    pub size: u64,
    pub removable_leaves_count: i64,
}

#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub scanned_size: u64,
    pub current_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ResourceValues {
    pub file_size: Option<u64>,
    pub total_file_allocated_size: Option<u64>,
    pub is_directory: bool,
    pub is_regular_file: bool,
    pub is_symbolic_link: bool,
    pub is_ubiquitous_item: bool,
    /// True when status equals `NSURLUbiquitousItemDownloadingStatusNotDownloaded`.
    pub ubiquitous_not_downloaded: bool,
}

pub struct PathFilter {
    pub is_fda_granted: bool,
    pub custom_ignore_paths: HashSet<PathBuf>,
    /// Set from `NSHomeDirectory()` + `/Library/Reminders` when FDA is denied.
    pub reminders_path: Option<PathBuf>,
}

pub struct ScanTask {
    pub directory: PathBuf,
    pub filter: PathFilter,
    pub strategy: CloudScannerStrategy,
    /// Stored at `0xd3bc`. No second read site in this slice.
    pub max_concurrent_scans: u64,
    /// Stored beside `totalScannedSize` / `delta` / `threshold`. 0.5s is inferred.
    pub progress_threshold_ns: u64,
}

pub fn create_scan_task(
    directory: PathBuf,
    strategy: CloudScannerStrategy,
    fda_granted: bool,
    home: &Path,
) -> ScanTask {
    let reminders_path = if fda_granted {
        None
    } else {
        Some(home.join("Library/Reminders"))
    };
    ScanTask {
        directory,
        filter: PathFilter {
            is_fda_granted: fda_granted,
            custom_ignore_paths: HashSet::new(),
            reminders_path,
        },
        strategy,
        max_concurrent_scans: 4,
        progress_threshold_ns: 500_000_000,
    }
}

/// `FUN_0000236c`, or the `lstat` branch when `strategy` byte is 1.
pub fn cloud_status(strategy: CloudScannerStrategy, values: &ResourceValues, st_flags: u32) -> CloudStatus {
    if strategy == CloudScannerStrategy::DatalessFlag {
        return if st_flags & SF_DATALESS != 0 {
            CloudStatus::CloudEvicted
        } else {
            CloudStatus::CloudLocal
        };
    }
    if !values.is_ubiquitous_item {
        CloudStatus::NotCloud
    } else if values.ubiquitous_not_downloaded {
        CloudStatus::CloudEvicted
    } else {
        CloudStatus::CloudLocal
    }
}

fn ignored(filter: &PathFilter, path: &Path) -> bool {
    if filter.custom_ignore_paths.contains(path) {
        return true;
    }
    let text = path.to_string_lossy();
    text == "/.nofollow" || text.starts_with("/.nofollow/")
}

/// Both `NSURLFileSizeKey` and `NSURLTotalFileAllocatedSizeKey` are requested.
/// The stored qword was not recovered (decompiler compares an untyped tag with 4).
/// This does not choose a key.
pub fn select_size(values: &ResourceValues) -> u64 {
    let _requested = (values.file_size, values.total_file_allocated_size);
    0
}

/// One directory, `FUN_00008134`. Caller `FUN_00007b54` is the async frame.
///
/// On entry, if the task frame `+0x68` is set, the caller bails.
/// Otherwise it calls this, sums child sizes, then switches to the progress update.
pub fn scan_directory(
    task: &ScanTask,
    directory: &Path,
    list: impl Fn(&Path) -> Vec<(PathBuf, ResourceValues, u32)>,
) -> Vec<Entity> {
    let mut children = Vec::new();
    for (path, values, st_flags) in list(directory) {
        if ignored(&task.filter, &path) {
            continue;
        }
        let status = cloud_status(task.strategy, &values, st_flags);
        if values.is_directory {
            let nested = scan_directory(task, &path, &list);
            let size = sum_child_sizes(&nested);
            children.push(Entity {
                path,
                size,
                is_directory: true,
                is_purgeable: false,
                cloud_status: status,
                children: nested,
            });
        } else {
            // Leaf store in `FUN_00008134`: is_directory and is_purgeable written as 0,
            // children set to an empty array, one status byte.
            children.push(Entity {
                path,
                size: select_size(&values),
                is_directory: false,
                is_purgeable: false,
                cloud_status: status,
                children: Vec::new(),
            });
        }
    }
    children
}

/// `FUN_00007b54` after the directory call. Stride 0x28, size at element +0x10.
/// Overflow traps. Then the frame loads `progressTracker` and task-switches.
pub fn sum_child_sizes(children: &[Entity]) -> u64 {
    let mut total: u64 = 0;
    for child in children {
        total = total.checked_add(child.size).expect("size add overflow");
    }
    total
}

pub fn progress_after_directory(children: &[Entity], current_path: Option<String>) -> ScanProgress {
    ScanProgress {
        scanned_size: sum_child_sizes(children),
        current_path,
    }
}

/// `leavesCount` at `0x3fe0`.
/// Purgeable node contributes 0 and is not opened.
/// Non-purgeable file contributes 1.
/// Non-purgeable directory sums its children. Checked add.
pub fn leaves_count(node: &Entity) -> i64 {
    if node.is_purgeable {
        return 0;
    }
    if !node.is_directory {
        return 1;
    }
    let mut total: i64 = 0;
    for child in &node.children {
        let add = if child.is_purgeable {
            0
        } else if child.is_directory {
            leaves_count(child)
        } else {
            1
        };
        total = total.checked_add(add).expect("leaves add overflow");
    }
    total
}

/// `removableLeavesCount` at `0x4460`.
/// Same shape as `leaves_count`, except a file counts only when `cloud_status != 1`.
/// `1` is the evicted / not-downloaded / dataless return. Name link is inferred.
pub fn removable_leaves_count(node: &Entity) -> i64 {
    if node.is_purgeable {
        return 0;
    }
    if !node.is_directory {
        return i64::from(node.cloud_status != CloudStatus::CloudEvicted);
    }
    let mut total: i64 = 0;
    for child in &node.children {
        total = total
            .checked_add(removable_leaves_count(child))
            .expect("removable add overflow");
    }
    total
}

/// Signature of `units(for:shouldSpare:)` at `0x5c38`.
/// The real flatten is `FUN_00005cb8` (2164 bytes) and was not decompiled.
/// This walk is only the public shape: spare predicate in, flat `Unit` rows out.
pub fn units(root: &Entity, should_spare: impl Fn(&Entity) -> bool) -> Vec<Unit> {
    let mut out = Vec::new();
    fn walk(node: &Entity, should_spare: &impl Fn(&Entity) -> bool, out: &mut Vec<Unit>) {
        if should_spare(node) {
            return;
        }
        if node.children.is_empty() {
            out.push(Unit {
                path: node.path.clone(),
                size: node.size,
                removable_leaves_count: removable_leaves_count(node),
            });
            return;
        }
        for child in &node.children {
            walk(child, should_spare, out);
        }
    }
    walk(root, &should_spare, &mut out);
    out
}
