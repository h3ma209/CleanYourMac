use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, FontFamily, FontId, Layout, Margin, RichText, Rect, Rounding, Sense, Shape, Stroke,
    TextStyle, Vec2,
};
use engine::{
    flush_dns_cache, folder_access, junk_roots, large_downloads, launch_agents, local_cloud_folders,
    machine_sample, move_download_to_trash, move_root_to_trash, reindex_spotlight, run_care, run_scan,
    scan_space, spotlight_status, survey_leftovers, trash_selected, prepare_and_scan, remember_ignore, trash_threat, computer_root, is_spared, AccessStatus, CareReport, CloudFolder, CloudStatus, Detection, ThreatReport,
    SPACE_LENS_BUBBLE_THRESHOLD, SpaceNode,
    ExecutionControl, ExtensionWatchDetector, InstalledApp, JunkRoot, LargeFile, LaunchAgent, LeftoverHit,
    MachineSample, ScanConfig, ScanEvent, SpaceReport,
};
use tokio::sync::mpsc;

mod icons;
mod theme;

use icons::{heading_font, icon_with_label, paint_brand_mark, paint_screen_icon, paint_ui_icon, UiIcon};

// Spotify-inspired dark UI (#121212 base, #1DB954 accent).
const BG: Color32 = Color32::from_rgb(18, 18, 18);
const SIDEBAR: Color32 = Color32::from_rgb(0, 0, 0);
const ELEVATED: Color32 = Color32::from_rgb(24, 24, 24);
const CARD: Color32 = Color32::from_rgb(40, 40, 40);
const CARD_HOVER: Color32 = Color32::from_rgb(51, 51, 51);
const LINE: Color32 = Color32::from_rgb(42, 42, 42);
const TEXT: Color32 = Color32::from_rgb(255, 255, 255);
const SUBTEXT: Color32 = Color32::from_rgb(179, 179, 179);
const GREEN: Color32 = Color32::from_rgb(29, 185, 84);
const GREEN_ON: Color32 = Color32::from_rgb(255, 255, 255);
const AMBER: Color32 = Color32::from_rgb(240, 180, 41);
const ROSE: Color32 = Color32::from_rgb(232, 112, 106);
const SKY: Color32 = Color32::from_rgb(120, 190, 240);

const IVORY: Color32 = TEXT;
const MUTED: Color32 = SUBTEXT;
const LIME: Color32 = GREEN;
const LIME_INK: Color32 = GREEN_ON;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Screen {
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
    const ALL: [Screen; 10] = [
        Self::Care,
        Self::Cleanup,
        Self::Apps,
        Self::Performance,
        Self::Protection,
        Self::Malware,
        Self::Clutter,
        Self::Space,
        Self::Cloud,
        Self::Assistant,
    ];

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
    fn auto_loads(self) -> bool {
        !matches!(self, Self::Protection | Self::Malware | Self::Space)
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
    content_max: f32,
}

impl UiLayout {
    fn from_viewport(w: f32) -> Self {
        let compact = w < 880.0;
        let narrow = w < 1100.0;
        let sidebar_w = if compact { 76.0 } else if narrow { 228.0 } else { 268.0 };
        let pad_x = if compact { 14.0 } else if narrow { 20.0 } else { 32.0 };
        let pad_y = if compact { 12.0 } else { 22.0 };
        let content_max = (w - sidebar_w - pad_x * 2.0).clamp(280.0, 980.0);
        Self {
            sidebar_w,
            compact,
            narrow,
            pad_x,
            pad_y,
            content_max,
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
        if self.compact {
            22.0
        } else if self.narrow {
            28.0
        } else {
            34.0
        }
    }

    fn subtitle_size(self) -> f32 {
        if self.compact { 13.0 } else { 14.5 }
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
    Space(SpaceReport),
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
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([720.0, 520.0]),
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
    theme::polish_style(&mut style, GREEN, TEXT, CARD);
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
    space_selected: HashSet<PathBuf>,
    space_hover: Option<PathBuf>,
    threat_root: String,
    threat: Option<ThreatReport>,
    clutter: Vec<LargeFile>,
    cloud: Vec<CloudFolder>,
    access: AccessStatus,
    events: Option<mpsc::Receiver<UiMsg>>,
    control: Option<Arc<ExecutionControl>>,
    busy: bool,
    layout: UiLayout,
}

impl ScanApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        let threat_root = home.join("Downloads").display().to_string();
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
            space_root: computer_root().display().to_string(),
            space: None,
            space_selected: HashSet::new(),
            space_hover: None,
            threat_root,
            threat: None,
            clutter: Vec::new(),
            cloud: Vec::new(),
            access,
            events: None,
            control: None,
            busy: false,
            layout: UiLayout::from_viewport(1180.0),
        }
    }

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.notice = None;
    }

    fn arm(&mut self) -> mpsc::Sender<UiMsg> {
        let (tx, rx) = mpsc::channel(256);
        self.events = Some(rx);
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
        let root = self.space.as_ref().map(|report| report.root.clone()).unwrap_or_default();
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

    fn start_space(&mut self) {
        if self.busy {
            return;
        }
        let trimmed = self.space_root.trim();
        if trimmed.is_empty() {
            self.notice = Some(Notice {
                ok: false,
                text: "Choose a location first — try “This Mac” or Home.".into(),
                refresh: None,
            });
            return;
        }
        let root = PathBuf::from(trimmed);
        if !root.exists() {
            self.notice = Some(Notice {
                ok: false,
                text: format!("That folder does not exist: {}", root.display()),
                refresh: None,
            });
            return;
        }
        self.space = None;
        self.space_selected.clear();
        self.space_hover = None;
        self.access = folder_access(Path::new("/Library/Application Support"));
        if let Some(message) = self.space_scan_blocked_message() {
            self.notice = Some(Notice {
                ok: false,
                text: message,
                refresh: None,
            });
            return;
        }
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let report = scan_space(root).await;
            let _ = tx.send(UiMsg::Space(report)).await;
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
        self.busy = false;
        self.status = "idle";
        self.control = None;
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
                    if let Some(screen) = notice.refresh {
                        self.loaded.remove(&screen);
                        self.loaded.remove(&Screen::Care);
                    }
                    self.notice = Some(notice);
                    self.finish();
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
                UiMsg::Space(report) => {
                    self.space_root = report.root.display().to_string();
                    self.space = Some(report);
                    self.space_selected.clear();
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
                ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(170));
                ui.allocate_rect(rect, Sense::click());
            });

        let mut choice: Option<bool> = None;
        egui::Window::new("confirm")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(theme::glass_card().inner_margin(Margin::same(26.0)))
            .show(ctx, |ui| {
                ui.set_width(dialog_w);
                icons::icon_with_label(ui, icons::UiIcon::Alert, &heading, TEXT);
                ui.add_space(4.0);
                ui.add_space(6.0);
                ui.label(RichText::new(body).color(MUTED));
                ui.add_space(18.0);
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

        egui::SidePanel::left("nav")
            .exact_width(layout.sidebar_w)
            .resizable(false)
            .frame(
                theme::glass_sidebar().inner_margin(Margin::symmetric(if layout.compact { 10.0 } else { 16.0 }, 18.0)),
            )
            .show(ctx, |ui| {
                sidebar_brand(ui, layout);
                ui.add_space(if layout.compact { 14.0 } else { 20.0 });
                label_small(ui, if layout.compact { "MENU" } else { "TOOLS" });
                ui.add_space(6.0);
                for item in Screen::ALL {
                    let response = sidebar_nav_item(ui, item, item == screen, layout);
                    if response.clicked() {
                        self.go(item);
                    }
                    ui.add_space(2.0);
                }
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    status_pill(ui, status, busy, pulse);
                    ui.add_space(8.0);
                    let granted = self.access == AccessStatus::Granted;
                    if layout.compact {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                        ui.painter().circle_filled(
                            rect.center(),
                            4.5,
                            if granted { GREEN } else { AMBER },
                        );
                    } else {
                        ui.label(
                            RichText::new(if granted {
                                "Full disk access"
                            } else {
                                "Limited disk access"
                            })
                            .size(12.0)
                            .color(if granted { GREEN } else { AMBER }),
                        );
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(layout.pad_x, layout.pad_y)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(format!("main_{}", screen.label()))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(layout.content_max);
                        ui.set_min_width(ui.available_width());
                        match self.screen {
                            Screen::Care => care_screen(ui, self),
                            Screen::Cleanup => cleanup_screen(ui, self),
                            Screen::Apps => apps_screen(ui, self),
                            Screen::Performance => performance_screen(ui, self),
                            Screen::Protection => protection_screen(ui, self),
                            Screen::Malware => malware_screen(ui, self),
                            Screen::Clutter => clutter_screen(ui, self),
                            Screen::Space => space_screen(ui, self),
                            Screen::Cloud => cloud_screen(ui, self),
                            Screen::Assistant => assistant_screen(ui, self),
                        }
                    });
            });

        self.confirm_modal(ctx);
    }
}

