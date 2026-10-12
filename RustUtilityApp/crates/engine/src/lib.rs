//! Chunked file walk and per-file checks.
//!
//! The pipeline follows the shape of a scan task: enumerate, chunk, check, report.
//! Detection rules are injected. Nothing in this crate deletes files.

mod control;
mod detect;
mod enumerate;
mod error;
mod event;
mod remove;
mod scan;

pub use control::ExecutionControl;
pub use detect::{Detector, ExtensionWatchDetector, MetadataDetector};
pub use enumerate::{walk, Chunk, WalkOptions};
pub use error::ScanError;
pub use event::{Detection, ScanEvent};
pub use remove::{RemovalPlan, RemoveTask};
pub use scan::{run_scan, ScanConfig};

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn cancel_unblocks_checkpoint() {
        let control = ExecutionControl::new();
        control.pause();
        control.cancel();
        let err = control.checkpoint().await.unwrap_err();
        assert!(matches!(err, ScanError::Canceled));
    }

    #[tokio::test]
    async fn walk_chunks_files_and_marks_finale() {
        let dir = std::env::temp_dir().join(format!("rust_utility_walk_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        for name in ["a.txt", "b.txt", "sub/c.txt", "sub/d.txt", "sub/e.dat"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let control = ExecutionControl::new();
        let (tx, mut rx) = mpsc::channel(8);
        walk(WalkOptions::new(&dir, 2), &control, tx).await.unwrap();
        let mut chunks = Vec::new();
        while let Some(chunk) = rx.recv().await {
            chunks.push(chunk);
        }
        let files: usize = chunks.iter().map(|c| c.paths.len()).sum();
        assert_eq!(files, 5);
        assert!(chunks.last().unwrap().finale);
        assert!(chunks.iter().rev().skip(1).all(|c| !c.finale));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn scan_reports_extension_hits_and_removal_is_a_plan() {
        let dir = std::env::temp_dir().join(format!("rust_utility_scan_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ok.txt"), b"ok").unwrap();
        std::fs::write(dir.join("note.dmg"), b"no").unwrap();

        let control = Arc::new(ExecutionControl::new());
        let detector = Arc::new(ExtensionWatchDetector::new(["dmg"]));
        let (tx, mut rx) = mpsc::channel(32);
        let config = ScanConfig {
            chunk_size: 1,
            check_concurrency: 2,
            ..ScanConfig::new(&dir)
        };
        run_scan(config, detector, control, tx).await;

        let mut hits = Vec::new();
        let mut finished = None;
        while let Some(event) = rx.recv().await {
            match event {
                ScanEvent::Detection(hit) => hits.push(hit),
                ScanEvent::Finished {
                    scanned,
                    detections,
                    errors,
                } => finished = Some((scanned, detections, errors)),
                _ => {}
            }
        }
        assert_eq!(hits.len(), 1);
        assert!(hits[0].path.ends_with("note.dmg"));
        assert_eq!(finished, Some((2, 1, 0)));

        let plan = RemoveTask::plan(&hits);
        assert!(!plan.executed);
        assert!(dir.join("note.dmg").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
