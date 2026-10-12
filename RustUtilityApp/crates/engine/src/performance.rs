use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MachineSample {
    pub used_memory: u64,
    pub total_memory: u64,
    pub cpu_percent: f32,
}

/// One memory read and a short CPU sample. This loop is new. The original binary did not expose one.
pub fn machine_sample() -> MachineSample {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );
    std::thread::sleep(Duration::from_millis(200));
    system.refresh_cpu_usage();
    let cpu_percent = system.global_cpu_usage();
    MachineSample {
        used_memory: system.used_memory(),
        total_memory: system.total_memory(),
        cpu_percent,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchAgent {
    pub path: PathBuf,
    pub label: String,
}

/// User and local LaunchAgents directories. Reads `Label` from each plist.
pub fn launch_agents(home: &Path) -> Vec<LaunchAgent> {
    let dirs = [
        home.join("Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchAgents"),
    ];
    let mut agents = Vec::new();
    for dir in dirs {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("plist") {
                continue;
            }
            let label = plist_label(&path).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("agent")
                    .to_string()
            });
            agents.push(LaunchAgent { path, label });
            if agents.len() >= 80 {
                return agents;
            }
        }
    }
    agents
}

fn plist_label(path: &Path) -> Option<String> {
    let output = Command::new("plutil")
        .args(["-extract", "Label", "raw", "-o", "-"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Documented user DNS cache flush. Not a copied privileged script.
pub fn flush_dns_cache() -> Result<String, String> {
    let output = Command::new("dscacheutil")
        .arg("-flushcache")
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok("DNS cache flush finished.".into())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(if err.trim().is_empty() {
            "DNS cache flush failed.".into()
        } else {
            err.trim().to_string()
        })
    }
}

/// Read-only Spotlight status for the startup volume.
pub fn spotlight_status() -> Result<String, String> {
    run_mdutil(["-s", "/"])
}

/// Asks `mdutil` to erase and rebuild the index. Often needs an administrator.
pub fn reindex_spotlight() -> Result<String, String> {
    run_mdutil(["-E", "/"])
}

fn run_mdutil(args: [&str; 2]) -> Result<String, String> {
    let output = Command::new("mdutil")
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;
    let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !err.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&err);
    }
    if output.status.success() {
        Ok(if text.is_empty() {
            "Spotlight command finished.".into()
        } else {
            text
        })
    } else {
        Err(if text.is_empty() {
            "Spotlight command failed.".into()
        } else {
            text
        })
    }
}