// ───────────────────────────── sidebar chrome ─────────────────────────────

fn sidebar_brand(ui: &mut egui::Ui, layout: UiLayout) {
    if layout.compact {
        ui.vertical_centered(|ui| {
            let (mark_rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
            paint_brand_mark(ui.painter(), mark_rect, GREEN);
            response.on_hover_text("Sable — Mac cleaner and tune-up");
        });
        return;
    }
    ui.horizontal(|ui| {
        let (mark_rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        paint_brand_mark(ui.painter(), mark_rect, GREEN);
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(RichText::new("Sable").font(heading_font(18.0)).color(TEXT));
            ui.label(RichText::new("For your Mac").size(11.5).color(SUBTEXT));
        });
    });
}

fn sidebar_nav_item(ui: &mut egui::Ui, screen: Screen, active: bool, layout: UiLayout) -> egui::Response {
    let icon_color = if active { GREEN } else { SUBTEXT };
    let title_color = if active { TEXT } else { SUBTEXT };
    let bg = if active {
        Color32::from_rgba_unmultiplied(255, 255, 255, 18)
    } else {
        Color32::TRANSPARENT
    };

    let inner = egui::Frame::none()
        .fill(bg)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin {
            left: if active { 14.0 } else { 10.0 },
            right: if layout.compact { 6.0 } else { 10.0 },
            top: 8.0,
            bottom: 8.0,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if layout.compact {
                ui.vertical_centered(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
                    paint_screen_icon(ui.painter(), screen, icon_rect, icon_color);
                });
            } else {
                ui.horizontal(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
                    paint_screen_icon(ui.painter(), screen, icon_rect, icon_color);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(RichText::new(screen.label()).font(heading_font(14.0)).color(title_color));
                        ui.label(RichText::new(screen.nav_hint()).size(11.5).color(SUBTEXT));
                    });
                });
            }
        });

    let mut response = inner.response.interact(Sense::click());
    if active {
        let bar = Rect::from_min_size(
            response.rect.left_top() - Vec2::new(4.0, 0.0),
            Vec2::new(3.0, response.rect.height()),
        );
        ui.painter().rect_filled(bar, Rounding::same(2.0), GREEN);
    }
    if layout.compact {
        response = response.on_hover_text(format!("{}\n{}", screen.label(), screen.nav_hint()));
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

// ───────────────────────────── screens ─────────────────────────────

fn care_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Care,
        "A quick health check of your Mac. This page only looks. It never changes or deletes anything.",
    );
    safety(ui, Safety::ReadOnly);
    banner(ui, app);

    let mut goto = None;
    let mut rerun = false;
    let busy = app.busy;

    let reclaim = if app.junk.is_empty() {
        app.care.as_ref().map(|r| r.cleanup_bytes)
    } else {
        Some(app.junk.iter().map(|r| r.bytes).sum())
    };

    let layout = app.layout;
    hero(ui, |ui| match reclaim {
        Some(bytes) => {
            let size = if layout.compact { 30.0 } else { 44.0 };
            if layout.compact {
                label_small(ui, "YOU CAN FREE UP");
                ui.label(RichText::new(format_bytes(bytes)).size(size).strong().color(GREEN));
                ui.label(
                    RichText::new("of temporary files (caches and logs). Apps rebuild what they need.")
                        .color(SUBTEXT),
                );
                ui.add_space(8.0);
                if primary(ui, "Review and clean up", true).clicked() {
                    goto = Some(Screen::Cleanup);
                }
            } else {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        label_small(ui, "YOU CAN FREE UP");
                        ui.label(RichText::new(format_bytes(bytes)).size(size).strong().color(GREEN));
                        ui.label(
                            RichText::new("of temporary files (caches and logs). Apps rebuild what they need.")
                                .color(SUBTEXT),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary(ui, "Review and clean up", true).clicked() {
                            goto = Some(Screen::Cleanup);
                        }
                    });
                });
            }
        }
        None => loading_line(ui, "Checking your Mac..."),
    });
    ui.add_space(14.0);

    if let Some(report) = &app.care {
        let (apps, agents, access) = (report.app_count, report.agent_count, report.access);
        let granted = access == AccessStatus::Granted;
        match layout.grid_cols(3) {
            1 => {
                summary_card(
                    ui,
                    "APPS",
                    &format_count(apps as u64),
                    "apps installed",
                    "Browse apps",
                    busy,
                    &mut || goto = Some(Screen::Apps),
                );
                summary_card(
                    ui,
                    "STARTUP ITEMS",
                    &format_count(agents as u64),
                    "background items start when you log in",
                    "See startup items",
                    busy,
                    &mut || goto = Some(Screen::Performance),
                );
                card(ui, |ui| disk_access_card(ui, granted));
            }
            2 => {
                ui.columns(2, |cols| {
                    summary_card(
                        &mut cols[0],
                        "APPS",
                        &format_count(apps as u64),
                        "apps installed",
                        "Browse apps",
                        busy,
                        &mut || goto = Some(Screen::Apps),
                    );
                    summary_card(
                        &mut cols[1],
                        "STARTUP ITEMS",
                        &format_count(agents as u64),
                        "background items start when you log in",
                        "See startup items",
                        busy,
                        &mut || goto = Some(Screen::Performance),
                    );
                });
                card(ui, |ui| disk_access_card(ui, granted));
            }
            _ => {
                ui.columns(3, |cols| {
                    summary_card(
                        &mut cols[0],
                        "APPS",
                        &format_count(apps as u64),
                        "apps installed",
                        "Browse apps",
                        busy,
                        &mut || goto = Some(Screen::Apps),
                    );
                    summary_card(
                        &mut cols[1],
                        "STARTUP ITEMS",
                        &format_count(agents as u64),
                        "background items start when you log in",
                        "See startup items",
                        busy,
                        &mut || goto = Some(Screen::Performance),
                    );
                    card(&mut cols[2], |ui| disk_access_card(ui, granted));
                });
            }
        }
        ui.add_space(8.0);
    }

    ui.add_space(6.0);
    section(ui, "WHERE TO GO NEXT");
    for (screen, hint) in [
        (Screen::Clutter, "Find large files in Downloads you may no longer need."),
        (Screen::Space, "See which folders are using the most disk space."),
        (Screen::Apps, "Check what an app leaves behind in your Library."),
    ] {
        let r = row(ui, false, true, |ui| {
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                paint_screen_icon(ui.painter(), screen, icon_rect, GREEN);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(screen.label()).strong().color(TEXT));
                    ui.label(RichText::new(hint).color(SUBTEXT).size(13.0));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("Open").color(GREEN));
                });
            });
        });
        if r.clicked() {
            goto = Some(screen);
        }
    }

    ui.add_space(10.0);
    if ghost(ui, "Run the check again", !busy).clicked() {
        rerun = true;
    }

    if rerun {
        app.loaded.remove(&Screen::Care);
        app.loaded.remove(&Screen::Cleanup);
    }
    if let Some(screen) = goto {
        app.go(screen);
    }
}

