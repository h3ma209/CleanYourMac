use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use eframe::egui::{
    self, Align, Align2, Color32, Id, Layout, RichText, Rect, Sense, Shape,
    Stroke, TextStyle, Vec2, pos2,
};
use engine::{
    flush_dns_cache, folder_access, junk_roots, large_downloads, launch_agents, local_cloud_folders,
    machine_sample, move_download_to_trash, move_root_to_trash, reindex_spotlight, run_care, run_scan,
    home_folder_access, scan_space_volume, space_folder, space_report_at, spotlight_status, survey_leftovers, trash_selected, prepare_and_scan, remember_ignore, trash_threat, computer_root, is_spared, volume_contains, AccessStatus, CareReport, CloudFolder, CloudStatus, Detection, ThreatReport,
    SPACE_LENS_BUBBLE_THRESHOLD, SpaceNode,
    ExecutionControl, ExtensionWatchDetector, InstalledApp, JunkRoot, LargeFile, LaunchAgent, LeftoverHit,
    MachineSample, ScanConfig, ScanEvent, SpaceEntity, SpaceReport, SpaceVolumeScan,
};
use tokio::sync::mpsc;

mod components;
mod icons;
mod theme;
mod tokens;

use tokens::*;

use components::{
    accent_for_screen, compact_page_header, paint_app_background, tab_bar, text_field, BtnKind,
    SafetyKind, ACTION_COL_W,
};
use components::{
    button as cmp_button, button_sized, chip as cmp_chip, label_small as cmp_label_small, list_actions, safety_badge,
    scroll_list,
};
use icons::{heading_font, icon_with_label, paint_brand_mark, paint_screen_icon, paint_ui_icon, UiIcon};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Screen {
    Care,
    Cleanup,
    Apps,
    Performance,
    Protection,
    Malware,
    Clutter,
    Space,
    Cloud,
    Assistant,
}

impl Screen {
    fn label(self) -> &'static str {
        match self {
            Self::Care => "Overview",
            Self::Cleanup => "Free Up Space",
            Self::Apps => "Apps",
            Self::Performance => "Tune-Up",
            Self::Protection => "File Finder",
            Self::Malware => "Threat Scan",
            Self::Clutter => "Big Downloads",
            Self::Space => "Space Lens",
            Self::Cloud => "Cloud Storage",
            Self::Assistant => "System Monitor",
        }
    }

    fn tagline(self) -> &'static str {
        match self {
            Self::Care => "Health check of your Mac",
            Self::Cleanup => "Caches and logs",
            Self::Apps => "Installed apps and leftovers",
            Self::Performance => "Startup items and fixes",
            Self::Protection => "Find installers and archives by type",
            Self::Malware => "Compare files to known malware hashes",
            Self::Clutter => "Large files sitting in Downloads",
            Self::Space => "Visual map of what uses your disk",
            Self::Cloud => "Local iCloud and cloud folder sizes",
            Self::Assistant => "Live CPU and memory readings",
        }
    }

    /// Screens that fill themselves the first time you open them.
    /// Clutter, Apps, and Performance wait for the dock Scan button.
    fn auto_loads(self) -> bool {
        matches!(
            self,
            Self::Care | Self::Cleanup | Self::Cloud | Self::Assistant
        )
    }

    fn nav_hint(self) -> &'static str {
        self.tagline()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct UiLayout {
    sidebar_w: f32,
    compact: bool,
    narrow: bool,
    pad_x: f32,
    pad_y: f32,
}

impl UiLayout {
    fn from_viewport(w: f32) -> Self {
        let compact = w < BREAK_COMPACT;
        let narrow = w < BREAK_NARROW;
        let sidebar_w = RAIL_W;
        let pad_x = S24;
        let pad_y = if compact { S16 } else { S24 };
        Self {
            sidebar_w,
            compact,
            narrow,
            pad_x,
            pad_y,
        }
    }

    fn grid_cols(self, prefer: usize) -> usize {
        if self.compact {
            1
        } else if self.narrow {
            prefer.min(2)
        } else {
            prefer
        }
    }

    fn title_size(self) -> f32 {
        let _ = (self.compact, self.narrow);
        H2
    }

    fn subtitle_size(self) -> f32 {
        if self.compact { SMALL } else { BODY }
    }
}

struct Notice {
    ok: bool,
    text: String,
    refresh: Option<Screen>,
}

enum ConfirmKind {
    TrashRoot(PathBuf),
    TrashDownload(PathBuf),
    TrashSpace(Vec<PathBuf>),
    TrashThreat(PathBuf),
    Reindex,
}

struct Confirm {
    heading: String,
    body: String,
    action: &'static str,
    kind: ConfirmKind,
}

enum Tool {
    Dns,
    SpotlightStatus,
    Reindex,
}

enum UiMsg {
    Scan(ScanEvent),
    Care(CareReport),
    Junk(Vec<JunkRoot>),
    Note(Notice),
    Agents(Vec<LaunchAgent>),
    Apps(Vec<InstalledApp>),
    Leftovers(Vec<LeftoverHit>),
    Sample(MachineSample),
    Space { id: u64, volume: SpaceVolumeScan, report: SpaceReport },
    Threat(Result<ThreatReport, String>),
    Clutter(Vec<LargeFile>),
    Cloud(Vec<CloudFolder>),
}

fn main() -> eframe::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_W, WINDOW_H])
            .with_min_inner_size([MIN_W, MIN_H]),
        ..Default::default()
    };
    eframe::run_native(
        "Sable",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(ScanApp::new(runtime)))
        }),
    )
}

fn apply_theme(ctx: &egui::Context) {
    theme::install_fonts(ctx);
    let mut style = (*ctx.style()).clone();
    theme::polish_style(&mut style);
    ctx.set_style(style);
}

struct ScanApp {
    runtime: tokio::runtime::Runtime,
    screen: Screen,
    home: PathBuf,
    root: String,
    watch: String,
    status: &'static str,
    notice: Option<Notice>,
    confirm: Option<Confirm>,
    loaded: HashSet<Screen>,
    last_path: String,
    scanned: u64,
    detection_count: u64,
    errors: u64,
    scan_ran: bool,
    detections: Vec<Detection>,
    care: Option<CareReport>,
    junk: Vec<JunkRoot>,
    agents: Vec<LaunchAgent>,
    apps: Vec<InstalledApp>,
    app_filter: String,
    leftovers: Vec<LeftoverHit>,
    leftovers_done: bool,
    selected_app: Option<usize>,
    sample: Option<MachineSample>,
    space_root: String,
    space: Option<SpaceReport>,
    /// Full volume tree from the last Scan. Folder clicks read this, like CMM `rootNode`.
    space_volume: Option<SpaceVolumeScan>,
    space_selected: HashSet<PathBuf>,
    space_selected_bytes: HashMap<PathBuf, u64>,
    space_hover: Option<PathBuf>,
    threat_root: String,
    threat: Option<ThreatReport>,
    clutter: Vec<LargeFile>,
    cloud: Vec<CloudFolder>,
    access: AccessStatus,
    events: Option<mpsc::Receiver<UiMsg>>,
    event_tx: Option<mpsc::Sender<UiMsg>>,
    control: Option<Arc<ExecutionControl>>,
    busy: bool,
    /// Ignores stale `UiMsg::Space` results after back, forward, or a newer scan.
    space_scan_id: u64,
    layout: UiLayout,
    /// 0 = home account, 1 = Macintosh HD. Module scans still read the home folder.
    landing_volume: u8,
    care_picks: HashSet<Screen>,
    /// Checked home tasks still waiting to run. One job at a time.
    care_followups: Vec<Screen>,
    last_refresh: Option<Instant>,
    space_history: Vec<String>,
    space_future: Vec<String>,
    apps_tab: usize,
    perf_tab: usize,
    clutter_tab: usize,
    malware_show_skipped: bool,
}

