/// A glanceable menu-bar reading. Sampled once, then reused.
#[derive(Clone, Debug)]
pub struct Glance {
    pub storage: String,
    pub memory: String,
    pub battery: String,
}

impl Glance {
    pub fn sample() -> Self {
        Self {
            storage: storage_line().unwrap_or_else(|| "Storage  —".into()),
            memory: memory_line().unwrap_or_else(|| "Memory  —".into()),
            battery: battery_line().unwrap_or_else(|| "Battery  —".into()),
        }
    }
}

fn storage_line() -> Option<String> {
    let out = std::process::Command::new("df")
        .args(["-k", "/"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().nth(1)?;
    let mut parts = line.split_whitespace();
    let _fs = parts.next()?;
    let _blocks = parts.next()?;
    let _used = parts.next()?;
    let avail: u64 = parts.next()?.parse().ok()?;
    Some(format!("{} free", crate::theme::format_bytes(avail * 1024)))
}

fn memory_line() -> Option<String> {
    let out = std::process::Command::new("memory_pressure")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let free = text
        .lines()
        .find(|l| l.contains("free percentage"))
        .and_then(|l| l.split(':').nth(1))
        .map(str::trim)?;
    Some(format!("Memory {free} free"))
}

fn battery_line() -> Option<String> {
    let out = std::process::Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let pct = text.split('%').next()?.split_whitespace().last()?;
    if pct.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("Battery {pct}%"))
    } else {
        None
    }
}
