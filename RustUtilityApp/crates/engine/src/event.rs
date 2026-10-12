use std::path::PathBuf;

/// One observation from a scan. The walk itself never deletes files.
#[derive(Clone, Debug)]
pub enum ScanEvent {
    Progress { path: PathBuf },
    Checked {
        path: PathBuf,
        size: u64,
        extension: String,
    },
    Detection(Detection),
    Error { path: PathBuf, message: String },
    Finished {
        scanned: u64,
        detections: u64,
        errors: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detection {
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
    pub note: String,
}