impl ScanApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        let threat_root = home.join("Downloads").display().to_string();
        let space_root = home.display().to_string();
        let access = folder_access(Path::new("/Library/Application Support"));
        Self {
            runtime,
            screen: Screen::Care,
            root: home.join("Downloads").display().to_string(),
            home,
            watch: "dmg,pkg".into(),
            status: "idle",
            notice: None,
            confirm: None,
            loaded: HashSet::new(),
            last_path: String::new(),
            scanned: 0,
            detection_count: 0,
            errors: 0,
            scan_ran: false,
            detections: Vec::new(),
            care: None,
            junk: Vec::new(),
            agents: Vec::new(),
            apps: Vec::new(),
            app_filter: String::new(),
            leftovers: Vec::new(),
            leftovers_done: false,
            selected_app: None,
            sample: None,
            space_root,
            space: None,
            space_volume: None,
            space_selected: HashSet::new(),
            space_selected_bytes: HashMap::new(),
            space_hover: None,
            threat_root,
            threat: None,
            clutter: Vec::new(),
            cloud: Vec::new(),
            access,
            events: None,
            event_tx: None,
            control: None,
            busy: false,
            space_scan_id: 0,
            layout: UiLayout::from_viewport(WINDOW_W),
            landing_volume: 0,
            care_picks: HashSet::from([
                Screen::Cleanup,
                Screen::Malware,
                Screen::Performance,
                Screen::Clutter,
            ]),
            care_followups: Vec::new(),
            last_refresh: None,
            space_history: Vec::new(),
            space_future: Vec::new(),
            apps_tab: 0,
            perf_tab: 0,
            clutter_tab: 0,
            malware_show_skipped: false,
        }
    }

    fn space_open(&mut self, path: PathBuf) {
        let next = path.display().to_string();
        if next == self.space_root.trim() {
            return;
        }
        let Some(volume) = self.space_volume.as_ref() else {
            self.notice = Some(Notice {
                ok: false,
                text: "Press Scan once. Folders open from that map.".into(),
                refresh: None,
            });
            return;
        };
        if !volume_contains(volume, &path) {
            self.notice = Some(Notice {
                ok: false,
                text: "That folder is not in the current map. Press Scan to rebuild.".into(),
                refresh: None,
            });
            return;
        }
        self.space_future.clear();
        self.space_history.push(self.space_root.trim().to_string());
        self.space_root = next;
        self.space_selected.clear();
        self.space_selected_bytes.clear();
        self.space_hover = None;
    }

    fn space_back(&mut self) {
        let Some(prev) = self.space_history.pop() else {
            return;
        };
        self.space_future.push(self.space_root.trim().to_string());
        self.space_root = prev;
        self.space_selected.clear();
        self.space_selected_bytes.clear();
        self.space_hover = None;
    }

    fn space_forward(&mut self) {
        let Some(next) = self.space_future.pop() else {
            return;
        };
        self.space_history.push(self.space_root.trim().to_string());
        self.space_root = next;
        self.space_selected.clear();
        self.space_selected_bytes.clear();
        self.space_hover = None;
    }

    fn dock_run(&mut self) {
        if self.busy {
            return;
        }
        match self.screen {
            Screen::Care => self.run_checked_care(),
            Screen::Clutter => {
                self.care_followups.clear();
                self.loaded.insert(Screen::Clutter);
                self.clutter.clear();
                self.start_clutter();
            }
            Screen::Apps => {
                self.care_followups.clear();
                self.load(Screen::Apps);
            }
            Screen::Performance => {
                self.care_followups.clear();
                self.load(Screen::Performance);
            }
            Screen::Space => {
                self.care_followups.clear();
                self.start_space(false);
            }
            _ => {}
        }
    }

    /// Run only the home tasks whose checkboxes are on.
    /// Apps is informational and is never queued.
    /// A clear threat result is skipped.
    fn run_checked_care(&mut self) {
        if self.busy {
            return;
        }
        let mut jobs = Vec::new();
        if self.care_picks.contains(&Screen::Cleanup) {
            jobs.push(Screen::Cleanup);
        }
        if self.care_picks.contains(&Screen::Performance) {
            jobs.push(Screen::Performance);
        }
        if self.care_picks.contains(&Screen::Clutter) {
            jobs.push(Screen::Clutter);
        }
        let threats_clear = self.threat.as_ref().is_some_and(|report| report.hits.is_empty());
        if self.care_picks.contains(&Screen::Malware) && !threats_clear {
            jobs.push(Screen::Malware);
        }
        if jobs.is_empty() {
            self.notice = Some(Notice {
                ok: false,
                text: "Select at least one task.".into(),
                refresh: None,
            });
            return;
        }
        self.care_followups = jobs;
        self.pump_care_queue();
    }

    fn pump_care_queue(&mut self) {
        while !self.care_followups.is_empty() {
            let next = self.care_followups.remove(0);
            self.start_care_job(next);
            if self.busy {
                return;
            }
        }
    }

    fn start_care_job(&mut self, screen: Screen) {
        match screen {
            Screen::Cleanup => self.start_junk(),
            Screen::Performance => self.load_agents(),
            Screen::Apps => self.load_apps(),
            Screen::Clutter => {
                self.loaded.insert(Screen::Clutter);
                self.clutter.clear();
                self.start_clutter();
            }
            Screen::Malware => self.start_threat(),
            _ => {}
        }
    }

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.notice = None;
    }

    fn event_sender(&mut self) -> mpsc::Sender<UiMsg> {
        if self.event_tx.is_none() {
            let (tx, rx) = mpsc::channel(256);
            self.event_tx = Some(tx.clone());
            self.events = Some(rx);
        }
        self.event_tx.as_ref().expect("event channel").clone()
    }

    fn arm(&mut self) -> mpsc::Sender<UiMsg> {
        let tx = self.event_sender();
        self.busy = true;
        tx
    }

    /// Start the loader that fills a screen.
    fn load(&mut self, screen: Screen) {
        self.loaded.insert(screen);
        match screen {
            Screen::Care => self.start_care(),
            Screen::Cleanup => self.start_junk(),
            Screen::Apps => self.load_apps(),
            Screen::Performance => self.load_agents(),
            Screen::Clutter => self.start_clutter(),
            Screen::Cloud => self.start_cloud(),
            Screen::Assistant => self.sample_machine(),
            Screen::Protection | Screen::Malware | Screen::Space => {}
        }
    }

    fn start_scan(&mut self) {
        if self.busy {
            return;
        }
        self.detections.clear();
        self.scanned = 0;
        self.detection_count = 0;
        self.errors = 0;
        self.scan_ran = true;
        self.last_path.clear();
        let control = Arc::new(ExecutionControl::new());
        let detector = Arc::new(ExtensionWatchDetector::new(
            self.watch
                .split(',')
                .map(|s| s.trim().trim_start_matches('.'))
                .filter(|s| !s.is_empty()),
        ));
        let config = ScanConfig::new(self.root.trim());
        let tx = self.arm();
        let control_task = Arc::clone(&control);
        self.runtime.spawn(async move {
            let (inner_tx, mut inner_rx) = mpsc::channel(256);
            tokio::spawn(async move {
                run_scan(config, detector, control_task, inner_tx).await;
            });
            while let Some(event) = inner_rx.recv().await {
                if tx.send(UiMsg::Scan(event)).await.is_err() {
                    break;
                }
            }
        });
        self.control = Some(control);
        self.status = "scanning";
    }

    fn start_care(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let report = run_care(home).await;
            let _ = tx.send(UiMsg::Care(report)).await;
        });
    }

    fn start_junk(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let roots = tokio::task::spawn_blocking(move || junk_roots(&home))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Junk(roots)).await;
        });
    }

    fn trash(&mut self, path: PathBuf) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "cleaning";
        self.runtime.spawn(async move {
            let notice = match move_root_to_trash(&home, &path) {
                Ok(()) => Notice {
                    ok: true,
                    text: format!("Moved \"{}\" to the Trash. Open the Trash to put it back.", file_name(&path)),
                    refresh: Some(Screen::Cleanup),
                },
                Err(err) => Notice { ok: false, text: err, refresh: None },
            };
            let _ = tx.send(UiMsg::Note(notice)).await;
        });
    }

    fn load_agents(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let agents = tokio::task::spawn_blocking(move || launch_agents(&home))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Agents(agents)).await;
        });
    }

    fn run_tool(&mut self, kind: Tool) {
        if self.busy {
            return;
        }
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let notice = match kind {
                Tool::Dns => outcome(flush_dns_cache(), "DNS cache cleared."),
                Tool::SpotlightStatus => outcome(spotlight_status(), "Spotlight status unavailable."),
                Tool::Reindex => outcome(reindex_spotlight(), "Spotlight is rebuilding its index."),
            };
            let _ = tx.send(UiMsg::Note(notice)).await;
        });
    }

    fn start_clutter(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let files = tokio::task::spawn_blocking(move || large_downloads(&home, 50 * 1024 * 1024))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Clutter(files)).await;
        });
    }

    fn trash_download(&mut self, path: PathBuf) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "cleaning";
        self.runtime.spawn(async move {
            let notice = match move_download_to_trash(&home, &path) {
                Ok(()) => Notice {
                    ok: true,
                    text: format!("Moved \"{}\" to the Trash. Open the Trash to put it back.", file_name(&path)),
                    refresh: Some(Screen::Clutter),
                },
                Err(err) => Notice { ok: false, text: err, refresh: None },
            };
            let _ = tx.send(UiMsg::Note(notice)).await;
        });
    }

    fn start_cloud(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let folders = tokio::task::spawn_blocking(move || local_cloud_folders(&home))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Cloud(folders)).await;
        });
    }

    fn sample_machine(&mut self) {
        if self.busy {
            return;
        }
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let sample = tokio::task::spawn_blocking(machine_sample).await.unwrap_or(MachineSample {
                used_memory: 0,
                total_memory: 0,
                cpu_percent: 0.0,
            });
            let _ = tx.send(UiMsg::Sample(sample)).await;
        });
    }

    fn survey_selected(&mut self) {
        if self.busy {
            return;
        }
        let Some(index) = self.selected_app else {
            return;
        };
        let Some(app) = self.apps.get(index) else {
            return;
        };
        let home = self.home.clone();
        let bundle_id = app.bundle_id.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let hits = tokio::task::spawn_blocking(move || survey_leftovers(&home, &bundle_id))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Leftovers(hits)).await;
        });
    }

    fn load_apps(&mut self) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let apps = tokio::task::spawn_blocking(move || engine::installed_apps(&home))
                .await
                .unwrap_or_default();
            let _ = tx.send(UiMsg::Apps(apps)).await;
        });
    }

    fn trash_space(&mut self, paths: Vec<PathBuf>) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let root = PathBuf::from(self.space_root.trim());
        let tx = self.arm();
        self.status = "cleaning";
        self.runtime.spawn(async move {
            let notice = match trash_selected(&root, &home, &paths) {
                Ok(count) => Notice {
                    ok: true,
                    text: format!("Moved {count} files to the Trash. Open the Trash to put them back."),
                    refresh: Some(Screen::Space),
                },
                Err(err) => Notice { ok: false, text: err, refresh: None },
            };
            let _ = tx.send(UiMsg::Note(notice)).await;
        });
    }

    fn start_threat(&mut self) {
        if self.busy {
            return;
        }
        let root = PathBuf::from(self.threat_root.trim());
        if !root.is_dir() {
            self.notice = Some(Notice {
                ok: false,
                text: "Choose a folder to scan.".into(),
                refresh: None,
            });
            return;
        }
        self.threat = None;
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let report = tokio::task::spawn_blocking(move || prepare_and_scan(&home, &root)).await;
            let message = match report {
                Ok(result) => UiMsg::Threat(result),
                Err(_) => UiMsg::Threat(Err("the malware scan stopped before it finished".into())),
            };
            let _ = tx.send(message).await;
        });
    }

    fn trash_threat(&mut self, path: PathBuf) {
        if self.busy {
            return;
        }
        let home = self.home.clone();
        let tx = self.arm();
        self.status = "cleaning";
        self.runtime.spawn(async move {
            let notice = match trash_threat(&home, &path) {
                Ok(()) => Notice {
                    ok: true,
                    text: format!("Moved {} to the Trash. Open the Trash to put it back.", path.display()),
                    refresh: Some(Screen::Malware),
                },
                Err(err) => Notice { ok: false, text: err, refresh: None },
            };
            let _ = tx.send(UiMsg::Note(notice)).await;
        });
    }

    fn ignore_threat(&mut self, path: PathBuf) {
        if let Err(err) = remember_ignore(&self.home, &path) {
            self.notice = Some(Notice { ok: false, text: err, refresh: None });
            return;
        }
        if let Some(report) = &mut self.threat {
            report.hits.retain(|hit| hit.path != path);
        }
        self.notice = Some(Notice {
            ok: true,
            text: "That file will be skipped on later scans.".into(),
            refresh: None,
        });
    }

    fn start_space(&mut self, force_rescan: bool) {
        let trimmed = self.space_root.trim();
        if trimmed.is_empty() {
            self.notice = Some(Notice {
                ok: false,
                text: "Choose a location first — try “This Mac” or Home.".into(),
                refresh: None,
            });
            return;
        }
        let folder = PathBuf::from(trimmed);
        if !force_rescan {
            if let Some(volume) = self.space_volume.as_ref() {
                if let Some(report) = space_report_at(volume, &folder) {
                    self.space_root = report.root.display().to_string();
                    self.space = Some(report);
                    self.space_selected.clear();
                    self.space_selected_bytes.clear();
                    self.space_hover = None;
                    return;
                }
                self.notice = Some(Notice {
                    ok: false,
                    text: "That folder is not in the current map. Press Scan to rebuild from the selected root.".into(),
                    refresh: None,
                });
                return;
            }
        }
        if !folder.exists() {
            self.notice = Some(Notice {
                ok: false,
                text: format!("That folder does not exist: {}", folder.display()),
                refresh: None,
            });
            return;
        }
        self.space_root = normalize_space_path(trimmed);
        self.access = folder_access(Path::new("/Library/Application Support"));
        if let Some(message) = self.space_scan_blocked_message() {
            self.notice = Some(Notice {
                ok: false,
                text: message,
                refresh: None,
            });
            return;
        }

        if self.busy && !force_rescan {
            return;
        }

        let view_folder = folder.clone();
        let volume_scan_root = self
            .space_volume
            .as_ref()
            .map(|volume| volume.scan_root.clone())
            .unwrap_or_else(|| view_folder.clone());
        if force_rescan {
            self.space_volume = None;
        }
        self.space = None;
        self.space_selected.clear();
        self.space_selected_bytes.clear();
        self.space_hover = None;
        self.space_scan_id = self.space_scan_id.wrapping_add(1);
        let scan_id = self.space_scan_id;
        let tx = self.event_sender();
        self.busy = true;
        self.status = "scanning";
        let scan_root = if force_rescan {
            volume_scan_root
        } else {
            view_folder.clone()
        };
        self.runtime.spawn(async move {
            let volume = scan_space_volume(scan_root).await;
            let report = space_report_at(&volume, &view_folder).unwrap_or(SpaceReport {
                root: view_folder,
                total: 0,
                children: Vec::new(),
            });
            let _ = tx.send(UiMsg::Space { id: scan_id, volume, report }).await;
        });
    }

    fn space_scan_blocked_message(&self) -> Option<String> {
        let root = PathBuf::from(self.space_root.trim());
        let mac = computer_root();
        let needs_fda = root == mac || (root.starts_with(&mac) && !root.starts_with(&self.home));
        if needs_fda && self.access != AccessStatus::Granted {
            Some(
                "Full Disk Access is required to scan this drive. Open System Settings → Privacy & Security → Full Disk Access, enable Sable, then press Scan again.".into(),
            )
        } else {
            None
        }
    }

    fn finish(&mut self) {
        let scanned = self.status == "scanning";
        self.busy = false;
        self.status = "idle";
        self.control = None;
        if scanned {
            self.last_refresh = Some(Instant::now());
        }
        self.pump_care_queue();
    }

    fn perform(&mut self, kind: ConfirmKind) {
        match kind {
            ConfirmKind::TrashRoot(path) => self.trash(path),
            ConfirmKind::TrashDownload(path) => self.trash_download(path),
            ConfirmKind::TrashSpace(paths) => self.trash_space(paths),
            ConfirmKind::TrashThreat(path) => self.trash_threat(path),
            ConfirmKind::Reindex => self.run_tool(Tool::Reindex),
        }
    }

    fn drain(&mut self, ctx: &egui::Context) {
        let Some(mut rx) = self.events.take() else {
            return;
        };
        let mut pending = Vec::new();
        while let Ok(event) = rx.try_recv() {
            pending.push(event);
        }
        self.events = Some(rx);
        for event in pending {
            match event {
                UiMsg::Scan(ScanEvent::Progress { path }) => {
                    self.last_path = path.display().to_string();
                }
                UiMsg::Scan(ScanEvent::Checked { path, .. }) => {
                    self.scanned += 1;
                    self.last_path = path.display().to_string();
                }
                UiMsg::Scan(ScanEvent::Detection(hit)) => {
                    self.scanned += 1;
                    self.detection_count += 1;
                    self.last_path = hit.path.display().to_string();
                    self.detections.push(hit);
                }
                UiMsg::Scan(ScanEvent::Error { path, message }) => {
                    self.errors += 1;
                    self.last_path = format!("{} ({message})", path.display());
                }
                UiMsg::Scan(ScanEvent::Finished { scanned, detections, errors }) => {
                    self.scanned = scanned;
                    self.detection_count = detections;
                    self.errors = errors;
                    self.finish();
                }
                UiMsg::Care(report) => {
                    self.access = report.access;
                    self.care = Some(report);
                    self.finish();
                }
                UiMsg::Junk(roots) => {
                    self.junk = roots;
                    self.finish();
                }
                UiMsg::Note(notice) => {
                    let rescan_space = notice.refresh == Some(Screen::Space);
                    if let Some(screen) = notice.refresh {
                        self.loaded.remove(&screen);
                        self.loaded.remove(&Screen::Care);
                    }
                    self.notice = Some(notice);
                    self.finish();
                    if rescan_space && self.screen == Screen::Space {
                        self.space_volume = None;
                        self.start_space(true);
                    }
                }
                UiMsg::Agents(agents) => {
                    self.agents = agents;
                    self.finish();
                }
                UiMsg::Apps(apps) => {
                    self.apps = apps;
                    self.selected_app = None;
                    self.leftovers.clear();
                    self.leftovers_done = false;
                    self.finish();
                }
                UiMsg::Leftovers(hits) => {
                    self.leftovers = hits;
                    self.leftovers_done = true;
                    self.finish();
                }
                UiMsg::Sample(sample) => {
                    self.sample = Some(sample);
                    self.finish();
                }
                UiMsg::Threat(result) => {
                    match result {
                        Ok(report) => self.threat = Some(report),
                        Err(text) => {
                            self.notice = Some(Notice { ok: false, text, refresh: None });
                        }
                    }
                    self.finish();
                }
                UiMsg::Space { id, volume, report } => {
                    if id != self.space_scan_id {
                        continue;
                    }
                    self.space_volume = Some(volume);
                    self.space_root = normalize_space_path(&report.root.display().to_string());
                    self.space = Some(report);
                    self.space_selected.clear();
                    self.space_selected_bytes.clear();
                    self.space_hover = None;
                    self.finish();
                }
                UiMsg::Clutter(files) => {
                    self.clutter = files;
                    self.finish();
                }
                UiMsg::Cloud(folders) => {
                    self.cloud = folders;
                    self.finish();
                }
            }
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
    }

    fn confirm_modal(&mut self, ctx: &egui::Context) {
        let Some(confirm) = self.confirm.as_ref() else {
            return;
        };
        let heading = confirm.heading.clone();
        let body = confirm.body.clone();
        let action = confirm.action;
        let dialog_w = if self.layout.compact { 320.0 } else { 420.0 };

        egui::Area::new(egui::Id::new("scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                let rect = ctx.screen_rect();
                ui.painter().rect_filled(rect, 0.0, SCRIM);
                ui.allocate_rect(rect, Sense::click());
            });

        let mut choice: Option<bool> = None;
        egui::Window::new("confirm")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(theme::modal_frame())
            .show(ctx, |ui| {
                ui.set_width(dialog_w);
                icons::icon_with_label(ui, icons::UiIcon::Alert, &heading, TEXT);
                ui.add_space(S4);
                ui.add_space(S8);
                ui.label(RichText::new(body).size(BODY).line_height(Some(lh(BODY))).color(TEXT_2));
                ui.add_space(S16);
                ui.horizontal(|ui| {
                    if primary(ui, action, true).clicked() {
                        choice = Some(true);
                    }
                    if ghost(ui, "Cancel", true).clicked() {
                        choice = Some(false);
                    }
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            choice = Some(false);
        }
        match choice {
            Some(true) => {
                if let Some(confirm) = self.confirm.take() {
                    self.perform(confirm.kind);
                }
            }
            Some(false) => self.confirm = None,
            None => {}
        }
    }
}

impl eframe::App for ScanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        self.layout = UiLayout::from_viewport(ctx.screen_rect().width());

        if !self.busy {
            for screen in [self.screen, Screen::Care, Screen::Cleanup] {
                if screen.auto_loads() && !self.loaded.contains(&screen) {
                    self.load(screen);
                    break;
                }
            }
        }

        let pulse = ctx.input(|i| i.time);
        let screen = self.screen;
        let busy = self.busy;
        let status = self.status;
        let layout = self.layout;
        let accent = accent_for_screen(screen);
        let granted = self.access == AccessStatus::Granted;

        egui::SidePanel::left("nav")
            .exact_width(layout.sidebar_w)
            .resizable(false)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rail = ui.max_rect();
                paint_app_background(ui.painter(), rail);
                ui.painter().rect_filled(rail, 0.0, Color32::from_black_alpha(32));
                ui.add_space(S16);
                rail_brand(ui);
                ui.add_space(S8);
                let status_h = S32 + S32 + S8;
                let nav_h = (rail.bottom() - ui.cursor().top() - status_h - S8).max(S32);
                let before = ui.cursor().top();
                egui::ScrollArea::vertical()
                    .id_salt("rail_nav")
                    .max_height(nav_h)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (group, items) in rail_groups() {
                            rail_group_label(ui, group);
                            for item in items {
                                if rail_nav_item(ui, *item, *item == screen, accent_for_screen(*item)).clicked() {
                                    self.go(*item);
                                }
                            }
                            ui.add_space(S8);
                        }
                    });
                let consumed = ui.cursor().top() - before;
                ui.add_space((nav_h - consumed).max(0.0));
                rail_status(ui, granted, status, busy, pulse);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let panel = ui.max_rect();
                paint_app_background(ui.painter(), panel);
                let content = Rect::from_min_max(
                    panel.min + v2(layout.pad_x, layout.pad_y),
                    panel.max - v2(layout.pad_x, layout.pad_y),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content), |ui| {
                    if screen == Screen::Space {
                        space_screen(ui, self);
                    } else if screen == Screen::Care {
                        care_screen(ui, self);
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt(format!("main_{}", screen.label()))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                match self.screen {
                                    Screen::Care => {}
                                    Screen::Cleanup => cleanup_screen(ui, self),
                                    Screen::Apps => apps_screen(ui, self),
                                    Screen::Performance => performance_screen(ui, self),
                                    Screen::Protection => protection_screen(ui, self),
                                    Screen::Malware => malware_screen(ui, self),
                                    Screen::Clutter => clutter_screen(ui, self),
                                    Screen::Cloud => cloud_screen(ui, self),
                                    Screen::Assistant => assistant_screen(ui, self),
                                    Screen::Space => {}
                                }
                            });
                    }
                });
            });

        self.confirm_modal(ctx);
    }
}