fn cleanup_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Cleanup,
        "Apps keep temporary files (caches and logs) to run faster. They are safe to remove, and apps recreate what they need. Review each group, then move it to the Trash.",
    );
    safety(ui, Safety::Trash);
    banner(ui, app);

    let busy = app.busy;
    if app.junk.is_empty() {
        loading_card(ui, busy, "Measuring caches and logs...", "Nothing to show yet.");
        return;
    }

    let total: u64 = app.junk.iter().map(|r| r.bytes).sum();
    let biggest = app.junk.iter().map(|r| r.bytes).max().unwrap_or(1).max(1);
    hero(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                label_small(ui, "TOTAL YOU CAN CLEAN");
                ui.label(RichText::new(format_bytes(total)).size(38.0).strong().color(LIME));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ghost(ui, "Measure again", !busy).clicked() {
                    app.loaded.remove(&Screen::Cleanup);
                }
            });
        });
    });
    ui.add_space(14.0);
    section(ui, "WHAT TAKES THE SPACE");

    let mut request = None;
    let mut reveal_path = None;
    for root in &app.junk {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&root.label).strong().size(15.5));
                    ui.label(RichText::new(describe_junk(&root.label)).color(MUTED).size(13.0));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if root.exists {
                        if ghost(ui, "Move to Trash", !busy && root.bytes > 0).clicked() {
                            request = Some(Confirm {
                                heading: format!("Move {} to the Trash?", root.label.to_lowercase()),
                                body: format!(
                                    "{} will be moved to the Trash. Apps recreate these files when needed, and you can put them back from the Trash.",
                                    format_bytes(root.bytes)
                                ),
                                action: "Move to Trash",
                                kind: ConfirmKind::TrashRoot(root.path.clone()),
                            });
                        }
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(root.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(root.bytes)).strong().size(16.0).color(LIME));
                    } else {
                        ui.label(RichText::new("Not on this Mac").color(MUTED));
                    }
                });
            });
            if root.exists {
                ui.add_space(4.0);
                meter(ui, root.bytes as f32 / biggest as f32, LIME);
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
    page_header(
        ui,
        app.layout,
        Screen::Apps,
        "Everything installed on this Mac. Pick an app to see the files it keeps in your Library, which stay behind even after you drag the app to the Trash.",
    );
    safety(ui, Safety::ReadOnly);
    banner(ui, app);

    let busy = app.busy;
    if app.apps.is_empty() {
        loading_card(ui, busy, "Looking for installed apps...", "No apps found in /Applications or ~/Applications.");
        return;
    }

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.app_filter)
                .hint_text("Search by app name")
                .desired_width(320.0)
                .margin(Vec2::new(10.0, 8.0)),
        );
        ui.label(RichText::new(format!("{} apps", app.apps.len())).color(MUTED));
    });
    ui.add_space(6.0);

    let needle = app.app_filter.trim().to_lowercase();
    let mut pick = None;
    card(ui, |ui| {
        egui::ScrollArea::vertical().max_height(290.0).id_salt("apps_list").show(ui, |ui| {
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
                        avatar(ui, &item.name, LIME);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(RichText::new(&item.name).strong());
                            ui.label(RichText::new(&item.bundle_id).size(12.0).color(MUTED));
                        });
                    });
                });
                if r.clicked() && !busy {
                    pick = Some(index);
                }
            }
            if shown == 0 {
                ui.label(RichText::new("No app matches your search.").color(MUTED));
            }
        });
    });
    if let Some(index) = pick {
        app.selected_app = Some(index);
        app.leftovers.clear();
        app.leftovers_done = false;
        app.survey_selected();
    }

    ui.add_space(12.0);
    match app.selected_app.and_then(|i| app.apps.get(i)) {
        None => empty(ui, "Pick an app above", "Its leftover files will appear here. Nothing is deleted from this screen."),
        Some(selected) => {
            let name = selected.name.clone();
            let app_path = selected.path.clone();
            section(ui, &format!("FILES {} KEEPS IN YOUR LIBRARY", name.to_uppercase()));
            if !app.leftovers_done {
                loading_card(ui, true, "Searching your Library...", "");
            } else if app.leftovers.is_empty() {
                empty(ui, "No leftover files found", "This app does not keep anything in the usual Library folders.");
            } else {
                let mut reveal_path = None;
                card(ui, |ui| {
                    for hit in &app.leftovers {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 1.0;
                                ui.label(RichText::new(file_name(&hit.path)).strong());
                                ui.label(
                                    RichText::new(format!("{}  ·  {}", hit.root, shorten(&pretty(&app.home, &hit.path), 70)))
                                        .size(12.0)
                                        .color(MUTED),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ghost(ui, "Show", true).clicked() {
                                    reveal_path = Some(hit.path.clone());
                                }
                            });
                        });
                    }
                });
                if let Some(path) = reveal_path {
                    reveal(&path);
                }
            }
            ui.add_space(6.0);
            if ghost(ui, "Show the app in Finder", true).clicked() {
                reveal(&app_path);
            }
        }
    }
}

fn performance_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Performance,
        "See which apps start automatically in the background, and run a few common macOS fixes.",
    );
    banner(ui, app);
    let busy = app.busy;

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

    ui.add_space(14.0);
    section(ui, "APPS THAT START AUTOMATICALLY");
    ui.label(
        RichText::new("These background items launch when you log in. Sable lists them but never turns them off.")
            .color(MUTED)
            .size(13.0),
    );
    ui.add_space(6.0);
    if app.agents.is_empty() {
        loading_card(ui, busy, "Reading startup items...", "No startup items found for your account.");
    } else {
        card(ui, |ui| {
            for agent in &app.agents {
                let yours = agent.path.starts_with(&app.home);
                let r = row(ui, false, false, |ui| {
                    ui.horizontal(|ui| {
                        avatar(ui, &agent.label, if yours { LIME } else { SKY });
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(RichText::new(shorten(&agent.label, 60)).strong());
                            ui.label(
                                RichText::new(if yours { "Starts for your account only" } else { "Starts for every user on this Mac" })
                                    .size(12.0)
                                    .color(MUTED),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ghost(ui, "Show", true).clicked() {
                                reveal(&agent.path);
                            }
                        });
                    });
                });
                let _ = r;
            }
        });
    }
}

