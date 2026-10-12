use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessStatus {
    Granted,
    Denied,
    Unknown,
}

impl AccessStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::Unknown => "unknown",
        }
    }
}

/// Probes a protected directory. A permission error is denied. Missing is unknown.
pub fn folder_access(path: &Path) -> AccessStatus {
    match std::fs::read_dir(path) {
        Ok(_) => AccessStatus::Granted,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => AccessStatus::Denied,
        Err(err) if err.raw_os_error() == Some(1) => AccessStatus::Denied,
        Err(_) => AccessStatus::Unknown,
    }
}