// ───────────────────────────── sidebar chrome ─────────────────────────────

struct Tone {
    top: Color32,
    bottom: Color32,
    accent: Color32,
}

fn tone_for(screen: Screen) -> Tone {
    match screen {
        Screen::Clutter => Tone {
            top: GRAD_TEAL_TOP,
            bottom: GRAD_TEAL_BOTTOM,
            accent: GRAD_TEAL_ACCENT,
        },
        Screen::Apps => Tone {
            top: GRAD_BLUE_TOP,
            bottom: GRAD_BLUE_BOTTOM,
            accent: GRAD_BLUE_ACCENT,
        },
        Screen::Performance => Tone {
            top: GRAD_ORANGE_TOP,
            bottom: GRAD_ORANGE_BOTTOM,
            accent: GRAD_ORANGE_ACCENT,
        },
        Screen::Care | Screen::Space => Tone {
            top: GRAD_PURPLE_TOP,
            bottom: GRAD_PURPLE_BOTTOM,
            accent: GRAD_PURPLE_ACCENT,
        },
        Screen::Cleanup => Tone {
            top: GRAD_GREEN_TOP,
            bottom: GRAD_GREEN_BOTTOM,
            accent: GRAD_GREEN_ACCENT,
        },
        Screen::Protection => Tone {
            top: GRAD_SLATE_TOP,
            bottom: GRAD_SLATE_BOTTOM,
            accent: GRAD_SLATE_ACCENT,
        },
        Screen::Malware => Tone {
            top: GRAD_WINE_TOP,
            bottom: GRAD_WINE_BOTTOM,
            accent: GRAD_WINE_ACCENT,
        },
        Screen::Cloud => Tone {
            top: GRAD_SKY_TOP,
            bottom: GRAD_SKY_BOTTOM,
            accent: GRAD_SKY_ACCENT,
        },
        Screen::Assistant => Tone {
            top: GRAD_INDIGO_TOP,
            bottom: GRAD_INDIGO_BOTTOM,
            accent: GRAD_INDIGO_ACCENT,
        },
    }
}

fn shows_dock(screen: Screen) -> bool {
    matches!(
        screen,
        Screen::Clutter | Screen::Apps | Screen::Performance | Screen::Space
    )
}

fn dock_label(screen: Screen) -> &'static str {
    if screen == Screen::Care { "Run" } else { "Scan" }
}

fn paint_vgrad(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
}

fn paint_dock(ctx: &egui::Context, label: &str, accent: Color32, enabled: bool) -> bool {
    let mut clicked = false;
    egui::Area::new(egui::Id::new("dock"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, [0.0, -S24])
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(CTA_H + S16), Sense::click());
            let center = rect.center();
            let radius = CTA_H / 2.0;
            let painter = ui.painter().with_clip_rect(rect);
            painter.circle_filled(center, radius + S8, with_alpha(accent, 48));
            let pressed = enabled && response.is_pointer_button_down_on();
            let fill = if !enabled {
                with_alpha(accent, 140)
            } else if pressed {
                mix(INK, accent, 28)
            } else if response.hovered() {
                mix(WHITE, accent, 16)
            } else {
                accent
            };
            painter.circle_filled(center, radius, fill);
            let ring = if response.has_focus() {
                focus_stroke()
            } else {
                Stroke::new(STROKE_W, with_alpha(WHITE, 200))
            };
            painter.circle_stroke(center, radius, ring);
            painter.text(center, Align2::CENTER_CENTER, label, font_semi(BODY), WHITE);
            if enabled && response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            clicked = enabled && response.clicked();
        });
    clicked
}

fn rail_groups() -> [(&'static str, &'static [Screen]); 4] {
    [
        ("Home", &[Screen::Care]),
        ("Clean", &[Screen::Cleanup, Screen::Clutter, Screen::Space]),
        (
            "Mac",
            &[
                Screen::Apps,
                Screen::Performance,
                Screen::Cloud,
                Screen::Assistant,
            ],
        ),
        ("Security", &[Screen::Protection, Screen::Malware]),
    ]
}

fn rail_brand(ui: &mut egui::Ui) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), S32), Sense::hover());
    let mark = Rect::from_min_size(rect.min + v2(S16, 0.0), Vec2::splat(S32));
    paint_brand_mark(ui.painter(), mark, ACCENT);
    ui.painter().text(
        mark.right_center() + v2(S8, 0.0),
        Align2::LEFT_CENTER,
        "Sable",
        font_semi(TITLE),
        WHITE,
    );
    response.on_hover_text("Sable — Mac cleaner and tune-up");
}

fn rail_group_label(ui: &mut egui::Ui, label: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), S24), Sense::hover());
    ui.painter().text(
        rect.left_center() + v2(S16, 0.0),
        Align2::LEFT_CENTER,
        label,
        font_semi(CAPTION),
        CARD_MUTED,
    );
}

fn rail_nav_item(ui: &mut egui::Ui, screen: Screen, active: bool, accent: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), BTN_MIN_H), Sense::click());
    let pill = rect.shrink2(v2(S8, S4));
    if active {
        ui.painter().rect_filled(pill, r_sm(), with_alpha(WHITE, 32));
        let bar = Rect::from_min_max(pill.left_top(), pos2(pill.left() + S4, pill.bottom()));
        ui.painter().rect_filled(bar, r_xs(), accent);
    } else if response.hovered() {
        ui.painter().rect_filled(pill, r_sm(), with_alpha(WHITE, 16));
    }
    if response.has_focus() {
        ui.painter().rect_stroke(pill, r_sm(), focus_stroke());
    }
    let icon = Rect::from_center_size(pos2(pill.left() + S16, pill.center().y), Vec2::splat(18.0));
    paint_screen_icon(
        ui.painter(),
        screen,
        icon,
        if active { WHITE } else { with_alpha(WHITE, 220) },
    );
    ui.painter().text(
        icon.right_center() + v2(S8, 0.0),
        Align2::LEFT_CENTER,
        screen.label(),
        font_regular(SMALL),
        WHITE,
    );
    let response = response.on_hover_text(format!("{}\n{}", screen.label(), screen.nav_hint()));
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn rail_status(ui: &mut egui::Ui, granted: bool, status: &str, busy: bool, pulse: f64) {
    let (color, text) = status_look(status, busy, pulse);
    rail_meta(
        ui,
        if granted { SUCCESS } else { WARN },
        if granted { "Full access" } else { "Limited access" },
        if granted { "Full disk access" } else { "Limited disk access" },
    );
    rail_meta(ui, color, text, text);
}

fn rail_meta(ui: &mut egui::Ui, color: Color32, label: &str, tip: &str) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), S32), Sense::hover());
    let center = rect.left_center() + v2(S16 + 5.0, 0.0);
    ui.painter().circle_filled(center, 5.0, color);
    ui.painter().text(
        center + v2(S12, 0.0),
        Align2::LEFT_CENTER,
        label,
        font_regular(CAPTION),
        WHITE,
    );
    response.on_hover_text(tip);
}

fn status_look(status: &str, busy: bool, time: f64) -> (Color32, &'static str) {
    let (color, text) = match status {
        "paused" => (WARN, "Paused"),
        "canceling" => (DANGER, "Stopping..."),
        "cleaning" => (WARN, "Moving to Trash..."),
        _ if busy => (WHITE, "Working..."),
        _ => (with_alpha(WHITE, 180), "Ready"),
    };
    let alpha = if busy { 0.45 + 0.55 * (time * 3.0).sin().abs() as f32 } else { 1.0 };
    (with_alpha(color, (alpha * 255.0) as u8), text)
}

// ───────────────────────────── screens ─────────────────────────────

fn care_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let area = ui.available_rect_before_wrap();
    let footer_h = S16 + CTA_H + S16;
    let body = Rect::from_min_max(area.min, pos2(area.right(), area.bottom() - footer_h));
    let footer = Rect::from_min_max(pos2(area.left(), area.bottom() - footer_h), area.max);

    let reclaim = if app.junk.is_empty() {
        app.care.as_ref().map(|report| report.cleanup_bytes)
    } else {
        Some(app.junk.iter().map(|root| root.bytes).sum())
    };
    let threats = app.threat.as_ref().map(|report| report.hits.len());
    let agents = app.care.as_ref().map(|report| report.agent_count).unwrap_or(app.agents.len());
    let apps = app.care.as_ref().map(|report| report.app_count).unwrap_or(app.apps.len());
    let leftovers = if app.leftovers_done { Some(app.leftovers.len()) } else { None };
    let downloads = if app.loaded.contains(&Screen::Clutter) {
        Some(app.clutter.len())
    } else {
        None
    };
    let threats_clear = matches!(threats, Some(0));

    let cards = [
        CareCard {
            screen: Screen::Cleanup,
            title: "Cleanup",
            metric: reclaim.map(format_bytes).unwrap_or_else(|| "—".into()),
            detail: "of junk to clean".into(),
            icon: Screen::Cleanup,
            accent: GRAD_GREEN_ACCENT,
            action: Some("Review"),
            selectable: true,
            success: false,
        },
        CareCard {
            screen: Screen::Malware,
            title: "Protection",
            metric: match threats {
                Some(0) => "No threats found".into(),
                Some(count) => format!("{count} threats"),
                None => "Not scanned".into(),
            },
            detail: match threats {
                Some(0) => "You're clear".into(),
                Some(_) => "to remove".into(),
                None => "Threat scan has not run".into(),
            },
            icon: Screen::Malware,
            accent: GRAD_WINE_ACCENT,
            action: if threats_clear { None } else { Some("Review") },
            selectable: !threats_clear,
            success: threats_clear,
        },
        CareCard {
            screen: Screen::Performance,
            title: "Performance",
            metric: format!("{agents} items"),
            detail: "to review".into(),
            icon: Screen::Performance,
            accent: GRAD_ORANGE_ACCENT,
            action: Some("Review"),
            selectable: true,
            success: false,
        },
        CareCard {
            screen: Screen::Apps,
            title: "Applications",
            metric: match leftovers {
                Some(count) if count > 0 => format!("{count} leftover files"),
                _ => format!("{apps} apps"),
            },
            detail: "installed".into(),
            icon: Screen::Apps,
            accent: GRAD_BLUE_ACCENT,
            action: Some("Browse"),
            selectable: false,
            success: false,
        },
        CareCard {
            screen: Screen::Clutter,
            title: "My Clutter",
            metric: match downloads {
                Some(count) => format!("{count} large downloads"),
                None => "Not scanned".into(),
            },
            detail: match downloads {
                Some(_) => "to remove".into(),
                None => "Large files in Downloads".into(),
            },
            icon: Screen::Clutter,
            accent: GRAD_TEAL_ACCENT,
            action: Some("Review"),
            selectable: true,
            success: false,
        },
    ];

    let selected = cards.iter().filter(|card| card.selectable && app.care_picks.contains(&card.screen)).count();
    let mut goto = None;
    let mut run = false;

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(body), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("care_body")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                banner(ui, app);
                ui.label(
                    RichText::new("Your tasks are ready to run.")
                        .font(heading_font(H2))
                        .color(WHITE),
                );
                ui.add_space(S16);
                care_summary(ui, app, reclaim, selected);
                ui.add_space(S16);

                if app.care.is_none() && app.busy && app.care_followups.is_empty() {
                    ui.vertical_centered(|ui| loading_line(ui, "Checking your Mac..."));
                    return;
                }

                let cols = app.layout.grid_cols(3);
                let rows = cards.len().div_ceil(cols);
                let gaps = (rows.saturating_sub(1) as f32) * S16;
                let raw = (ui.available_height() - gaps) / rows as f32;
                let card_h = raw.clamp(176.0, 220.0);
                let slack = (ui.available_height() - card_h * rows as f32 - gaps).max(0.0);
                ui.add_space(slack * 0.25);

                let mut index = 0;
                while index < cards.len() {
                    let span = cols.min(cards.len() - index);
                    let gap = S16;
                    let col_w = (ui.available_width() - gap * (span.saturating_sub(1) as f32)) / span as f32;
                    let previous_gap = ui.spacing().item_spacing.x;
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.horizontal(|ui| {
                        for column in 0..span {
                            let card = &cards[index + column];
                            ui.allocate_ui_with_layout(
                                Vec2::new(col_w, card_h),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    if care_task_card(ui, app, card, card_h) {
                                        goto = Some(card.screen);
                                    }
                                },
                            );
                        }
                    });
                    ui.spacing_mut().item_spacing.x = previous_gap;
                    index += span;
                    if index < cards.len() {
                        ui.add_space(S16);
                    }
                }
            });
    });

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(footer), |ui| {
        let line = Rect::from_min_max(footer.min, pos2(footer.right(), footer.top() + STROKE_W));
        ui.painter().rect_filled(line, 0.0, CARD_LINE);
        ui.vertical_centered(|ui| {
            ui.add_space(S16);
            let label = if app.busy { "Running" } else { "Run" };
            if contrast_button(ui, "care-run", label, !app.busy && selected > 0, 160.0).clicked() {
                run = true;
            }
        });
    });

    if run {
        app.run_checked_care();
    }
    if let Some(screen) = goto {
        app.go(screen);
    }
}