fn protection_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Protection,
        "Looks through a folder and lists files of the types you choose, like old installers or archives. It does not check for viruses and never changes your files.",
    );
    safety(ui, Safety::ReadOnly);
    banner(ui, app);

    let busy = app.busy;
    if app.access != AccessStatus::Granted {
        card(ui, |ui| {
            ui.label(RichText::new("Sable has limited disk access").color(AMBER).strong());
            ui.label(
                RichText::new("Some folders cannot be read and will be skipped. Grant Full Disk Access to include them.")
                    .color(MUTED)
                    .size(13.0),
            );
            if ghost(ui, "Open Privacy settings", true).clicked() {
                open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
            }
        });
        ui.add_space(8.0);
    }

    card(ui, |ui| {
        step_label(ui, "1", "Where should it look?");
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            for (name, sub) in [("Downloads", "Downloads"), ("Documents", "Documents"), ("Desktop", "Desktop"), ("Home folder", "")] {
                let target = if sub.is_empty() { home.clone() } else { home.join(sub) }.display().to_string();
                if chip(ui, name, app.root == target).clicked() && !busy {
                    app.root = target;
                }
            }
        });
        field(ui, &mut app.root, !busy);

        ui.add_space(10.0);
        step_label(ui, "2", "Which kinds of files?");
        ui.horizontal_wrapped(|ui| {
            for (name, exts) in [
                ("Installers", "dmg,pkg"),
                ("Archives", "zip,rar,7z,tar,gz"),
                ("Disk images", "iso,img,dmg"),
                ("Videos", "mov,mp4,mkv,avi"),
                ("Documents", "pdf,doc,docx,pages"),
            ] {
                if chip(ui, name, app.watch == exts).clicked() && !busy {
                    app.watch = exts.into();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Extensions:").color(MUTED).size(13.0));
            ui.add(
                egui::TextEdit::singleline(&mut app.watch)
                    .desired_width(300.0)
                    .interactive(!busy)
                    .hint_text("dmg, pkg, zip")
                    .margin(Vec2::new(10.0, 8.0))
                    .font(TextStyle::Monospace),
            );
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if primary(ui, "Start searching", !busy).clicked() {
                app.start_scan();
            }
            let paused = app.control.as_ref().is_some_and(|c| c.is_paused());
            if ghost(ui, if paused { "Resume" } else { "Pause" }, busy).clicked() {
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
            if ghost(ui, "Stop", busy).clicked() {
                if let Some(control) = &app.control {
                    control.cancel();
                    app.status = "canceling";
                }
            }
        });
    });
    ui.add_space(12.0);

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
                ui.label(RichText::new(shorten(&app.last_path, 90)).monospace().size(12.0).color(MUTED));
            }
        });
        ui.add_space(8.0);
    } else if app.scan_ran {
        match app.layout.grid_cols(3) {
            1 => {
                stat(ui, "Files checked", app.scanned, TEXT);
                stat(ui, "Matches", app.detection_count, GREEN);
                stat(ui, "Could not read", app.errors, if app.errors > 0 { AMBER } else { TEXT });
            }
            2 => {
                ui.columns(2, |cols| {
                    stat(&mut cols[0], "Files checked", app.scanned, TEXT);
                    stat(&mut cols[1], "Matches", app.detection_count, GREEN);
                });
                stat(ui, "Could not read", app.errors, if app.errors > 0 { AMBER } else { TEXT });
            }
            _ => {
                ui.columns(3, |cols| {
                    stat(&mut cols[0], "Files checked", app.scanned, TEXT);
                    stat(&mut cols[1], "Matches", app.detection_count, GREEN);
                    stat(&mut cols[2], "Could not read", app.errors, if app.errors > 0 { AMBER } else { TEXT });
                });
            }
        }
        ui.add_space(8.0);
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
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(RichText::new(file_name(&hit.path)).strong());
                        ui.label(
                            RichText::new(shorten(&pretty(&app.home, hit.path.parent().unwrap_or(Path::new(""))), 70))
                                .size(12.0)
                                .color(MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(hit.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(hit.size)).color(LIME).strong());
                        badge(ui, &format!(".{}", hit.extension), SKY);
                    });
                });
            }
            if app.detections.len() > 300 {
                ui.label(RichText::new(format!("and {} more", app.detections.len() - 300)).color(MUTED));
            }
        });
        if let Some(path) = reveal_path {
            reveal(&path);
        }
    }
}