fn care_summary(ui: &mut egui::Ui, app: &ScanApp, reclaim: Option<u64>, selected: usize) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), CTA_H), Sense::hover());
    ui.painter().rect_filled(rect, r_md(), CARD_HOME);
    ui.painter().rect_stroke(rect, r_md(), stroke(CARD_LINE));
    let text_rect = rect.shrink2(v2(S16, 0.0));
    let size = reclaim.map(format_bytes).unwrap_or_else(|| "—".into());
    let task_label = if selected == 1 {
        "1 task selected".to_string()
    } else {
        format!("{selected} tasks selected")
    };
    let text = format!("Last scan: {}  ·  {size} reclaimable  ·  {task_label}", scan_age(app.last_refresh));
    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        Align2::LEFT_CENTER,
        text,
        font_regular(SMALL),
        WHITE,
    );
}

fn scan_age(at: Option<Instant>) -> String {
    let Some(at) = at else {
        return "not yet".into();
    };
    let secs = at.elapsed().as_secs();
    if secs < 10 {
        "just now".into()
    } else if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else {
        format!("{} hr ago", secs / 3600)
    }
}

struct CareCard {
    screen: Screen,
    title: &'static str,
    metric: String,
    detail: String,
    icon: Screen,
    accent: Color32,
    action: Option<&'static str>,
    selectable: bool,
    success: bool,
}

fn care_task_card(ui: &mut egui::Ui, app: &mut ScanApp, card: &CareCard, card_h: f32) -> bool {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, card_h), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, r_md(), CARD_HOME);
    painter.rect_stroke(rect, r_md(), stroke(CARD_LINE));
    let bar = Rect::from_min_size(
        pos2(rect.left() + S8, rect.top() + S16),
        Vec2::new(S4, rect.height() - S32),
    );
    painter.rect_filled(bar, r_xs(), card.accent);

    let inner = Rect::from_min_max(
        rect.min + v2(S16 + S8, S16),
        rect.max - v2(S16, S16),
    );
    let mut review = false;
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        let head = Rect::from_min_max(inner.min, pos2(inner.right(), inner.top() + S24));
        if card.selectable {
            let check = Rect::from_min_size(head.min, Vec2::splat(S24));
            let on = app.care_picks.contains(&card.screen);
            if task_check(ui, check, card.title, on, !app.busy).clicked() {
                if on {
                    app.care_picks.remove(&card.screen);
                } else {
                    app.care_picks.insert(card.screen);
                }
            }
        }
        let title_x = if card.selectable { head.left() + S24 + S8 } else { head.left() };
        ui.painter().text(
            pos2(title_x, head.center().y),
            Align2::LEFT_CENTER,
            card.title,
            font_regular(CAPTION),
            CARD_MUTED,
        );
        let icon = Rect::from_center_size(pos2(head.right() - S12, head.center().y), Vec2::splat(22.0));
        paint_screen_icon(ui.painter(), card.icon, icon, card.accent);

        let metric_color = if card.success { SUCCESS } else { WHITE };
        ui.painter().text(
            pos2(inner.left(), inner.top() + S32 + S4),
            Align2::LEFT_TOP,
            &card.metric,
            font_semi(STAT),
            metric_color,
        );
        ui.painter().text(
            pos2(inner.left(), inner.top() + S32 + S4 + STAT + S4),
            Align2::LEFT_TOP,
            &card.detail,
            font_regular(SMALL),
            CARD_MUTED,
        );

        if let Some(label) = card.action {
            let galley = ui.painter().layout_no_wrap(label.to_owned(), font_semi(SMALL), INK);
            let w = (galley.size().x + S16 * 2.0).max(96.0);
            let btn = Rect::from_min_size(
                pos2(inner.left(), inner.bottom() - BTN_MIN_H),
                Vec2::new(w, BTN_MIN_H),
            );
            if contrast_button_at(ui, btn, &format!("care-{}", card.title), label, !app.busy).clicked() {
                review = true;
            }
        }
    });
    review
}

fn task_check(ui: &mut egui::Ui, rect: Rect, salt: &str, on: bool, enabled: bool) -> egui::Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, Id::new(("care-check", salt)), sense);
    let box_rect = Rect::from_center_size(rect.center(), Vec2::splat(S16));
    let painter = ui.painter();
    if on {
        painter.rect_filled(box_rect, r_xs(), if enabled { WHITE } else { with_alpha(WHITE, 140) });
        let a = box_rect.left_center() + v2(3.0, 1.0);
        let b = box_rect.center() + v2(-1.0, 4.0);
        let c = box_rect.right_center() + v2(-3.0, -4.0);
        painter.line_segment([a, b], Stroke::new(2.0, INK));
        painter.line_segment([b, c], Stroke::new(2.0, INK));
    } else {
        let fill = if response.hovered() { with_alpha(WHITE, 36) } else { Color32::TRANSPARENT };
        painter.rect_filled(box_rect, r_xs(), fill);
        painter.rect_stroke(box_rect, r_xs(), Stroke::new(FOCUS_W, if enabled { WHITE } else { CARD_MUTED }));
    }
    if response.has_focus() {
        painter.rect_stroke(box_rect.expand(S4), r_xs(), focus_stroke());
    }
    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn contrast_button(ui: &mut egui::Ui, salt: &str, label: &str, enabled: bool, min_w: f32) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font_semi(BODY), INK);
    let w = (galley.size().x + S16 * 2.0).max(min_w);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, CTA_H), Sense::hover());
    contrast_button_at(ui, rect, salt, label, enabled)
}

fn contrast_button_at(ui: &mut egui::Ui, rect: Rect, salt: &str, label: &str, enabled: bool) -> egui::Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, Id::new(("contrast", salt)), sense);
    let pressed = enabled && response.is_pointer_button_down_on();
    let fill = if !enabled {
        BTN_DISABLED
    } else if pressed {
        BTN_PRESSED
    } else if response.hovered() {
        BTN_HOVER
    } else {
        WHITE
    };
    let text = if enabled { INK } else { CARD_MUTED };
    ui.painter().rect_filled(rect, r_pill(), fill);
    if enabled && response.has_focus() {
        ui.painter().rect_stroke(rect.expand(S4), r_pill(), focus_stroke());
    }
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_semi(BODY), text);
    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn home_label(home: &Path) -> String {
    home.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Home")
        .to_string()
}

fn volume_menu(ui: &mut egui::Ui, app: &mut ScanApp) {
    let home = home_label(&app.home);
    let selected = if app.landing_volume == 0 {
        home.clone()
    } else {
        "Macintosh HD".into()
    };
    egui::ComboBox::from_id_salt("landing_volume")
        .width(200.0)
        .selected_text(RichText::new(selected).color(WHITE))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut app.landing_volume, 0, home);
            ui.selectable_value(&mut app.landing_volume, 1, "Macintosh HD");
        });
}

fn module_landing(
    ui: &mut egui::Ui,
    app: &mut ScanApp,
    hero: icons::HeroKind,
    title: &str,
    subtitle: &str,
    items: &[(&str, icons::UiIcon)],
) {
    let accent = tone_for(app.screen).accent;
    ui.add_space(S32);
    let wide = !app.layout.compact && ui.available_width() > 820.0;
    let mut copy = |ui: &mut egui::Ui| {
        ui.label(RichText::new(title).font(heading_font(HERO)).color(WHITE));
        ui.add_space(S8);
        ui.label(
            RichText::new(subtitle)
                .size(BODY)
                .line_height(Some(lh(BODY)))
                .color(with_alpha(WHITE, 210)),
        );
        ui.add_space(S16);
        for (label, icon) in items {
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                paint_ui_icon(ui.painter(), *icon, icon_rect, with_alpha(WHITE, 230));
                ui.label(RichText::new(*label).size(BODY).color(WHITE));
            });
            ui.add_space(S4);
        }
        ui.add_space(S12);
        volume_menu(ui, app);
    };
    let art = |ui: &mut egui::Ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(260.0), Sense::hover());
        icons::paint_hero(ui.painter(), hero, rect, accent);
    };
    if wide {
        ui.horizontal(|ui| {
            ui.add_space(S32);
            art(ui);
            ui.add_space(S32);
            ui.vertical(|ui| {
                ui.set_width(380.0);
                copy(ui);
            });
        });
    } else {
        ui.vertical_centered(|ui| {
            art(ui);
            ui.add_space(S16);
            ui.vertical(|ui| {
                ui.set_max_width(420.0);
                copy(ui);
            });
        });
    }
    ui.add_space(S24);
}

fn cleanup_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Cleanup);
    let busy = app.busy;
    banner(ui, app);
    let remeasure = page_header_with_action(
        ui,
        app.layout,
        Screen::Cleanup,
        "Apps keep temporary files (caches and logs) to run faster. They are safe to remove, and apps recreate what they need.",
        "Measure again",
        !busy && !app.junk.is_empty(),
    );
    if remeasure {
        app.loaded.remove(&Screen::Cleanup);
        app.load(Screen::Cleanup);
    }

    if app.junk.is_empty() {
        loading_card(ui, busy, "Measuring caches and logs...", "Nothing to show yet.");
        return;
    }

    let total: u64 = app.junk.iter().filter(|r| r.exists).map(|r| r.bytes).sum();
    ui.label(
        RichText::new(format!("{} reclaimable", format_bytes(total)))
            .size(H2)
            .strong()
            .color(WHITE)
            .monospace(),
    );
    ui.add_space(S12);
    section(ui, "WHAT TAKES THE SPACE");

    let mut request = None;
    let mut reveal_path = None;
    let (present, missing): (Vec<_>, Vec<_>) = app.junk.iter().partition(|r| r.exists);
    scroll_list(ui, "junk_list", ui.available_height() - S8, |ui| {
        for root in present {
            let mut show = false;
            let mut trash = false;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&root.label).strong().size(TITLE));
                    ui.label(
                        RichText::new(describe_junk(&root.label))
                            .color(TEXT_SECONDARY)
                            .size(SMALL)
                            .line_height(Some(lh(SMALL))),
                    );
                });
                list_actions(
                    ui,
                    &format_bytes(root.bytes),
                    true,
                    !busy && root.bytes > 0,
                    accent,
                    &root.label,
                    &mut show,
                    &mut trash,
                );
            });
            ui.add_space(S8);
            if show {
                reveal_path = Some(root.path.clone());
            }
            if trash {
                request = Some(Confirm {
                    heading: format!("Move {} to the Trash?", root.label.to_lowercase()),
                    body: format!(
                        "{} will be moved to the Trash. Apps recreate these files when needed.",
                        format_bytes(root.bytes)
                    ),
                    action: "Move to Trash",
                    kind: ConfirmKind::TrashRoot(root.path.clone()),
                });
            }
        }
    });
    if !missing.is_empty() {
        ui.add_space(S8);
        section(ui, "NOT FOUND ON THIS MAC");
        card(ui, |ui| {
            for root in missing {
                ui.label(
                    RichText::new(&root.label)
                        .size(SMALL)
                        .color(TEXT_SECONDARY),
                );
            }
        });
    }
    if let Some(path) = reveal_path {
        reveal(&path);
    }
    if request.is_some() {
        app.confirm = request;
    }
}

fn apps_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Apps);
    let busy = app.busy;
    banner(ui, app);
    let scan = page_header_with_action(
        ui,
        app.layout,
        Screen::Apps,
        "Review installed applications and leftover files in your Library.",
        "Scan",
        !busy && !app.loaded.contains(&Screen::Apps),
    );
    if scan {
        app.load(Screen::Apps);
    }
    if !app.loaded.contains(&Screen::Apps) {
        return;
    }

    if app.apps.is_empty() {
        loading_card(ui, busy, "Looking for installed apps...", "No apps found in /Applications or ~/Applications.");
        return;
    }

    tab_bar(ui, "apps_tabs", &["Installed apps", "File leftovers"], &mut app.apps_tab, accent);

    if app.apps_tab == 0 {
        ui.horizontal(|ui| {
            text_field(ui, &mut app.app_filter, "Search by app name", true, false);
            ui.label(
                RichText::new(format!("{} apps", app.apps.len()))
                    .color(TEXT_SECONDARY)
                    .size(SMALL),
            );
        });
        ui.add_space(S8);
        let needle = app.app_filter.trim().to_lowercase();
        let mut pick = None;
        let list_h = ui.available_height() - S16;
        scroll_list(ui, "apps_list", list_h, |ui| {
            let mut shown = 0;
            for (index, item) in app.apps.iter().enumerate() {
                if !needle.is_empty()
                    && !item.name.to_lowercase().contains(&needle)
                    && !item.bundle_id.to_lowercase().contains(&needle)
                {
                    continue;
                }
                shown += 1;
                let selected = app.selected_app == Some(index);
                let r = row(ui, selected, true, |ui| {
                    ui.horizontal(|ui| {
                        avatar(ui, &item.name, accent);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = S4;
                            ui.label(RichText::new(&item.name).strong());
                            ui.label(
                                RichText::new(&item.bundle_id)
                                    .size(CAPTION)
                                    .line_height(Some(lh(CAPTION)))
                                    .color(TEXT_SECONDARY),
                            );
                        });
                    });
                });
                if r.clicked() && !busy {
                    pick = Some(index);
                }
            }
            if shown == 0 {
                ui.label(RichText::new("No app matches your search.").color(TEXT_SECONDARY));
            }
        });
        if let Some(index) = pick {
            app.selected_app = Some(index);
            app.apps_tab = 1;
            app.leftovers.clear();
            app.leftovers_done = false;
            app.survey_selected();
        }
    } else {
        match app.selected_app.and_then(|i| app.apps.get(i)) {
            None => empty(ui, "Pick an app", "Select an app on the Installed apps tab to survey leftovers."),
            Some(selected) => {
                let name = selected.name.clone();
                let app_path = selected.path.clone();
                ui.label(RichText::new(format!("Leftovers: {name}")).strong().size(TITLE));
                ui.add_space(S8);
                if !app.leftovers_done {
                    loading_card(ui, true, "Searching your Library...", "");
                } else if app.leftovers.is_empty() {
                    empty(ui, "No leftover files found", "This app does not keep anything in the usual Library folders.");
                } else {
                    let mut reveal_path = None;
                    scroll_list(ui, "leftovers", ui.available_height() - S32, |ui| {
                        for hit in &app.leftovers {
                            let mut show = false;
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(file_name(&hit.path)).strong());
                                    ui.label(
                                        RichText::new(shorten(&pretty(&app.home, &hit.path), 70))
                                            .size(CAPTION)
                                            .color(TEXT_SECONDARY),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ghost_accent(ui, "Show", true, accent).clicked() {
                                        show = true;
                                    }
                                });
                            });
                            if show {
                                reveal_path = Some(hit.path.clone());
                            }
                        }
                    });
                    if let Some(path) = reveal_path {
                        reveal(&path);
                    }
                }
                if ghost_accent(ui, "Show app in Finder", true, accent).clicked() {
                    reveal(&app_path);
                }
            }
        }
    }
}

fn performance_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Performance);
    let busy = app.busy;
    banner(ui, app);
    let scan = page_header_with_action(
        ui,
        app.layout,
        Screen::Performance,
        "Maintenance tools, login items, and background startup entries for your account.",
        "Scan",
        !busy && !app.loaded.contains(&Screen::Performance),
    );
    if scan {
        app.load(Screen::Performance);
    }
    if !app.loaded.contains(&Screen::Performance) {
        empty(ui, "Press Scan", "Sable will list maintenance actions and startup items.");
        return;
    }

    tab_bar(
        ui,
        "perf_tabs",
        &["Maintenance", "Login items", "Background"],
        &mut app.perf_tab,
        accent,
    );

    if app.perf_tab != 0 {
        ui.label(
            RichText::new(format!("{} startup items", app.agents.len()))
                .color(TEXT_SECONDARY)
                .size(SMALL),
        );
        ui.add_space(S8);
        if app.agents.is_empty() {
            loading_card(ui, busy, "Reading startup items...", "No startup items found for your account.");
        } else {
            scroll_list(ui, "agents", ui.available_height() - S8, |ui| {
                for agent in &app.agents {
                    row(ui, false, false, |ui| {
                        ui.horizontal(|ui| {
                            avatar(ui, &agent.label, accent);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(shorten(&agent.label, 60)).strong());
                                ui.label(
                                    RichText::new(if agent.path.starts_with(&app.home) {
                                        "Your account"
                                    } else {
                                        "All users"
                                    })
                                    .size(CAPTION)
                                    .color(TEXT_SECONDARY),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ghost_accent(ui, "Show", true, accent).clicked() {
                                    reveal(&agent.path);
                                }
                            });
                        });
                    });
                }
            });
        }
        return;
    }

    section(ui, "FIXES AND MAINTENANCE");
    let mut tool = None;
    let mut request = None;
    let layout = app.layout;
    let mut tool_cards = |ui: &mut egui::Ui| {
        tool_card(
            ui,
            "Flush DNS cache",
            "Clears the saved list of website addresses. Try it when sites will not load after switching Wi-Fi or a VPN.",
            Safety::ReadOnly,
            "Flush DNS",
            !busy,
            &mut || tool = Some(Tool::Dns),
        );
        tool_card(
            ui,
            "Check Spotlight",
            "Shows whether Spotlight search is indexing your disk. Only reads the status.",
            Safety::ReadOnly,
            "Check status",
            !busy,
            &mut || tool = Some(Tool::SpotlightStatus),
        );
        tool_card(
            ui,
            "Rebuild Spotlight",
            "Starts a fresh index when search results are wrong or missing. Can take a while and may ask for your password.",
            Safety::Admin,
            "Rebuild index",
            !busy,
            &mut || {
                request = Some(Confirm {
                    heading: "Rebuild the Spotlight index?".into(),
                    body: "Search may be slow or incomplete for a while as macOS reindexes your disk. Your files are not touched.".into(),
                    action: "Rebuild",
                    kind: ConfirmKind::Reindex,
                })
            },
        );
    };
    match layout.grid_cols(3) {
        1 => tool_cards(ui),
        2 => {
            ui.columns(2, |cols| {
                tool_card(
                    &mut cols[0],
                    "Flush DNS cache",
                    "Clears the saved list of website addresses. Try it when sites will not load after switching Wi-Fi or a VPN.",
                    Safety::ReadOnly,
                    "Flush DNS",
                    !busy,
                    &mut || tool = Some(Tool::Dns),
                );
                tool_card(
                    &mut cols[1],
                    "Check Spotlight",
                    "Shows whether Spotlight search is indexing your disk. Only reads the status.",
                    Safety::ReadOnly,
                    "Check status",
                    !busy,
                    &mut || tool = Some(Tool::SpotlightStatus),
                );
            });
            tool_card(
                ui,
                "Rebuild Spotlight",
                "Starts a fresh index when search results are wrong or missing. Can take a while and may ask for your password.",
                Safety::Admin,
                "Rebuild index",
                !busy,
                &mut || {
                    request = Some(Confirm {
                        heading: "Rebuild the Spotlight index?".into(),
                        body: "Search may be slow or incomplete for a while as macOS reindexes your disk. Your files are not touched.".into(),
                        action: "Rebuild",
                        kind: ConfirmKind::Reindex,
                    })
                },
            );
        }
        _ => {
            ui.columns(3, |cols| {
                tool_card(
                    &mut cols[0],
                    "Flush DNS cache",
                    "Clears the saved list of website addresses. Try it when sites will not load after switching Wi-Fi or a VPN.",
                    Safety::ReadOnly,
                    "Flush DNS",
                    !busy,
                    &mut || tool = Some(Tool::Dns),
                );
                tool_card(
                    &mut cols[1],
                    "Check Spotlight",
                    "Shows whether Spotlight search is indexing your disk. Only reads the status.",
                    Safety::ReadOnly,
                    "Check status",
                    !busy,
                    &mut || tool = Some(Tool::SpotlightStatus),
                );
                tool_card(
                    &mut cols[2],
                    "Rebuild Spotlight",
                    "Starts a fresh index when search results are wrong or missing. Can take a while and may ask for your password.",
                    Safety::Admin,
                    "Rebuild index",
                    !busy,
                    &mut || {
                        request = Some(Confirm {
                            heading: "Rebuild the Spotlight index?".into(),
                            body: "Search may be slow or incomplete for a while as macOS reindexes your disk. Your files are not touched.".into(),
                            action: "Rebuild",
                            kind: ConfirmKind::Reindex,
                        })
                    },
                );
            });
        }
    }
    if let Some(kind) = tool {
        app.run_tool(kind);
    }
    if request.is_some() {
        app.confirm = request;
    }

}

fn protection_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Protection);
    banner(ui, app);
    page_header(
        ui,
        app.layout,
        Screen::Protection,
        "Lists files of the types you choose in a folder. It does not check for viruses and never changes your files.",
    );

    let busy = app.busy;
    if app.access != AccessStatus::Granted {
        card(ui, |ui| {
            ui.label(RichText::new("Sable has limited disk access").color(WARN).strong());
            ui.label(
                RichText::new("Some folders cannot be read and will be skipped. Grant Full Disk Access to include them.")
                    .color(TEXT_2)
                    .size(SMALL).line_height(Some(lh(SMALL))),
            );
            if ghost(ui, "Open Privacy settings", true).clicked() {
                open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
            }
        });
        ui.add_space(S8);
    }

    card(ui, |ui| {
        step_label(ui, "1", "Where should it look?");
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = v2(S8, S8);
            for (name, sub) in [("Downloads", "Downloads"), ("Documents", "Documents"), ("Desktop", "Desktop"), ("Home folder", "")] {
                let target = if sub.is_empty() { home.clone() } else { home.join(sub) }.display().to_string();
                if chip_accent(ui, name, app.root == target, accent).clicked() && !busy {
                    app.root = target;
                }
            }
        });
        field(ui, &mut app.root, !busy);

        ui.add_space(S8);
        step_label(ui, "2", "Which kinds of files?");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = v2(S8, S8);
            for (name, exts) in [
                ("Installers", "dmg,pkg"),
                ("Archives", "zip,rar,7z,tar,gz"),
                ("Disk images", "iso,img,dmg"),
                ("Videos", "mov,mp4,mkv,avi"),
                ("Documents", "pdf,doc,docx,pages"),
            ] {
                if chip_accent(ui, name, app.watch == exts, accent).clicked() && !busy {
                    app.watch = exts.into();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Extensions:").color(WHITE).size(SMALL).line_height(Some(lh(SMALL))));
            ui.add(
                egui::TextEdit::singleline(&mut app.watch)
                    .desired_width(300.0)
                    .interactive(!busy)
                    .hint_text("dmg, pkg, zip")
                    .margin(v2(S8, S8))
                    .font(TextStyle::Monospace),
            );
        });

        ui.add_space(S8);
        ui.horizontal(|ui| {
            if primary_accent(ui, "Start searching", !busy, accent).clicked() {
                app.start_scan();
            }
            if busy {
                let paused = app.control.as_ref().is_some_and(|c| c.is_paused());
                if ghost_accent(ui, if paused { "Resume" } else { "Pause" }, true, accent).clicked() {
                    if let Some(control) = &app.control {
                        if paused {
                            control.resume();
                            app.status = "scanning";
                        } else {
                            control.pause();
                            app.status = "paused";
                        }
                    }
                }
                if ghost_accent(ui, "Stop", true, accent).clicked() {
                    if let Some(control) = &app.control {
                        control.cancel();
                        app.status = "canceling";
                    }
                }
            }
        });
    });
    ui.add_space(S12);

    if busy && app.scan_ran {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(format!(
                        "{} {} files so far, {} match",
                        if app.status == "paused" { "Paused after checking" } else { "Checked" },
                        format_count(app.scanned),
                        format_count(app.detection_count)
                    ))
                    .strong(),
                );
            });
            if !app.last_path.is_empty() {
                ui.label(RichText::new(shorten(&app.last_path, 90)).monospace().size(CAPTION).line_height(Some(lh(CAPTION))).color(TEXT_2));
            }
        });
        ui.add_space(S8);
    } else if app.scan_ran {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                cmp_label_small(ui, "MATCHES");
                ui.label(
                    RichText::new(format_count(app.detection_count))
                        .size(STAT)
                        .strong()
                        .color(accent)
                        .monospace(),
                );
            });
            ui.add_space(S24);
            ui.vertical(|ui| {
                cmp_label_small(ui, "FILES CHECKED");
                ui.label(
                    RichText::new(format_count(app.scanned))
                        .size(H2)
                        .color(WHITE)
                        .monospace(),
                );
            });
            ui.add_space(S24);
            ui.vertical(|ui| {
                cmp_label_small(ui, "COULD NOT READ");
                ui.label(
                    RichText::new(format_count(app.errors))
                        .size(H2)
                        .color(if app.errors > 0 { WARN } else { WHITE })
                        .monospace(),
                );
            });
        });
        ui.add_space(S8);
    }

    if app.detections.is_empty() {
        if !app.scan_ran {
            empty(ui, "Nothing searched yet", "Choose a folder and file types, then press Start searching.");
        } else if !busy {
            empty(ui, "No matching files", "Nothing in that folder uses the extensions you picked.");
        }
    } else {
        section(ui, &format!("{} MATCHING FILES", format_count(app.detections.len() as u64)));
        let mut reveal_path = None;
        card(ui, |ui| {
            for hit in app.detections.iter().take(300) {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = S4;
                        ui.label(RichText::new(file_name(&hit.path)).strong());
                        ui.label(
                            RichText::new(shorten(&pretty(&app.home, hit.path.parent().unwrap_or(Path::new(""))), 70))
                                .size(CAPTION).line_height(Some(lh(CAPTION)))
                                .color(TEXT_2),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(hit.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(hit.size)).color(TEXT).strong());
                        badge(ui, &format!(".{}", hit.extension), INFO);
                    });
                });
            }
            if app.detections.len() > 300 {
                ui.label(RichText::new(format!("and {} more", app.detections.len() - 300)).color(TEXT_2));
            }
        });
        if let Some(path) = reveal_path {
            reveal(&path);
        }
    }
}