fn malware_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Malware,
        "Checks a folder against a public list of malware file hashes from abuse.ch MalwareBazaar. A match is moved to the Trash only after you confirm.",
    );
    safety(ui, Safety::Trash);
    banner(ui, app);

    card(ui, |ui| {
        ui.label(RichText::new("What this scan does not do").strong());
        ui.label(
            RichText::new("It does not use CleanMyMac's signature database. It does not read browser cookies, history, passwords, or messages. It does not change camera, microphone, or other app permissions. It does not watch the network in the background.")
                .color(MUTED)
                .size(13.0),
        );
        if ghost(ui, "Open Privacy settings", true).clicked() {
            open_url("x-apple.systempreferences:com.apple.preference.security?Privacy");
        }
    });
    ui.add_space(8.0);

    let busy = app.busy;
    card(ui, |ui| {
        step_label(ui, "1", "Where should it look?");
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            for (name, sub) in [
                ("Downloads", "Downloads"),
                ("Desktop", "Desktop"),
                ("Documents", "Documents"),
                ("Launch Agents", "Library/LaunchAgents"),
            ] {
                let target = home.join(sub).display().to_string();
                if chip(ui, name, app.threat_root == target).clicked() && !busy {
                    app.threat_root = target;
                }
            }
        });
        field(ui, &mut app.threat_root, !busy);
        ui.add_space(8.0);
        ui.label(
            RichText::new("The first scan downloads the public hash list if the saved copy is missing or older than a day. Later scans reuse that copy.")
                .color(MUTED)
                .size(13.0),
        );
        ui.add_space(8.0);
        if primary(ui, "Scan", !busy).clicked() {
            app.start_threat();
        }
    });
    ui.add_space(12.0);

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

    stat(ui, "Files hashed", report.scanned, TEXT);
    ui.add_space(8.0);
    card(ui, |ui| {
        ui.label(RichText::new(&report.signatures_note).color(MUTED).size(13.0));
        if report.skipped_large > 0 || report.unreadable > 0 {
            ui.label(
                RichText::new(format!(
                    "{} files were larger than 50 MB and were skipped. {} items could not be read.",
                    format_count(report.skipped_large),
                    format_count(report.unreadable)
                ))
                .color(MUTED)
                .size(13.0),
            );
        }
    });
    ui.add_space(8.0);

    if report.hits.is_empty() {
        empty(ui, "No known malware hashes", "Nothing in that folder matched the public list.");
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
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(file_name(&hit.path)).strong());
                    ui.label(
                        RichText::new(shorten(&hit.path.display().to_string(), 72))
                            .size(12.0)
                            .color(MUTED),
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
                    badge(ui, "hash match", ROSE);
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
    page_header(
        ui,
        app.layout,
        Screen::Clutter,
        "Large files (50 MB or more) sitting in your Downloads folder. Old installers and videos pile up here. Review them one at a time.",
    );
    safety(ui, Safety::Trash);
    banner(ui, app);

    let busy = app.busy;
    if app.clutter.is_empty() {
        if busy {
            loading_card(ui, true, "Looking through Downloads...", "");
        } else {
            empty(ui, "Your Downloads folder is tidy", "No file there is 50 MB or larger.");
            if ghost(ui, "Look again", true).clicked() {
                app.loaded.remove(&Screen::Clutter);
            }
        }
        return;
    }

    let total: u64 = app.clutter.iter().map(|f| f.bytes).sum();
    hero(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                label_small(ui, "LARGE FILES FOUND");
                ui.label(RichText::new(format_bytes(total)).size(38.0).strong().color(LIME));
                ui.label(RichText::new(format!("across {} files", app.clutter.len())).color(MUTED));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ghost(ui, "Look again", !busy).clicked() {
                    app.loaded.remove(&Screen::Clutter);
                }
            });
        });
    });
    ui.add_space(14.0);

    let mut request = None;
    let mut reveal_path = None;
    let biggest = app.clutter.iter().map(|f| f.bytes).max().unwrap_or(1).max(1);
    card(ui, |ui| {
        for (index, file) in app.clutter.iter().enumerate() {
            ui.push_id(index, |ui| {
                row(ui, false, false, |ui| {
                    ui.horizontal(|ui| {
                        let bar_w = (ui.available_width() * 0.55).clamp(160.0, 420.0);
                        ui.vertical(|ui| {
                            ui.set_max_width(bar_w);
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(RichText::new(shorten(&file_name(&file.path), 55)).strong());
                            ui.label(
                                RichText::new(shorten(
                                    &pretty(&app.home, file.path.parent().unwrap_or(Path::new(""))),
                                    70,
                                ))
                                .size(12.0)
                                .color(MUTED),
                            );
                            meter_with_width(ui, file.bytes as f32 / biggest as f32, GREEN, bar_w);
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(format_bytes(file.bytes)).strong().color(GREEN));
                            ui.add_space(8.0);
                            if ghost(ui, "Show", true).clicked() {
                                reveal_path = Some(file.path.clone());
                            }
                            if ghost(ui, "Move to Trash", !busy).clicked() {
                                request = Some(Confirm {
                                    heading: format!(
                                        "Move \"{}\" to the Trash?",
                                        shorten(&file_name(&file.path), 40)
                                    ),
                                    body: format!(
                                        "This frees {}. You can put it back from the Trash if you change your mind.",
                                        format_bytes(file.bytes)
                                    ),
                                    action: "Move to Trash",
                                    kind: ConfirmKind::TrashDownload(file.path.clone()),
                                });
                            }
                        });
                    });
                });
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
    if app.access == AccessStatus::Granted {
        return;
    }
    if app.space_scan_blocked_message().is_none() {
        return;
    }
    card(ui, |ui| {
        ui.label(RichText::new("Full Disk Access needed").color(AMBER).strong());
        ui.label(
            RichText::new(
                "To scan the whole Mac, allow Full Disk Access for Sable in System Settings. Without it, scanning stops here.",
            )
            .color(SUBTEXT)
            .size(13.0),
        );
        if ghost(ui, "Open Privacy settings", true).clicked() {
            open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
        }
    });
    ui.add_space(8.0);
}

fn space_lens_hover_card(ctx: &egui::Context, home: &Path, hover: &SpaceNode) {
    let pointer = ctx.input(|i| i.pointer.hover_pos()).unwrap_or_else(|| ctx.screen_rect().center());
    egui::Area::new(egui::Id::new("space_lens_hover"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pointer + Vec2::new(14.0, 14.0))
        .show(ctx, |ui| {
            egui::Frame::popup(&ctx.style()).show(ui, |ui| {
                ui.set_max_width(360.0);
                ui.label(RichText::new(file_name(&hover.path)).strong());
                ui.label(RichText::new(pretty(home, &hover.path)).size(12.0).color(SUBTEXT).monospace());
                ui.separator();
                ui.label(RichText::new(format!("Type: {}", space_item_kind(hover))).size(13.0));
                ui.label(RichText::new(format!("Size: {}", format_bytes(hover.bytes))).size(13.0));
                ui.label(RichText::new(format_modified(hover.modified)).size(13.0).color(SUBTEXT));
                ui.label(RichText::new(cloud_label(hover.cloud)).size(12.0).color(SUBTEXT));
                if hover.is_directory {
                    ui.label(RichText::new("Double-click to zoom into this folder").size(12.0).color(GREEN));
                }
            });
        });
}

fn space_lens_selection_registry(
    ui: &mut egui::Ui,
    app: &mut ScanApp,
    index: &std::collections::HashMap<PathBuf, u64>,
    busy: bool,
) {
    let removable: Vec<_> = app
        .space_selected
        .iter()
        .filter(|path| !is_spared(path, &app.home))
        .cloned()
        .collect();
    let selected_bytes: u64 = app
        .space_selected
        .iter()
        .map(|path| index.get(path).copied().unwrap_or(0))
        .sum();
    ui.push_id("space_selection_registry", |ui| {
    card(ui, |ui| {
        label_small(ui, "SELECTION");
        ui.label(RichText::new(format_bytes(selected_bytes)).size(26.0).strong().color(GREEN));
        ui.label(
            RichText::new(format!("{} item(s) marked", app.space_selected.len()))
                .color(SUBTEXT)
                .size(13.0),
        );
        ui.add_space(6.0);
        if app.space_selected.is_empty() {
            ui.label(RichText::new("Check files in the list to add them here.").color(SUBTEXT).size(12.5));
        } else {
            let selected: Vec<_> = app.space_selected.iter().cloned().collect();
            let mut remove = None;
            egui::ScrollArea::vertical()
                .id_salt("space_selection_scroll")
                .max_height(220.0)
                .show(ui, |ui| {
                for path in &selected {
                    let spared = is_spared(path, &app.home);
                    ui.push_id(path.display().to_string(), |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(RichText::new(shorten(&file_name(path), 28)).size(12.5));
                            ui.label(
                                RichText::new(format!(
                                    "{}{}",
                                    format_bytes(index.get(path).copied().unwrap_or(0)),
                                    if spared { " · protected" } else { "" }
                                ))
                                .size(11.0)
                                .color(if spared { AMBER } else { SUBTEXT }),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ghost(ui, "×", !busy).clicked() {
                                remove = Some(path.clone());
                            }
                        });
                    });
                    });
                }
            });
            if let Some(path) = remove {
                app.space_selected.remove(&path);
            }
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new("Protected system paths and folders are blocked. Sable moves files to the Trash (undoable), not permanent delete.")
                .color(SUBTEXT)
                .size(11.5),
        );
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            if primary(ui, "Remove", !busy && !removable.is_empty()).clicked() {
                let paths: Vec<_> = removable;
                app.confirm = Some(Confirm {
                    heading: "Move selected files to the Trash?".into(),
                    body: format!(
                        "This frees about {}. System files stay protected. You can restore items from the Trash.",
                        format_bytes(selected_bytes)
                    ),
                    action: "Move to Trash",
                    kind: ConfirmKind::TrashSpace(paths),
                });
            }
            if ghost(ui, "Clear selection", !busy && !app.space_selected.is_empty()).clicked() {
                app.space_selected.clear();
            }
        });
    });
    });
}

fn space_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Space,
        "Pick a drive or folder, press Scan, then explore the map. Double-click a bubble or folder to zoom in.",
    );
    banner(ui, app);

    let busy = app.busy;
    let layout = app.layout;
    let mut drill = false;
    let mut reveal_path = None;
    let mut open = None;
    let mut up = false;
    let mac = computer_root().display().to_string();

    space_lens_permission_card(ui, app);

    card(ui, |ui| {
        step_label(ui, "1", "Select storage");
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            if chip(ui, "Macintosh HD", app.space_root == mac).clicked() && !busy {
                app.space_root = mac.clone();
            }
            for (name, sub) in [
                ("Home", ""),
                ("Documents", "Documents"),
                ("Downloads", "Downloads"),
                ("Desktop", "Desktop"),
            ] {
                let target = if sub.is_empty() {
                    home.clone()
                } else {
                    home.join(sub)
                }
                .display()
                .to_string();
                if chip(ui, name, app.space_root == target).clicked() && !busy {
                    app.space_root = target;
                }
            }
        });
        ui.collapsing("Custom folder", |ui| {
            field(ui, &mut app.space_root, !busy);
        });
        ui.add_space(8.0);
        step_label(ui, "2", "Scan");
        ui.label(
            RichText::new(format!("Target: {}", shorten(&pretty(&app.home, Path::new(app.space_root.trim())), 72)))
                .size(12.5)
                .color(SUBTEXT)
                .monospace(),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if primary_icon(ui, icons::UiIcon::Scan, "Scan", !busy).clicked() {
                app.start_space();
            }
            if app.space.is_some() && ghost(ui, "Scan again", !busy).clicked() {
                app.start_space();
            }
        });
        if busy {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new("Indexing folders and measuring sizes in the background…")
                        .color(SUBTEXT),
                );
            });
        }
    });
    ui.add_space(10.0);

    if busy && app.space.is_none() {
        loading_card(ui, true, "Building your storage map…", "");
    } else if let Some(report) = app.space.clone() {
        ui.horizontal(|ui| {
            if ghost(
                ui,
                "Up one level",
                !busy && report.root != computer_root() && report.root.parent().is_some(),
            )
            .clicked()
            {
                up = true;
            }
            ui.label(
                RichText::new(shorten(&pretty(&app.home, &report.root), 72))
                    .monospace()
                    .color(SUBTEXT),
            );
        });
        ui.label(
            RichText::new(format!("{} in this folder", format_bytes(report.total)))
                .size(22.0)
                .strong()
                .color(GREEN),
        );
        ui.add_space(8.0);

        let total = report.total.max(1) as f32;
        let mut kids: Vec<&SpaceNode> = report.children.iter().collect();
        kids.sort_by(|a, b| b.bytes.cmp(&a.bytes));
        let index: std::collections::HashMap<PathBuf, u64> =
            kids.iter().map(|node| (node.path.clone(), node.bytes)).collect();
        let bubble_nodes: Vec<_> = kids
            .iter()
            .copied()
            .filter(|node| node.bytes as f32 / total >= SPACE_LENS_BUBBLE_THRESHOLD)
            .collect();
        let other_bytes: u64 = kids
            .iter()
            .filter(|node| node.bytes as f32 / total < SPACE_LENS_BUBBLE_THRESHOLD)
            .map(|node| node.bytes)
            .sum();

        if kids.is_empty() {
            empty(ui, "Nothing to show", "This folder is empty or cannot be read.");
        } else {
            space_lens_results_grid(
                ui,
                app,
                layout,
                &kids,
                &bubble_nodes,
                other_bytes,
                total,
                &index,
                busy,
                &mut open,
                &mut reveal_path,
            );

            if let Some(hover_path) = &app.space_hover {
                if let Some(node) = kids.iter().find(|node| &node.path == hover_path) {
                    space_lens_hover_card(ui.ctx(), &app.home, node);
                }
            }
        }
    } else if !busy {
        empty(
            ui,
            "No map yet",
            "Macintosh HD is selected by default. Press Scan to build your storage map.",
        );
        ui.add_space(8.0);
        if primary(ui, "Scan", true).clicked() {
            app.start_space();
        }
    }

    if let Some(path) = reveal_path {
        reveal(&path);
    }
    if up {
        if let Some(parent) = app.space.as_ref().and_then(|r| r.root.parent()) {
            app.space_root = parent.display().to_string();
            drill = true;
        }
    }
    if let Some(path) = open {
        app.space_root = path.display().to_string();
        drill = true;
    }
    if drill && !busy {
        app.start_space();
    }
}

fn space_lens_results_grid(
    ui: &mut egui::Ui,
    app: &mut ScanApp,
    layout: UiLayout,
    kids: &[&SpaceNode],
    bubble_nodes: &[&SpaceNode],
    other_bytes: u64,
    total: f32,
    index: &std::collections::HashMap<PathBuf, u64>,
    busy: bool,
    open: &mut Option<PathBuf>,
    reveal_path: &mut Option<PathBuf>,
) {
    ui.push_id("space_lens_grid", |ui| {
        let side_registry = !layout.compact && ui.available_width() > 780.0;
        let total_w = ui.available_width();
        let registry_w = if side_registry { 248.0 } else { 0.0 };
        let gap = 12.0;
        let pair_w = (total_w - registry_w - if side_registry { gap } else { 0.0 }).max(360.0);
        let list_w = (pair_w * 0.46).clamp(200.0, 420.0);
        let lens_w = (pair_w - list_w - gap).max(200.0);
        let pane_h = if layout.compact { 380.0 } else { 480.0 };

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(Vec2::new(list_w, pane_h), Layout::top_down(Align::Min), |ui| {
                ui.set_width(list_w);
                space_lens_list_pane(ui, app, kids, total, busy, open, reveal_path);
            });
            ui.allocate_ui_with_layout(Vec2::new(lens_w, pane_h), Layout::top_down(Align::Min), |ui| {
                ui.set_width(lens_w);
                space_lens_bubble_pane(ui, app, bubble_nodes, other_bytes, total, layout.compact, busy, open);
            });
            if side_registry {
                ui.allocate_ui_with_layout(Vec2::new(registry_w, pane_h), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(registry_w);
                    space_lens_selection_registry(ui, app, index, busy);
                });
            }
        });
        if !side_registry {
            ui.add_space(10.0);
            space_lens_selection_registry(ui, app, index, busy);
        }
    });
}