fn malware_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Malware);
    banner(ui, app);
    if page_header_with_action(
        ui,
        app.layout,
        Screen::Malware,
        "Compares files in a folder to the public MalwareBazaar hash list. Matches move to Trash only after you confirm.",
        "Scan",
        !app.busy,
    ) {
        app.start_threat();
    }

    card(ui, |ui| {
        ui.label(RichText::new("What this scan does not do").strong().color(WHITE));
        for line in [
            "Does not read browser cookies, history, passwords, or messages.",
            "Does not change camera, microphone, or other app permissions.",
            "Does not watch the network in the background.",
        ] {
            ui.label(RichText::new(format!("• {line}")).color(TEXT_SECONDARY).size(SMALL));
        }
        ui.add_space(S8);
        if ghost_accent(ui, "Open Privacy settings", true, accent).clicked() {
            open_url("x-apple.systempreferences:com.apple.preference.security?Privacy");
        }
    });
    ui.add_space(S8);

    let busy = app.busy;
    card(ui, |ui| {
        ui.label(RichText::new("Folder to scan").strong().color(WHITE));
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = v2(S8, S8);
            for (name, sub) in [
                ("Downloads", "Downloads"),
                ("Desktop", "Desktop"),
                ("Documents", "Documents"),
                ("Launch Agents", "Library/LaunchAgents"),
            ] {
                let target = home.join(sub).display().to_string();
                if chip_accent(ui, name, app.threat_root == target, accent).clicked() && !busy {
                    app.threat_root = target;
                }
            }
        });
        field(ui, &mut app.threat_root, !busy);
        ui.add_space(S8);
        ui.label(
            RichText::new("The first scan downloads the public hash list if needed. Later scans reuse the saved copy.")
                .color(TEXT_SECONDARY)
                .size(SMALL)
                .line_height(Some(lh(SMALL))),
        );
    });
    ui.add_space(S12);

    if busy {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Updating signatures if needed, then hashing files.").strong());
            });
        });
        return;
    }

    let Some(report) = app.threat.clone() else {
        empty(ui, "Nothing scanned yet", "Choose a folder and press Scan.");
        return;
    };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            cmp_label_small(ui, "FILES HASHED");
            ui.label(
                RichText::new(format_count(report.scanned))
                    .size(H2)
                    .color(WHITE)
                    .monospace(),
            );
        });
        ui.add_space(S24);
        ui.vertical(|ui| {
            cmp_label_small(ui, "SIGNATURES");
            ui.label(
                RichText::new(&report.signatures_note)
                    .color(TEXT_SECONDARY)
                    .size(SMALL),
            );
        });
    });
    ui.add_space(S8);
    if report.skipped_large > 0 {
        theme::notice_frame(WARN).show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "{} files larger than 50 MB were skipped.",
                    format_count(report.skipped_large)
                ))
                .color(WARN)
                .strong(),
            );
            if ghost_accent(ui, "Why skipped?", true, accent).clicked() {
                app.malware_show_skipped = !app.malware_show_skipped;
            }
        });
        ui.add_space(S8);
    }
    if report.unreadable > 0 {
        ui.label(
            RichText::new(format!("{} items could not be read.", format_count(report.unreadable)))
                .color(TEXT_SECONDARY)
                .size(SMALL),
        );
    }

    if report.hits.is_empty() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(S24), Sense::hover());
                paint_ui_icon(ui.painter(), UiIcon::Check, icon_rect, SUCCESS);
                ui.add_space(S8);
                ui.vertical(|ui| {
                    ui.label(RichText::new("No known malware hashes").strong().color(SUCCESS));
                    ui.label(
                        RichText::new("Nothing in that folder matched the public list. This is not a guarantee your Mac is clean.")
                            .color(TEXT_SECONDARY)
                            .size(SMALL),
                    );
                });
            });
        });
        return;
    }

    section(ui, &format!("{} MATCHES", format_count(report.hits.len() as u64)));
    let mut reveal_path = None;
    let mut ignore_path = None;
    let mut trash_path = None;
    card(ui, |ui| {
        for hit in &report.hits {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = S4;
                    ui.label(RichText::new(file_name(&hit.path)).strong());
                    ui.label(
                        RichText::new(shorten(&hit.path.display().to_string(), 72))
                            .size(CAPTION).line_height(Some(lh(CAPTION)))
                            .color(TEXT_2),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost(ui, "Trash", true).clicked() {
                        trash_path = Some(hit.path.clone());
                    }
                    if ghost(ui, "Ignore", true).clicked() {
                        ignore_path = Some(hit.path.clone());
                    }
                    if ghost(ui, "Show", true).clicked() {
                        reveal_path = Some(hit.path.clone());
                    }
                    badge(ui, "hash match", DANGER);
                });
            });
        }
    });
    if let Some(path) = reveal_path {
        reveal(&path);
    }
    if let Some(path) = ignore_path {
        app.ignore_threat(path);
    }
    if let Some(path) = trash_path {
        let name = file_name(&path);
        app.confirm = Some(Confirm {
            heading: format!("Move {name} to the Trash?"),
            body: "This file matched a public malware hash. The Trash can restore it. Related helpers and system files are left alone.".into(),
            action: "Move to Trash",
            kind: ConfirmKind::TrashThreat(path),
        });
    }
}

fn clutter_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Clutter);
    let busy = app.busy;
    banner(ui, app);
    let rescan = page_header_with_action(
        ui,
        app.layout,
        Screen::Clutter,
        "Large files (50 MB or more) in your Downloads folder.",
        "Scan",
        !busy && (!app.loaded.contains(&Screen::Clutter) || app.clutter.is_empty()),
    );
    if rescan {
        app.loaded.insert(Screen::Clutter);
        app.clutter.clear();
        app.start_clutter();
    }
    if !app.loaded.contains(&Screen::Clutter) {
        empty(ui, "Press Scan", "Sable will list large downloads.");
        return;
    }

    if app.clutter.is_empty() {
        if busy {
            loading_card(ui, true, "Looking through Downloads...", "");
        } else {
            empty(ui, "Downloads is tidy", "No file there is 50 MB or larger.");
        }
        return;
    }

    let total: u64 = app.clutter.iter().map(|f| f.bytes).sum();
    ui.label(
        RichText::new(format!("{} across {} files", format_bytes(total), app.clutter.len()))
            .color(WHITE)
            .strong()
            .monospace(),
    );
    ui.add_space(S12);

    let mut request = None;
    let mut reveal_path = None;
    scroll_list(ui, "clutter_list", ui.available_height() - S8, |ui| {
        for (index, file) in app.clutter.iter().enumerate() {
            ui.push_id(index, |ui| {
                let mut show = false;
                let mut trash = false;
                row(ui, false, false, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = S4;
                            ui.label(RichText::new(shorten(&file_name(&file.path), 55)).strong());
                            ui.label(
                                RichText::new(shorten(
                                    &pretty(&app.home, file.path.parent().unwrap_or(Path::new(""))),
                                    70,
                                ))
                                .size(CAPTION)
                                .color(TEXT_SECONDARY),
                            );
                        });
                        list_actions(
                            ui,
                            &format_bytes(file.bytes),
                            true,
                            !busy,
                            accent,
                            &format!("clutter_{index}"),
                            &mut show,
                            &mut trash,
                        );
                    });
                });
                if show {
                    reveal_path = Some(file.path.clone());
                }
                if trash {
                    request = Some(Confirm {
                        heading: format!("Move \"{}\" to the Trash?", file_name(&file.path)),
                        body: format!(
                            "This frees {}. You can restore it from the Trash.",
                            format_bytes(file.bytes)
                        ),
                        action: "Move to Trash",
                        kind: ConfirmKind::TrashDownload(file.path.clone()),
                    });
                }
            });
        }
    });
    if let Some(path) = reveal_path {
        reveal(&path);
    }
    if request.is_some() {
        app.confirm = request;
    }
}

fn space_item_kind(node: &SpaceNode) -> String {
    if node.is_directory {
        "Folder".into()
    } else {
        node.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_else(|| "File".into())
    }
}

fn format_modified(secs: Option<u64>) -> String {
    let Some(secs) = secs else {
        return "Modified date unknown".into();
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(secs);
    let days = now.saturating_sub(secs) / 86_400;
    if days == 0 {
        "Modified today".into()
    } else if days == 1 {
        "Modified yesterday".into()
    } else if days < 30 {
        format!("Modified {days} days ago")
    } else if days < 365 {
        format!("Modified {} months ago", days / 30)
    } else {
        format!("Modified {} years ago", days / 365)
    }
}

fn space_lens_permission_card(ui: &mut egui::Ui, app: &ScanApp) {
    let home = path_place(&app.home.display().to_string());
    let here = path_place(app.space_root.trim());
    let at_home = here == home || here.starts_with(&format!("{home}/"));
    let home_denied = at_home && home_folder_access(&app.home) == AccessStatus::Denied;

    if app.space_scan_blocked_message().is_some() {
        card(ui, |ui| {
            ui.label(RichText::new("Full Disk Access needed").color(WARN).strong());
            ui.label(
                RichText::new(
                    "To scan the whole Mac, allow Full Disk Access for Sable in System Settings. Without it, scanning stops here.",
                )
                .color(TEXT_2)
                .size(SMALL)
                .line_height(Some(lh(SMALL))),
            );
            if ghost(ui, "Open Privacy settings", true).clicked() {
                open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
            }
        });
        ui.add_space(S8);
    }

    if home_denied {
        card(ui, |ui| {
            ui.label(RichText::new("Desktop & Downloads access").color(WARN).strong());
            ui.label(
                RichText::new(
                    "macOS blocks apps from reading Desktop, Downloads, and Documents until you allow it. \
                     In System Settings → Privacy & Security → Files and Folders, enable those folders for Sable, \
                     or grant Full Disk Access, then press Scan again.",
                )
                .color(TEXT_2)
                .size(SMALL)
                .line_height(Some(lh(SMALL))),
            );
            if ghost(ui, "Open Privacy settings", true).clicked() {
                open_url("x-apple.systempreferences:com.apple.preference.security?Privacy");
            }
        });
        ui.add_space(S8);
    }
}

fn space_lens_hover_card(ctx: &egui::Context, home: &Path, hover: &SpaceNode) {
    let pointer = ctx.input(|i| i.pointer.hover_pos()).unwrap_or_else(|| ctx.screen_rect().center());
    egui::Area::new(egui::Id::new("space_lens_hover"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pointer + v2(S16, S16))
        .show(ctx, |ui| {
            egui::Frame::popup(&ctx.style()).show(ui, |ui| {
                ui.set_max_width(360.0);
                ui.label(RichText::new(file_name(&hover.path)).strong());
                ui.label(RichText::new(pretty(home, &hover.path)).size(CAPTION).line_height(Some(lh(CAPTION))).color(TEXT_2).monospace());
                ui.separator();
                ui.label(RichText::new(format!("Type: {}", space_item_kind(hover))).size(SMALL).line_height(Some(lh(SMALL))));
                ui.label(RichText::new(format!("Size: {}", format_bytes(hover.bytes))).size(SMALL).line_height(Some(lh(SMALL))));
                ui.label(RichText::new(format_modified(hover.modified)).size(SMALL).line_height(Some(lh(SMALL))).color(TEXT_2));
                ui.label(RichText::new(cloud_label(hover.cloud)).size(CAPTION).line_height(Some(lh(CAPTION))).color(TEXT_2));
                if hover.is_directory {
                    ui.label(RichText::new("Click to zoom into this folder").size(CAPTION).line_height(Some(lh(CAPTION))).color(ACCENT_TEXT));
                }
            });
        });
}

fn space_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    let accent = accent_for_screen(Screen::Space);
    ui.ctx().input(|input| {
        if input.key_pressed(egui::Key::ArrowLeft) && input.modifiers.alt {
            app.space_back();
        }
        if input.key_pressed(egui::Key::ArrowRight) && input.modifiers.alt {
            app.space_forward();
        }
    });
    banner(ui, app);
    ui.horizontal(|ui| {
        if page_header_with_action(
            ui,
            app.layout,
            Screen::Space,
            "Visual map of what uses disk space on the selected volume.",
            "Scan",
            !app.busy,
        ) {
            app.start_space(true);
        }
        if app.busy && ghost(ui, "Cancel", true).clicked() {
            app.space_scan_id = app.space_scan_id.wrapping_add(1);
            app.finish();
        }
    });
    space_top_bar(ui, app, accent);
    space_lens_permission_card(ui, app);

    let busy = app.busy;
    let folder_path = PathBuf::from(app.space_root.trim());
    if app.space_volume.is_none() {
        let footer_h = 72.0;
        let body_h = (ui.available_height() - footer_h).max(120.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), body_h),
            Layout::top_down(Align::Center),
            |ui| {
                ui.add_space((body_h * 0.32).min(160.0));
                if busy {
                    let target = volume_title(app.space_root.trim());
                    loading_line(ui, &format!("Building storage map for {target}…"));
                } else {
                    ui.label(
                        RichText::new("Press Scan to map this disk.")
                            .color(with_alpha(WHITE, 220))
                            .size(BODY),
                    );
                }
            },
        );
        space_footer(ui, app, 0, 0);
        return;
    }

    let mut gesture = SpaceGesture::default();
    let footer_h = 78.0;
    let body_h = (ui.available_height() - footer_h).max(260.0);
    let compact = app.layout.compact || ui.available_width() < 860.0;
    let in_map = app
        .space_volume
        .as_ref()
        .is_some_and(|volume| space_folder(volume, &folder_path).is_some());
    if !in_map {
        empty(ui, "Not in this map", "Press Scan to include this folder.");
        space_footer(ui, app, 0, 0);
        return;
    }
    let selected = &app.space_selected;
    let home = &app.home;
    let volume = app.space_volume.as_ref().unwrap();
    let folder = space_folder(volume, &folder_path).unwrap();
        let mut bubbles = Vec::new();
        let mut shown = 0u64;
        let total = folder.total.max(1);
        for child in folder.children {
            if (child.bytes as f32) / (total as f32) < SPACE_LENS_BUBBLE_THRESHOLD {
                break;
            }
            shown = shown.saturating_add(child.bytes);
            bubbles.push(entity_bubble(child));
        }
        let other_bytes = folder.total.saturating_sub(shown);
        if folder.children.is_empty() {
            empty(ui, "Nothing to show", "This folder is empty or cannot be read.");
        } else if compact {
            space_list(
                ui,
                selected,
                home,
                &volume_title(&folder.path.display().to_string()),
                folder.children,
                220.0,
                busy,
                &mut gesture,
            );
            ui.add_space(S8);
            space_orbit(ui, selected, home, &bubbles, other_bytes, total, 280.0, busy, &mut gesture);
        } else {
            let list_w = 280.0;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = S16;
                ui.allocate_ui_with_layout(Vec2::new(list_w, body_h), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(list_w);
                    space_list(
                        ui,
                        selected,
                        home,
                        &volume_title(&folder.path.display().to_string()),
                        folder.children,
                        body_h,
                        busy,
                        &mut gesture,
                    );
                });
                ui.allocate_ui_with_layout(
                    Vec2::new((ui.available_width() - S8).max(200.0), body_h),
                    Layout::top_down(Align::Min),
                    |ui| {
                        space_orbit(ui, selected, home, &bubbles, other_bytes, total, body_h, busy, &mut gesture);
                    },
                );
            });
        }
        if let Some(node) = &gesture.hover {
            space_lens_hover_card(ui.ctx(), home, node);
        }
    if gesture.clear_selection {
        app.space_selected.clear();
        app.space_selected_bytes.clear();
    }
    if let Some(path) = gesture.reveal {
        reveal(&path);
    }
    if let Some((path, bytes)) = gesture.toggle {
        if app.space_selected.contains(&path) {
            app.space_selected.remove(&path);
            app.space_selected_bytes.remove(&path);
        } else {
            app.space_selected.insert(path.clone());
            app.space_selected_bytes.insert(path, bytes);
        }
    }
    if let Some(node) = gesture.hover {
        app.space_hover = Some(node.path);
    }
    let selected_bytes = app.space_selected_bytes.values().sum();
    space_footer(ui, app, selected_bytes, 0);
    if let Some(path) = gesture.open {
        app.space_open(path);
    }
}