fn space_lens_list_pane(
    ui: &mut egui::Ui,
    app: &mut ScanApp,
    kids: &[&SpaceNode],
    total: f32,
    busy: bool,
    open: &mut Option<PathBuf>,
    reveal_path: &mut Option<PathBuf>,
) {
    ui.push_id("space_lens_list_pane", |ui| {
    card(ui, |ui| {
        label_small(ui, "LIST — LARGEST FIRST");
        egui::ScrollArea::vertical()
            .id_salt("space_lens_list_scroll")
            .max_height(420.0)
            .show(ui, |ui| {
            for node in kids {
                let share = node.bytes as f32 / total;
                let spared = is_spared(&node.path, &app.home);
                let response = ui
                    .push_id(node.path.display().to_string(), |ui| {
                        row(ui, false, true, |ui| {
                            ui.horizontal(|ui| {
                                let mut on = app.space_selected.contains(&node.path);
                                ui.add_enabled_ui(!spared && !node.is_directory, |ui| {
                                    if ui.checkbox(&mut on, "").changed() {
                                        if on {
                                            app.space_selected.insert(node.path.clone());
                                        } else {
                                            app.space_selected.remove(&node.path);
                                        }
                                    }
                                });
                                if spared || node.is_directory {
                                    ui.add_space(20.0);
                                }
                                let text_w = (ui.available_width() * 0.72).clamp(120.0, 280.0);
                                ui.vertical(|ui| {
                                    ui.set_max_width(text_w);
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.label(RichText::new(file_name(&node.path)).strong());
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · {}",
                                            space_item_kind(node),
                                            format_modified(node.modified)
                                        ))
                                        .size(11.5)
                                        .color(SUBTEXT),
                                    );
                                    meter_with_width(
                                        ui,
                                        share,
                                        if share > 0.4 { AMBER } else { GREEN },
                                        text_w,
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format_bytes(node.bytes)).strong().color(GREEN));
                                });
                            });
                        })
                    })
                    .inner;
                if response.hovered() {
                    app.space_hover = Some(node.path.clone());
                }
                if response.double_clicked() && node.is_directory && !busy {
                    *open = Some(node.path.clone());
                }
                if response.secondary_clicked() {
                    *reveal_path = Some(node.path.clone());
                }
            }
        });
    });
    });
}

fn space_lens_bubble_pane(
    ui: &mut egui::Ui,
    app: &mut ScanApp,
    bubble_nodes: &[&SpaceNode],
    other_bytes: u64,
    total: f32,
    compact: bool,
    busy: bool,
    open: &mut Option<PathBuf>,
) {
    ui.push_id("space_lens_bubble_pane", |ui| {
    card(ui, |ui| {
        label_small(ui, "LENS — PROPORTIONAL MAP");
        ui.label(
            RichText::new("Circle size = disk space. Items under 2% roll into “Other”.")
                .color(SUBTEXT)
                .size(12.0),
        );
        ui.add_space(6.0);
        egui::ScrollArea::both()
            .id_salt("space_lens_bubble_scroll")
            .auto_shrink([false; 2])
            .max_height(420.0)
            .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for node in bubble_nodes {
                    let share = node.bytes as f32 / total;
                    let selected = app.space_selected.contains(&node.path);
                    let name = file_name(&node.path);
                    let response = ui
                        .push_id(node.path.display().to_string(), |ui| {
                            bubble(ui, &name, share, selected, compact)
                        })
                        .inner;
                    if response.hovered() {
                        app.space_hover = Some(node.path.clone());
                    }
                    if response.double_clicked() && node.is_directory && !busy {
                        *open = Some(node.path.clone());
                    }
                }
                if other_bytes > 0 {
                    let share = other_bytes as f32 / total;
                    ui.push_id("other_bucket", |ui| bubble(ui, "Other", share, false, compact));
                }
            });
        });
    });
    });
}

fn cloud_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        app.layout,
        Screen::Cloud,
        "How much space iCloud Drive and other cloud services use on this Mac. Nothing is signed in, uploaded, or deleted.",
    );
    safety(ui, Safety::ReadOnly);
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
                    ui.label(RichText::new(&folder.label).strong().size(16.0));
                    ui.label(
                        RichText::new(if folder.exists {
                            shorten(&pretty(&app.home, &folder.path), 70)
                        } else {
                            "This service is not set up on this Mac.".to_string()
                        })
                        .size(12.5)
                        .color(MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if folder.exists {
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(folder.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(folder.bytes)).strong().size(18.0).color(LIME));
                    }
                });
            });
            if folder.exists {
                ui.add_space(4.0);
                meter(ui, folder.bytes as f32 / biggest as f32, SKY);
            }
        });
    }
    if let Some(path) = reveal_path {
        reveal(&path);
    }
    ui.add_space(6.0);
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
    safety(ui, Safety::ReadOnly);
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
            ui.label(RichText::new(value).size(36.0).strong().color(gauge(frac)));
            meter(ui, frac, gauge(frac));
            ui.label(RichText::new(caption).color(SUBTEXT).size(13.0));
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
    ui.add_space(8.0);
    let verdict = if mem_pct > 0.85 {
        ("Memory is nearly full. Quit apps you are not using.", AMBER)
    } else if cpu > 0.8 {
        ("The processor is very busy. Something heavy is running.", AMBER)
    } else {
        ("Everything looks fine.", LIME)
    };
    card(ui, |ui| {
        ui.label(RichText::new(verdict.0).color(verdict.1).strong());
    });
    ui.add_space(6.0);
    if ghost(ui, "Refresh", !busy).clicked() {
        app.sample_machine();
    }
}

// ───────────────────────────── widgets ─────────────────────────────

fn disk_access_card(ui: &mut egui::Ui, granted: bool) {
    label_small(ui, "DISK ACCESS");
    ui.label(
        RichText::new(if granted { "Full" } else { "Limited" })
            .size(28.0)
            .strong()
            .color(if granted { GREEN } else { AMBER }),
    );
    ui.label(
        RichText::new(if granted {
            "Sable can read every folder it needs."
        } else {
            "Some folders are off limits, so sizes may look smaller than they are."
        })
        .color(SUBTEXT)
        .size(13.0),
    );
    if !granted && ghost(ui, "Open Privacy settings", true).clicked() {
        open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
    }
}

#[derive(Clone, Copy)]
enum Safety {
    ReadOnly,
    Trash,
    Admin,
}

fn safety(ui: &mut egui::Ui, kind: Safety) {
    let (icon, text, color) = match kind {
        Safety::ReadOnly => (UiIcon::Eye, "Read-only — nothing on disk is changed", LIME),
        Safety::Trash => (UiIcon::Trash, "Moves to Trash — you can undo this", AMBER),
        Safety::Admin => (UiIcon::Lock, "May ask for your Mac password", ROSE),
    };
    theme::glass_pill().show(ui, |ui| {
        icon_with_label(ui, icon, text, color);
    });
    ui.add_space(6.0);
}

fn page_header(ui: &mut egui::Ui, layout: UiLayout, screen: Screen, subtitle: &str) {
    theme::glass_card().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
            paint_screen_icon(ui.painter(), screen, icon_rect, GREEN);
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("SABLE").size(11.0).color(GREEN).strong());
                ui.label(
                    RichText::new(screen.label())
                        .font(heading_font(layout.title_size()))
                        .color(TEXT),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(subtitle)
                        .color(SUBTEXT)
                        .size(layout.subtitle_size()),
                );
            });
        });
    });
    ui.add_space(12.0);
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    label_small(ui, text);
}

fn label_small(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(SUBTEXT).strong());
}

fn step_label(ui: &mut egui::Ui, number: &str, text: &str) {
    ui.horizontal(|ui| {
        badge(ui, number, LIME);
        ui.label(RichText::new(text).font(heading_font(15.0)));
    });
}

fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90)))
        .rounding(Rounding::same(20.0))
        .inner_margin(Margin::symmetric(10.0, 3.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(color));
        });
}

fn banner(ui: &mut egui::Ui, app: &mut ScanApp) {
    let Some(notice) = &app.notice else {
        return;
    };
    let accent = if notice.ok { LIME } else { ROSE };
    let text = notice.text.clone();
    let mut dismiss = false;
    theme::glass_notice(accent).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                paint_ui_icon(
                    ui.painter(),
                    if notice.ok { UiIcon::Check } else { UiIcon::Alert },
                    icon_rect,
                    accent,
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(if notice.ok { "Done" } else { "That did not work" })
                            .font(heading_font(14.0))
                            .color(accent),
                    );
                    ui.label(RichText::new(&text).color(IVORY));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost(ui, "Dismiss", true).clicked() {
                        dismiss = true;
                    }
                });
            });
        });
    ui.add_space(10.0);
    if dismiss {
        app.notice = None;
    }
}