fn space_top_bar(ui: &mut egui::Ui, app: &mut ScanApp, accent: Color32) {
    let mut go_back = false;
    let mut go_forward = false;
    ui.horizontal(|ui| {
        if ghost_accent(ui, "Back", !app.busy && !app.space_history.is_empty(), accent).clicked() {
            go_back = true;
        }
        if ghost_accent(ui, "Forward", !app.busy && !app.space_future.is_empty(), accent).clicked() {
            go_forward = true;
        }
    });
    if go_back {
        app.space_back();
    }
    if go_forward {
        app.space_forward();
    }
    if let Some(path) = space_breadcrumb(ui, app) {
        app.space_open(path);
    }
    let current = path_place(&app.space_root);
    let mac = path_place(&computer_root().display().to_string());
    let home = path_place(&app.home.display().to_string());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = v2(S8, S4);
        if chip_accent(ui, "This Mac", current == mac, accent).clicked() {
            app.space_open(computer_root());
        }
        if chip_accent(ui, "Home", current == home, accent).clicked() {
            app.space_open(app.home.clone());
        }
    });
    ui.add_space(S8);
}

fn space_crumbs(app: &ScanApp) -> Vec<(String, PathBuf)> {
    let here = PathBuf::from(app.space_root.trim());
    let anchor = app
        .space_volume
        .as_ref()
        .map(|volume| volume.scan_root.clone())
        .unwrap_or_else(|| here.clone());
    let anchor_place = path_place(&anchor.display().to_string());
    let here_place = path_place(&here.display().to_string());
    let home = path_place(&app.home.display().to_string());
    let root_label = if anchor_place.is_empty() || anchor_place == "/" {
        "Macintosh HD".to_string()
    } else if anchor_place == home {
        "Home".to_string()
    } else {
        file_name(&anchor)
    };
    let mut crumbs = vec![(root_label, anchor.clone())];
    let relative = if anchor_place.is_empty() {
        here_place.trim_start_matches('/')
    } else if here_place == anchor_place {
        ""
    } else {
        here_place
            .strip_prefix(&format!("{anchor_place}/"))
            .unwrap_or("")
    };
    let mut path = anchor;
    for part in relative.split('/').filter(|part| !part.is_empty()) {
        path = path.join(part);
        crumbs.push((part.to_string(), path.clone()));
    }
    crumbs
}

fn space_breadcrumb(ui: &mut egui::Ui, app: &ScanApp) -> Option<PathBuf> {
    let crumbs = space_crumbs(app);
    let last = crumbs.len().saturating_sub(1);
    let mut open = None;
    ui.add_space(S8);
    egui::ScrollArea::horizontal()
        .id_salt("space_crumbs")
        .max_height(28.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = S4;
                for (index, (label, path)) in crumbs.iter().enumerate() {
                    if index > 0 {
                        ui.label(RichText::new("/").size(SMALL).color(TEXT_3));
                    }
                    if index == last {
                        ui.label(RichText::new(label).font(font_semi(SMALL)).color(WHITE));
                        let track = ui.id().with("space_crumb_track");
                        let key = app.space_root.trim().to_string();
                        let moved = ui.ctx().data(|data| {
                            data.get_temp::<String>(track).map(|prev| prev != key).unwrap_or(true)
                        });
                        if moved {
                            ui.scroll_to_cursor(Some(Align::RIGHT));
                            ui.ctx().data_mut(|data| data.insert_temp(track, key));
                        }
                    } else if crumb_link(ui, label).clicked() && !app.busy {
                        open = Some(path.clone());
                    }
                }
            });
        });
    open
}

fn crumb_link(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font_regular(SMALL), ACCENT_TEXT);
    let (rect, response) = ui.allocate_exact_size(galley.size() + v2(S4, S4), Sense::click());
    let color = if response.hovered() { WHITE } else { ACCENT_TEXT };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_regular(SMALL), color);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

#[derive(Default)]
struct SpaceGesture {
    open: Option<PathBuf>,
    reveal: Option<PathBuf>,
    toggle: Option<(PathBuf, u64)>,
    hover: Option<SpaceNode>,
    clear_selection: bool,
}

fn entity_bubble(entity: &SpaceEntity) -> SpaceNode {
    SpaceNode {
        path: entity.path.clone(),
        bytes: entity.bytes,
        is_directory: entity.is_directory,
        is_purgeable: entity.is_purgeable,
        cloud: entity.cloud,
        leaves: entity.leaves,
        modified: entity.modified,
    }
}

fn path_place(path: &str) -> String {
    let text = path.trim();
    text.strip_prefix("/System/Volumes/Data").unwrap_or(text).to_string()
}

fn space_list(
    ui: &mut egui::Ui,
    selected_paths: &HashSet<PathBuf>,
    home: &Path,
    title: &str,
    kids: &[SpaceEntity],
    height: f32,
    busy: bool,
    gesture: &mut SpaceGesture,
) {
    ui.label(RichText::new(title).font(heading_font(TITLE)).color(WHITE));
    ui.label(
        RichText::new(format!("{} items", kids.len()))
            .size(SMALL)
            .color(with_alpha(WHITE, 190)),
    );
    ui.add_space(S8);
    let select_label = if selected_paths.is_empty() {
        "Select: None".to_string()
    } else {
        format!("Select: {}", selected_paths.len())
    };
    if ghost(ui, &select_label, !busy && !selected_paths.is_empty()).clicked() {
        gesture.clear_selection = true;
    }
    ui.add_space(S8);
    let row_h = 26.0;
    egui::ScrollArea::vertical()
        .id_salt(kids.as_ptr() as usize)
        .max_height((height - 88.0).max(120.0))
        .show_rows(ui, row_h, kids.len(), |ui, range| {
            for index in range {
                let node = &kids[index];
                let selected = selected_paths.contains(&node.path);
                let response = ui
                    .push_id(index, |ui| {
                        let bg = ui.painter().add(Shape::Noop);
                        let inner = ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                            paint_ui_icon(
                                ui.painter(),
                                if node.is_directory { icons::UiIcon::Folder } else { icons::UiIcon::Info },
                                icon_rect,
                                WHITE,
                            );
                            ui.label(RichText::new(shorten(&file_name(&node.path), 22)).color(WHITE).size(SMALL));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format_bytes(node.bytes)).color(WHITE).size(SMALL));
                            });
                        });
                        let fill = if selected {
                            with_alpha(WHITE, 48)
                        } else if inner.response.hovered() {
                            with_alpha(WHITE, 24)
                        } else {
                            Color32::TRANSPARENT
                        };
                        ui.painter().set(bg, Shape::rect_filled(inner.response.rect, r_sm(), fill));
                        inner.response.interact(Sense::click())
                    })
                    .inner;
                if response.hovered() {
                    gesture.hover = Some(entity_bubble(node));
                }
                if !busy && (response.double_clicked() || response.clicked()) && node.is_directory {
                    gesture.open = Some(node.path.clone());
                } else if !busy && response.clicked() && !node.is_directory && !is_spared(&node.path, home) {
                    gesture.toggle = Some((node.path.clone(), node.bytes));
                }
                if response.secondary_clicked() {
                    gesture.reveal = Some(node.path.clone());
                }
            }
        });
}

struct OrbitBubble {
    name: String,
    bytes: u64,
    path: Option<PathBuf>,
    is_dir: bool,
    share: f32,
}

fn space_orbit(
    ui: &mut egui::Ui,
    selected_paths: &HashSet<PathBuf>,
    home: &Path,
    nodes: &[SpaceNode],
    other_bytes: u64,
    total: u64,
    height: f32,
    busy: bool,
    gesture: &mut SpaceGesture,
) {
    let mut bubbles: Vec<OrbitBubble> = nodes
        .iter()
        .map(|node| OrbitBubble {
            name: file_name(&node.path),
            bytes: node.bytes,
            path: Some(node.path.clone()),
            is_dir: node.is_directory,
            share: node.bytes as f32 / total.max(1) as f32,
        })
        .collect();
    if other_bytes > 0 {
        bubbles.push(OrbitBubble {
            name: "Other".into(),
            bytes: other_bytes,
            path: None,
            is_dir: false,
            share: other_bytes as f32 / total.max(1) as f32,
        });
    }
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let shares: Vec<f32> = bubbles.iter().map(|bubble| bubble.share).collect();
    let placed = orbit_layout(rect, &shares);
    let painter = ui.painter();
    for (bubble, (center, radius)) in bubbles.iter().zip(placed.iter()) {
        let selected = bubble.path.as_ref().is_some_and(|path| selected_paths.contains(path));
        painter.circle_filled(*center, *radius, with_alpha(WHITE, if selected { 70 } else { 36 }));
        painter.circle_stroke(*center, *radius, Stroke::new(1.5, with_alpha(WHITE, 140)));
        if *radius > 36.0 {
            painter.text(*center + v2(0.0, -8.0), Align2::CENTER_CENTER, shorten(&bubble.name, 14), font_semi(SMALL), WHITE);
            painter.text(
                *center + v2(0.0, 12.0),
                Align2::CENTER_CENTER,
                format_bytes(bubble.bytes),
                font_regular(CAPTION),
                with_alpha(WHITE, 210),
            );
        }
    }
    let pointer = response.hover_pos();
    if let Some(pos) = pointer {
        if let Some((index, _)) = hit_bubble(pos, &placed) {
            if let Some(node) = nodes.get(index) {
                gesture.hover = Some(node.clone());
            }
        }
    }
    if !busy {
        let pos = response.interact_pointer_pos().or(pointer);
        if let Some(pos) = pos {
            if let Some((index, _)) = hit_bubble(pos, &placed) {
                let bubble = &bubbles[index];
                if response.double_clicked() || response.clicked() {
                    if bubble.is_dir {
                        if let Some(path) = bubble.path.clone() {
                            gesture.open = Some(path);
                        }
                    } else if let Some(path) = &bubble.path {
                        if !is_spared(path, home) {
                            gesture.toggle = Some((path.clone(), bubble.bytes));
                        }
                    }
                }
            }
        }
    }
}

fn orbit_layout(rect: Rect, shares: &[f32]) -> Vec<(egui::Pos2, f32)> {
    if shares.is_empty() {
        return Vec::new();
    }
    let center = rect.center();
    let limit = rect.width().min(rect.height()) * 0.46;
    let main = (limit * (0.42 + 0.22 * shares[0].sqrt())).clamp(64.0, limit * 0.46);
    let mut placed = vec![(center, main)];
    let rest = shares.len().saturating_sub(1);
    for index in 0..rest {
        let share = shares[index + 1];
        let radius = (20.0 + 78.0 * share.sqrt()).clamp(18.0, main * 0.72);
        let angle = -std::f32::consts::FRAC_PI_2 + index as f32 / rest as f32 * std::f32::consts::TAU;
        let dist = main + radius + 14.0;
        let raw = center + Vec2::new(angle.cos(), angle.sin()) * dist;
        let min = rect.min + Vec2::splat(radius + 6.0);
        let max = rect.max - Vec2::splat(radius + 6.0);
        let pos = egui::pos2(raw.x.clamp(min.x, max.x.max(min.x)), raw.y.clamp(min.y, max.y.max(min.y)));
        placed.push((pos, radius));
    }
    placed
}

fn hit_bubble(pos: egui::Pos2, placed: &[(egui::Pos2, f32)]) -> Option<(usize, f32)> {
    let mut best: Option<(usize, f32)> = None;
    for (index, (center, radius)) in placed.iter().enumerate() {
        let distance = pos.distance(*center);
        if distance <= *radius && best.map(|(_, previous)| distance < previous).unwrap_or(true) {
            best = Some((index, distance));
        }
    }
    best
}

fn space_footer(ui: &mut egui::Ui, app: &mut ScanApp, selected_bytes: u64, _folder_total: u64) {
    ui.add_space(S8);
    let (used, capacity) = volume_capacity(Path::new(app.space_root.trim())).unwrap_or_else(|| {
        let used = selected_bytes;
        (used, used.max(1))
    });
    let fraction = if capacity == 0 { 0.0 } else { used as f32 / capacity as f32 };
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(220.0);
            ui.label(
                RichText::new(format!("{} of {} used", format_bytes(used), format_bytes(capacity)))
                    .size(SMALL)
                    .color(WHITE),
            );
            meter(ui, fraction, WHITE);
        });
        let selected_bytes = selected_bytes;
        ui.label(
            RichText::new(format!(
                "{} items selected   {}",
                app.space_selected.len(),
                format_bytes(selected_bytes)
            ))
            .size(SMALL)
            .color(with_alpha(WHITE, 210)),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let removable: Vec<PathBuf> = app
                .space_selected
                .iter()
                .filter(|path| !is_spared(path, &app.home))
                .cloned()
                .collect();
            let accent = accent_for_screen(Screen::Space);
            if primary_accent(ui, "Review and Remove", !app.busy && !removable.is_empty(), accent).clicked() {
                app.confirm = Some(Confirm {
                    heading: "Move selected files to the Trash?".into(),
                    body: format!(
                        "This frees about {}. System files stay protected. You can restore items from the Trash.",
                        format_bytes(selected_bytes)
                    ),
                    action: "Move to Trash",
                    kind: ConfirmKind::TrashSpace(removable),
                });
            }
        });
    });
}

fn volume_capacity(path: &Path) -> Option<(u64, u64)> {
    // macOS `struct statfs` from `<sys/mount.h>`. 2168 bytes on this SDK.
    #[repr(C)]
    struct Statfs {
        f_bsize: u32,
        f_iosize: i32,
        f_blocks: u64,
        f_bfree: u64,
        f_bavail: u64,
        f_files: u64,
        f_ffree: u64,
        _rest: [u8; 2120],
    }
    extern "C" {
        fn statfs(path: *const i8, buf: *mut Statfs) -> i32;
    }
    let text = path.to_string_lossy();
    let c_path = std::ffi::CString::new(text.as_bytes()).ok()?;
    let mut buf = std::mem::MaybeUninit::<Statfs>::zeroed();
    let rc = unsafe { statfs(c_path.as_ptr(), buf.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let stat = unsafe { buf.assume_init() };
    if stat.f_bsize == 0 || stat.f_blocks == 0 {
        return None;
    }
    let block = stat.f_bsize as u64;
    let capacity = stat.f_blocks.saturating_mul(block);
    let free = stat.f_bavail.saturating_mul(block);
    if capacity < free || capacity > 200 * 1024 * 1024 * 1024 * 1024 {
        return None;
    }
    Some((capacity - free, capacity))
}

fn cloud_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Cloud,
        "How much space iCloud Drive and other cloud services use on this Mac. Nothing is signed in, uploaded, or deleted.",
    );
    banner(ui, app);

    let busy = app.busy;
    if app.cloud.is_empty() {
        loading_card(ui, busy, "Measuring cloud folders...", "No cloud folders found.");
        return;
    }
    let biggest = app.cloud.iter().map(|c| c.bytes).max().unwrap_or(1).max(1);
    let mut reveal_path = None;
    for folder in &app.cloud {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&folder.label).strong().size(TITLE));
                    ui.label(
                        RichText::new(if folder.exists {
                            shorten(&pretty(&app.home, &folder.path), 70)
                        } else {
                            "This service is not set up on this Mac.".to_string()
                        })
                        .size(SMALL).line_height(Some(lh(SMALL)))
                        .color(TEXT_2),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if folder.exists {
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(folder.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(folder.bytes)).strong().size(TITLE).color(TEXT));
                    }
                });
            });
            if folder.exists {
                ui.add_space(S4);
                meter(ui, folder.bytes as f32 / biggest as f32, INFO);
            }
        });
    }
    if let Some(path) = reveal_path {
        reveal(&path);
    }
    ui.add_space(S8);
    if ghost(ui, "Measure again", !busy).clicked() {
        app.loaded.remove(&Screen::Cloud);
    }
}