fn hero(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    theme::glass_hero(GREEN).show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui);
    });
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    theme::glass_card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui);
    });
    ui.add_space(4.0);
}

fn summary_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    caption: &str,
    action: &str,
    busy: bool,
    on_click: &mut dyn FnMut(),
) {
    card(ui, |ui| {
        label_small(ui, label);
        ui.label(RichText::new(value).size(28.0).strong());
        ui.label(RichText::new(caption).color(MUTED).size(13.0));
        if ghost(ui, action, !busy).clicked() {
            on_click();
        }
    });
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
        ui.label(RichText::new(heading).strong().size(15.0));
        ui.label(RichText::new(body).color(MUTED).size(13.0));
        safety(ui, kind);
        if ghost(ui, action, enabled).clicked() {
            on_click();
        }
    });
}

fn empty(ui: &mut egui::Ui, heading: &str, body: &str) {
    card(ui, |ui| {
        ui.add_space(14.0);
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
            icons::paint_ui_icon(ui.painter(), icons::UiIcon::Info, rect, MUTED);
            ui.add_space(6.0);
            ui.label(RichText::new(heading).size(16.0).strong().font(icons::heading_font(16.0)));
            ui.add_space(2.0);
            ui.label(RichText::new(body).color(MUTED));
        });
        ui.add_space(14.0);
    });
}

fn loading_line(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new(text).color(MUTED));
    });
}

/// Spinner while work is running, otherwise an empty-state card.
fn loading_card(ui: &mut egui::Ui, busy: bool, loading: &str, idle: &str) {
    if busy {
        card(ui, |ui| {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(6.0);
                ui.label(RichText::new(loading).color(MUTED));
            });
            ui.add_space(12.0);
        });
    } else if !idle.is_empty() {
        empty(ui, "Nothing here", idle);
    }
}

fn row(ui: &mut egui::Ui, selected: bool, clickable: bool, body: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let bg = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::none()
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .rounding(Rounding::same(10.0))
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
        Color32::from_rgba_unmultiplied(29, 185, 84, 32)
    } else if hovered {
        CARD_HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().set(bg, Shape::rect_filled(response.rect, Rounding::same(10.0), fill));
    if clickable && hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn avatar(ui: &mut egui::Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        Rounding::same(9.0),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 34),
    );
    let letter = text
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, letter, FontId::proportional(16.0), color);
}

fn cloud_label(status: CloudStatus) -> &'static str {
    match status {
        CloudStatus::NotCloud => "On this Mac",
        CloudStatus::CloudLocal => "Cloud, stored locally",
        CloudStatus::CloudEvicted => "Cloud, not on this Mac",
    }
}

fn bubble(ui: &mut egui::Ui, name: &str, share: f32, selected: bool, compact: bool) -> egui::Response {
    let scale = if compact { 0.72 } else { 1.0 };
    let diameter = (36.0 + 70.0 * share.sqrt()) * scale;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(diameter + 8.0, diameter + 22.0), egui::Sense::click());
    let center = rect.center() - Vec2::new(0.0, 8.0);
    ui.painter().circle_filled(center, diameter / 2.0, if selected { GREEN } else { CARD_HOVER });
    ui.painter().circle_stroke(center, diameter / 2.0, Stroke::new(1.0, if selected { GREEN } else { LINE }));
    ui.painter().text(
        rect.center_bottom(),
        egui::Align2::CENTER_BOTTOM,
        if name.chars().count() > 12 {
            format!("{}…", name.chars().take(11).collect::<String>())
        } else {
            name.to_string()
        },
        FontId::new(if compact { 10.0 } else { 11.0 }, FontFamily::Proportional),
        if selected { GREEN_ON } else { SUBTEXT },
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn meter(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    let width = ui.available_width().clamp(48.0, 220.0);
    meter_with_width(ui, fraction, color, width);
}

fn meter_with_width(ui: &mut egui::Ui, fraction: f32, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width.max(40.0), 6.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(3.0), LINE);
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction > 0.0 {
        let mut fill = rect;
        fill.set_width((rect.width() * fraction).max(4.0));
        ui.painter().rect_filled(fill, Rounding::same(3.0), color);
    }
}

fn gauge(fraction: f32) -> Color32 {
    if fraction > 0.85 {
        ROSE
    } else if fraction > 0.65 {
        AMBER
    } else {
        LIME
    }
}

fn stat(ui: &mut egui::Ui, label: &str, value: u64, value_color: Color32) {
    card(ui, |ui| {
        label_small(ui, &label.to_uppercase());
        ui.label(RichText::new(format_count(value)).size(28.0).strong().color(value_color));
    });
}

fn status_pill(ui: &mut egui::Ui, status: &str, busy: bool, time: f64) {
    let (color, text) = match status {
        "paused" => (AMBER, "Paused"),
        "canceling" => (ROSE, "Stopping..."),
        "cleaning" => (AMBER, "Moving to Trash..."),
        _ if busy => (LIME, "Working..."),
        _ => (MUTED, "Ready"),
    };
    let alpha = if busy { 0.45 + 0.55 * (time * 3.0).sin().abs() as f32 } else { 1.0 };
    let dot = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (alpha * 255.0) as u8);
    theme::glass_pill().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, dot);
            ui.label(RichText::new(text).size(12.5).color(TEXT));
        });
    });
}

fn field(ui: &mut egui::Ui, text: &mut String, enabled: bool) {
    ui.add(
        egui::TextEdit::singleline(text)
            .desired_width(ui.available_width())
            .interactive(enabled)
            .margin(Vec2::new(10.0, 8.0))
            .font(TextStyle::Monospace),
    );
}

fn chip(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(13.0).color(if active { GREEN_ON } else { TEXT }))
            .fill(if active { GREEN } else { ELEVATED })
            .stroke(Stroke::new(1.0, if active { GREEN } else { LINE }))
            .rounding(Rounding::same(500.0))
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

fn primary_icon(ui: &mut egui::Ui, icon: icons::UiIcon, label: &str, enabled: bool) -> egui::Response {
    let fg = if enabled { GREEN_ON } else { SUBTEXT };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(15.0), Sense::hover());
        icons::paint_ui_icon(ui.painter(), icon, rect, fg);
        primary(ui, label, enabled)
    })
    .inner
}

fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    if enabled {
        ui.add(
            egui::Button::new(RichText::new(label).color(GREEN_ON).strong())
                .fill(GREEN)
                .rounding(Rounding::same(500.0))
                .min_size(Vec2::new(120.0, 40.0)),
        )
    } else {
        ui.add_enabled(
            false,
            egui::Button::new(RichText::new(label).color(SUBTEXT))
                .fill(CARD)
                .stroke(Stroke::new(1.0, LINE))
                .rounding(Rounding::same(500.0))
                .min_size(Vec2::new(120.0, 40.0)),
        )
    }
}

fn ghost(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(TEXT))
            .fill(ELEVATED)
            .stroke(Stroke::new(1.0, LINE))
            .rounding(Rounding::same(500.0))
            .min_size(Vec2::new(70.0, 32.0)),
    )
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