fn assistant_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Assistant,
        "How hard your Mac is working right now. Press Refresh for a new reading.",
    );
    banner(ui, app);

    let busy = app.busy;
    let Some(sample) = app.sample else {
        loading_card(ui, busy, "Taking a reading...", "No reading yet.");
        return;
    };
    let mem_pct = if sample.total_memory > 0 {
        sample.used_memory as f32 / sample.total_memory as f32
    } else {
        0.0
    };
    let cpu = (sample.cpu_percent / 100.0).clamp(0.0, 1.0);

    let stat_card = |ui: &mut egui::Ui, title: &str, value: &str, frac: f32, caption: &str| {
        card(ui, |ui| {
            label_small(ui, title);
            ui.label(RichText::new(value).size(HERO).strong().color(gauge(frac)));
            meter(ui, frac, gauge(frac));
            ui.label(RichText::new(caption).color(TEXT_2).size(SMALL).line_height(Some(lh(SMALL))));
        });
    };
    if app.layout.compact {
        stat_card(
            ui,
            "PROCESSOR",
            &format!("{:.0}%", sample.cpu_percent),
            cpu,
            "How busy the CPU is.",
        );
        stat_card(
            ui,
            "MEMORY",
            &format!("{:.0}%", mem_pct * 100.0),
            mem_pct,
            &format!(
                "{} of {} in use",
                format_bytes(sample.used_memory),
                format_bytes(sample.total_memory)
            ),
        );
    } else {
        ui.columns(2, |cols| {
            stat_card(
                &mut cols[0],
                "PROCESSOR",
                &format!("{:.0}%", sample.cpu_percent),
                cpu,
                "How busy the CPU is.",
            );
            stat_card(
                &mut cols[1],
                "MEMORY",
                &format!("{:.0}%", mem_pct * 100.0),
                mem_pct,
                &format!(
                    "{} of {} in use",
                    format_bytes(sample.used_memory),
                    format_bytes(sample.total_memory)
                ),
            );
        });
    }
    ui.add_space(S8);
    let verdict = if mem_pct > 0.85 {
        ("Memory is nearly full. Quit apps you are not using.", WARN)
    } else if cpu > 0.8 {
        ("The processor is very busy. Something heavy is running.", WARN)
    } else {
        ("Everything looks fine.", SUCCESS)
    };
    card(ui, |ui| {
        ui.label(RichText::new(verdict.0).color(verdict.1).strong());
    });
    ui.add_space(S8);
    if ghost(ui, "Refresh", !busy).clicked() {
        app.sample_machine();
    }
}

// ───────────────────────────── widgets ─────────────────────────────

#[derive(Clone, Copy)]
enum Safety {
    ReadOnly,
    Trash,
    Admin,
}

fn safety(ui: &mut egui::Ui, kind: Safety) {
    safety_badge(ui, map_safety(kind));
    ui.add_space(S8);
}

fn map_safety(kind: Safety) -> SafetyKind {
    match kind {
        Safety::ReadOnly => SafetyKind::ReadOnly,
        Safety::Trash => SafetyKind::Trash,
        Safety::Admin => SafetyKind::Admin,
    }
}

fn page_header(ui: &mut egui::Ui, layout: UiLayout, screen: Screen, subtitle: &str) {
    let accent = accent_for_screen(screen);
    let safety_kind = match screen {
        Screen::Cleanup | Screen::Malware | Screen::Clutter => Safety::Trash,
        Screen::Protection | Screen::Apps | Screen::Cloud | Screen::Assistant => Safety::ReadOnly,
        _ => Safety::ReadOnly,
    };
    let _ = compact_page_header(
        ui,
        screen,
        layout.title_size(),
        subtitle,
        map_safety(safety_kind),
        accent,
        None,
    );
}

fn page_header_with_action(
    ui: &mut egui::Ui,
    layout: UiLayout,
    screen: Screen,
    subtitle: &str,
    action_label: &str,
    action_enabled: bool,
) -> bool {
    let accent = accent_for_screen(screen);
    let safety_kind = match screen {
        Screen::Cleanup | Screen::Malware | Screen::Clutter => Safety::Trash,
        _ => Safety::ReadOnly,
    };
    compact_page_header(
        ui,
        screen,
        layout.title_size(),
        subtitle,
        map_safety(safety_kind),
        accent,
        Some((action_label, action_enabled)),
    )
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(S4);
    label_small(ui, text);
}

fn label_small(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(CAPTION)
            .line_height(Some(lh(CAPTION)))
            .color(with_alpha(WHITE, 180))
            .strong(),
    );
}

fn step_label(ui: &mut egui::Ui, number: &str, text: &str) {
    ui.horizontal(|ui| {
        badge(ui, number, ACCENT);
        ui.label(RichText::new(text).font(heading_font(BODY)));
    });
}

fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::none()
        .fill(with_alpha(color, 48))
        .stroke(stroke(with_alpha(WHITE, 80)))
        .rounding(r_pill())
        .inner_margin(m_xy(S8, S4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(CAPTION).line_height(Some(lh(CAPTION))).color(WHITE));
        });
}

fn banner(ui: &mut egui::Ui, app: &mut ScanApp) {
    let Some(notice) = &app.notice else {
        return;
    };
    let accent = if notice.ok { SUCCESS } else { DANGER };
    let text = notice.text.clone();
    let mut dismiss = false;
    theme::notice_frame(accent).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(S24), Sense::hover());
                paint_ui_icon(
                    ui.painter(),
                    if notice.ok { UiIcon::Check } else { UiIcon::Alert },
                    icon_rect,
                    accent,
                );
                ui.add_space(S8);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(if notice.ok { "Done" } else { "That did not work" })
                            .font(heading_font(BODY))
                            .color(accent),
                    );
                    ui.label(RichText::new(&text).size(BODY).line_height(Some(lh(BODY))).color(TEXT));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost(ui, "Dismiss", true).clicked() {
                        dismiss = true;
                    }
                });
            });
        });
    ui.add_space(S8);
    if dismiss {
        app.notice = None;
    }
}

fn hero(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    theme::hero_card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui);
    });
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    components::surface_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui);
    });
    ui.add_space(S8);
}

fn tool_card(
    ui: &mut egui::Ui,
    heading: &str,
    body: &str,
    kind: Safety,
    action: &str,
    enabled: bool,
    on_click: &mut dyn FnMut(),
) {
    card(ui, |ui| {
        ui.label(RichText::new(heading).strong().size(BODY).line_height(Some(lh(BODY))));
        ui.label(RichText::new(body).color(TEXT_2).size(SMALL).line_height(Some(lh(SMALL))));
        safety(ui, kind);
        if ghost(ui, action, enabled).clicked() {
            on_click();
        }
    });
}

fn empty(ui: &mut egui::Ui, heading: &str, body: &str) {
    card(ui, |ui| {
        ui.add_space(S16);
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(S24), Sense::hover());
            icons::paint_ui_icon(ui.painter(), icons::UiIcon::Info, rect, TEXT_2);
            ui.add_space(S8);
            ui.label(RichText::new(heading).size(TITLE).strong().font(icons::heading_font(TITLE)));
            ui.add_space(S4);
            ui.label(RichText::new(body).size(BODY).line_height(Some(lh(BODY))).color(TEXT_2));
        });
        ui.add_space(S16);
    });
}

fn loading_line(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new(text).color(TEXT_2));
    });
}

/// Spinner while work is running, otherwise an empty-state card.
fn loading_card(ui: &mut egui::Ui, busy: bool, loading: &str, idle: &str) {
    if busy {
        card(ui, |ui| {
            ui.add_space(S12);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(S8);
                ui.label(RichText::new(loading).color(TEXT_2));
            });
            ui.add_space(S12);
        });
    } else if !idle.is_empty() {
        empty(ui, "Nothing here", idle);
    }
}

fn row(ui: &mut egui::Ui, selected: bool, clickable: bool, body: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let bg = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::none()
        .inner_margin(m_xy(S12, S8))
        .rounding(r_sm())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
    let mut response = inner.response;
    let hovered;
    if clickable {
        response = response.interact(Sense::click());
        hovered = response.hovered();
    } else {
        hovered = ui.rect_contains_pointer(response.rect);
    }
    let fill = if selected {
        with_alpha(WHITE, 48)
    } else if hovered {
        with_alpha(WHITE, 24)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().set(bg, Shape::rect_filled(response.rect, r_sm(), fill));
    if clickable && hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn avatar(ui: &mut egui::Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(S32), Sense::hover());
    ui.painter().rect_filled(rect, r_sm(), with_alpha(WHITE, 36));
    let letter = text
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, letter, font_semi(TITLE), color);
}

fn cloud_label(status: CloudStatus) -> &'static str {
    match status {
        CloudStatus::NotCloud => "On this Mac",
        CloudStatus::CloudLocal => "Cloud, stored locally",
        CloudStatus::CloudEvicted => "Cloud, not on this Mac",
    }
}

fn meter(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    let width = ui.available_width().clamp(48.0, 220.0);
    meter_with_width(ui, fraction, color, width);
}

fn meter_with_width(ui: &mut egui::Ui, fraction: f32, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width.max(S32), S4), Sense::hover());
    ui.painter().rect_filled(rect, r_xs(), with_alpha(WHITE, 40));
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction > 0.0 {
        let mut fill = rect;
        fill.set_width((rect.width() * fraction).max(S4));
        ui.painter().rect_filled(fill, r_xs(), color);
    }
}

fn gauge(fraction: f32) -> Color32 {
    if fraction > 0.85 {
        DANGER
    } else if fraction > 0.65 {
        WARN
    } else {
        SUCCESS
    }
}

fn stat(ui: &mut egui::Ui, label: &str, value: u64, value_color: Color32) {
    card(ui, |ui| {
        label_small(ui, &label.to_uppercase());
        ui.label(RichText::new(format_count(value)).size(STAT).strong().color(value_color));
    });
}

fn field(ui: &mut egui::Ui, text: &mut String, enabled: bool) {
    text_field(ui, text, "Path", enabled, true);
}

fn chip(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    chip_accent(ui, label, active, ACCENT_TEXT)
}

fn chip_accent(ui: &mut egui::Ui, label: &str, active: bool, accent: Color32) -> egui::Response {
    cmp_chip(ui, label, label, active, accent)
}

fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    primary_accent(ui, label, enabled, ACCENT_TEXT)
}

fn primary_accent(ui: &mut egui::Ui, label: &str, enabled: bool, accent: Color32) -> egui::Response {
    cmp_button(ui, label, label, BtnKind::Primary, enabled, accent)
}

fn ghost(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ghost_accent(ui, label, enabled, ACCENT_TEXT)
}

fn ghost_accent(ui: &mut egui::Ui, label: &str, enabled: bool, accent: Color32) -> egui::Response {
    cmp_button(ui, label, label, BtnKind::Ghost, enabled, accent)
}

fn destructive(ui: &mut egui::Ui, label: &str, enabled: bool, accent: Color32) -> egui::Response {
    cmp_button(ui, label, label, BtnKind::Destructive, enabled, accent)
}

// ───────────────────────────── helpers ─────────────────────────────

fn describe_junk(label: &str) -> &'static str {
    match label {
        "User caches" => "Temporary files apps keep so they open faster. Rebuilt automatically.",
        "User logs" => "Activity notes apps write for troubleshooting. Not needed day to day.",
        "Diagnostic reports" => "Crash reports macOS saves after an app quits unexpectedly.",
        "Spotify cache" => "Songs Spotify stored for faster playback. It downloads them again if needed.",
        "Adobe media cache" => "Preview files from Adobe video and audio apps. Rebuilt on demand.",
        "Xcode derived data" => "Build files from Xcode. The next build recreates them.",
        _ => "Temporary files that apps can recreate.",
    }
}

fn outcome(result: Result<String, String>, fallback: &str) -> Notice {
    match result {
        Ok(text) => Notice {
            ok: true,
            text: if text.trim().is_empty() { fallback.to_string() } else { text.trim().to_string() },
            refresh: None,
        },
        Err(text) => Notice { ok: false, text, refresh: None },
    }
}

fn reveal(path: &Path) {
    let _ = Command::new("open").arg("-R").arg(path).spawn();
}

fn open_url(url: &str) {
    let _ = Command::new("open").arg(url).spawn();
}

fn normalize_space_path(path: &str) -> String {
    let trimmed = path.trim();
    let candidate = PathBuf::from(trimmed);
    std::fs::canonicalize(&candidate)
        .map(|resolved| resolved.display().to_string())
        .unwrap_or_else(|_| trimmed.to_string())
}

fn volume_title(path: &str) -> String {
    let place = path_place(path);
    let mac = path_place(&computer_root().display().to_string());
    if place == mac || place == "/" {
        "Macintosh HD".into()
    } else {
        file_name(Path::new(path))
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("item")
        .to_string()
}

fn pretty(home: &Path, path: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

fn shorten(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max || max < 8 {
        return text.to_string();
    }
    let keep = max - 3;
    let head = keep / 2;
    let tail = keep - head;
    let start: String = chars[..head].iter().collect();
    let end: String = chars[chars.len() - tail..].iter().collect();
    format!("{start}...{end}")
}

fn format_count(n: u64) -> String {
    let raw = n.to_string();
    let mut out = String::new();
    for (i, ch) in raw.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn format_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
