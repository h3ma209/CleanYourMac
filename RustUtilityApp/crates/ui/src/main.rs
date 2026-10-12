use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, FontFamily, FontId, Layout, Margin, RichText, Rounding, Sense, Shape, Stroke,
    TextStyle, Vec2,
};
use engine::{
    flush_dns_cache, folder_access, junk_roots, large_downloads, launch_agents, local_cloud_folders,
    machine_sample, move_download_to_trash, move_root_to_trash, reindex_spotlight, run_care, run_scan,
    scan_space, spotlight_status, survey_leftovers, trash_selected, computer_root, is_spared, AccessStatus, CareReport, CloudFolder, CloudStatus, Detection,
    ExecutionControl, ExtensionWatchDetector, InstalledApp, JunkRoot, LargeFile, LaunchAgent, LeftoverHit,
    MachineSample, ScanConfig, ScanEvent, SpaceReport,
};
use tokio::sync::mpsc;

const INK: Color32 = Color32::from_rgb(13, 14, 17);
const SURFACE: Color32 = Color32::from_rgb(20, 21, 26);
const CARD: Color32 = Color32::from_rgb(28, 29, 35);
const CARD_HOVER: Color32 = Color32::from_rgb(38, 39, 46);
const LINE: Color32 = Color32::from_rgb(46, 47, 55);
const IVORY: Color32 = Color32::from_rgb(244, 241, 234);
const MUTED: Color32 = Color32::from_rgb(158, 153, 144);
const LIME: Color32 = Color32::from_rgb(214, 245, 92);
const LIME_INK: Color32 = Color32::from_rgb(28, 36, 8);
const AMBER: Color32 = Color32::from_rgb(232, 168, 74);
const ROSE: Color32 = Color32::from_rgb(232, 112, 106);
const SKY: Color32 = Color32::from_rgb(120, 190, 240);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Screen {
    Care,
    Cleanup,
    Apps,
    Performance,
    Protection,
    Clutter,
    Space,
    Cloud,
    Assistant,
}

impl Screen {
    const ALL: [Screen; 9] = [
        Self::Care,
        Self::Cleanup,
        Self::Apps,
        Self::Performance,
        Self::Protection,
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
            Self::Clutter => "Big Downloads",
            Self::Space => "Disk Usage",
            Self::Cloud => "Cloud Folders",
            Self::Assistant => "Live Stats",
        }
    }

    fn tagline(self) -> &'static str {
        match self {
            Self::Care => "Health check of your Mac",
            Self::Cleanup => "Caches and logs",
            Self::Apps => "Installed apps and leftovers",
            Self::Performance => "Startup items and fixes",
            Self::Protection => "Find files by type",
            Self::Clutter => "Large files you forgot",
            Self::Space => "What is using your disk",
            Self::Cloud => "iCloud on this Mac",
            Self::Assistant => "CPU and memory right now",
        }
    }

    /// Screens that fill themselves the first time you open them.
    fn auto_loads(self) -> bool {
        !matches!(self, Self::Protection)
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
            .with_min_inner_size([960.0, 620.0]),
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
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.scroll.bar_width = 6.0;
    style.visuals.window_fill = INK;
    style.visuals.panel_fill = INK;
    style.visuals.extreme_bg_color = INK;
    style.visuals.faint_bg_color = CARD;
    style.visuals.code_bg_color = CARD;
    style.visuals.widgets.noninteractive.bg_fill = CARD;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, IVORY);
    style.visuals.widgets.inactive.bg_fill = CARD;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, IVORY);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE);
    style.visuals.widgets.hovered.bg_fill = CARD_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, IVORY);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, LIME);
    style.visuals.widgets.active.bg_fill = LIME;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, LIME_INK);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(214, 245, 92, 40);
    style.visuals.selection.stroke = Stroke::new(1.0, LIME);
    style.visuals.override_text_color = Some(IVORY);
    style.text_styles.insert(TextStyle::Heading, FontId::new(26.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body, FontId::new(14.5, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace));
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
    space_review: bool,
    clutter: Vec<LargeFile>,
    cloud: Vec<CloudFolder>,
    access: AccessStatus,
    events: Option<mpsc::Receiver<UiMsg>>,
    control: Option<Arc<ExecutionControl>>,
    busy: bool,
}

impl ScanApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
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
            space_review: false,
            clutter: Vec::new(),
            cloud: Vec::new(),
            access,
            events: None,
            control: None,
            busy: false,
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
            Screen::Protection => {}
            Screen::Space => self.start_space(),
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

    fn start_space(&mut self) {
        if self.busy {
            return;
        }
        let root = PathBuf::from(self.space_root.trim());
        let tx = self.arm();
        self.status = "scanning";
        self.runtime.spawn(async move {
            let report = scan_space(root).await;
            let _ = tx.send(UiMsg::Space(report)).await;
        });
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
                UiMsg::Space(report) => {
                    self.space = Some(report);
                    self.space_selected.clear();
                    self.space_review = false;
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
            .frame(
                egui::Frame::none()
                    .fill(CARD)
                    .stroke(Stroke::new(1.0, LINE))
                    .rounding(Rounding::same(18.0))
                    .inner_margin(Margin::same(26.0)),
            )
            .show(ctx, |ui| {
                ui.set_width(420.0);
                ui.label(RichText::new(heading).size(20.0).strong());
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

        egui::SidePanel::left("nav")
            .exact_width(252.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, LINE))
                    .inner_margin(Margin::same(18.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::same(9.0), LIME);
                    ui.painter().circle_filled(rect.center(), 5.5, LIME_INK);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new("Sable").strong().size(17.0));
                        ui.label(RichText::new("Mac cleaner and tune-up").size(11.5).color(MUTED));
                    });
                });
                ui.add_space(22.0);
                for item in Screen::ALL {
                    let active = item == screen;
                    let response = egui::Frame::none()
                        .fill(if active { Color32::from_rgb(36, 39, 30) } else { Color32::TRANSPARENT })
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(
                                RichText::new(item.label())
                                    .size(14.0)
                                    .strong()
                                    .color(if active { LIME } else { IVORY }),
                            );
                            ui.label(RichText::new(item.tagline()).size(11.5).color(MUTED));
                        })
                        .response
                        .interact(Sense::click());
                    if response.clicked() {
                        self.go(item);
                    }
                    ui.add_space(2.0);
                }
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    status_pill(ui, status, busy, pulse);
                    ui.add_space(8.0);
                    let granted = self.access == AccessStatus::Granted;
                    ui.label(
                        RichText::new(if granted {
                            "Full disk access on"
                        } else {
                            "Limited disk access"
                        })
                        .size(12.0)
                        .color(if granted { LIME } else { AMBER }),
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(INK).inner_margin(Margin::symmetric(32.0, 26.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(900.0);
                        match self.screen {
                            Screen::Care => care_screen(ui, self),
                            Screen::Cleanup => cleanup_screen(ui, self),
                            Screen::Apps => apps_screen(ui, self),
                            Screen::Performance => performance_screen(ui, self),
                            Screen::Protection => protection_screen(ui, self),
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

// ───────────────────────────── screens ─────────────────────────────

fn care_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        "Overview",
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

    hero(ui, |ui| match reclaim {
        Some(bytes) => {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    label_small(ui, "YOU CAN FREE UP");
                    ui.label(RichText::new(format_bytes(bytes)).size(44.0).strong().color(LIME));
                    ui.label(
                        RichText::new("of temporary files (caches and logs). Apps rebuild what they need.")
                            .color(MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if primary(ui, "Review and clean up", true).clicked() {
                        goto = Some(Screen::Cleanup);
                    }
                });
            });
        }
        None => loading_line(ui, "Checking your Mac..."),
    });
    ui.add_space(14.0);

    if let Some(report) = &app.care {
        let (apps, agents, access) = (report.app_count, report.agent_count, report.access);
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
            let granted = access == AccessStatus::Granted;
            card(&mut cols[2], |ui| {
                label_small(ui, "DISK ACCESS");
                ui.label(
                    RichText::new(if granted { "Full" } else { "Limited" })
                        .size(28.0)
                        .strong()
                        .color(if granted { LIME } else { AMBER }),
                );
                ui.label(
                    RichText::new(if granted {
                        "Sable can read every folder it needs."
                    } else {
                        "Some folders are off limits, so sizes may look smaller than they are."
                    })
                    .color(MUTED)
                    .size(13.0),
                );
                if !granted && ghost(ui, "Open Privacy settings", true).clicked() {
                    open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
                }
            });
        });
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
                ui.vertical(|ui| {
                    ui.label(RichText::new(screen.label()).strong());
                    ui.label(RichText::new(hint).color(MUTED).size(13.0));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("Open").color(LIME));
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
        "Free Up Space",
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
        "Apps",
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
        "Tune-Up",
        "See which apps start automatically in the background, and run a few common macOS fixes.",
    );
    banner(ui, app);
    let busy = app.busy;

    section(ui, "FIXES AND MAINTENANCE");
    let mut tool = None;
    let mut request = None;
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
        "File Finder",
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
        ui.columns(3, |cols| {
            stat(&mut cols[0], "Files checked", app.scanned, IVORY);
            stat(&mut cols[1], "Matches", app.detection_count, LIME);
            stat(&mut cols[2], "Could not read", app.errors, if app.errors > 0 { AMBER } else { IVORY });
        });
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

fn clutter_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        "Big Downloads",
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
        for file in &app.clutter {
            row(ui, false, false, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(shorten(&file_name(&file.path), 55)).strong());
                        ui.label(
                            RichText::new(shorten(&pretty(&app.home, file.path.parent().unwrap_or(Path::new(""))), 70))
                                .size(12.0)
                                .color(MUTED),
                        );
                        meter(ui, file.bytes as f32 / biggest as f32, LIME);
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ghost(ui, "Move to Trash", !busy).clicked() {
                            request = Some(Confirm {
                                heading: format!("Move \"{}\" to the Trash?", shorten(&file_name(&file.path), 40)),
                                body: format!(
                                    "This frees {}. You can put it back from the Trash if you change your mind.",
                                    format_bytes(file.bytes)
                                ),
                                action: "Move to Trash",
                                kind: ConfirmKind::TrashDownload(file.path.clone()),
                            });
                        }
                        if ghost(ui, "Show", true).clicked() {
                            reveal_path = Some(file.path.clone());
                        }
                        ui.label(RichText::new(format_bytes(file.bytes)).strong().color(LIME));
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

fn space_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
        page_header(
        ui,
        "Space Lens",
        "Starts on this Mac. Open a bubble to look inside. A single folder is optional.",
    );
    safety(ui, Safety::ReadOnly);
    banner(ui, app);

    let busy = app.busy;
    let mut measure = false;
    let mac = computer_root().display().to_string();
    card(ui, |ui| {
        let home = app.home.clone();
        ui.horizontal_wrapped(|ui| {
            if chip(ui, "This Mac", app.space_root == mac).clicked() && !busy {
                app.space_root = mac.clone();
                measure = true;
            }
            for (name, sub) in [
                ("Home", ""),
                ("Documents", "Documents"),
                ("Downloads", "Downloads"),
                ("Desktop", "Desktop"),
            ] {
                let target = if sub.is_empty() { home.clone() } else { home.join(sub) }.display().to_string();
                if chip(ui, name, app.space_root == target).clicked() && !busy {
                    app.space_root = target;
                    measure = true;
                }
            }
        });
        ui.collapsing("Choose a folder", |ui| {
            field(ui, &mut app.space_root, !busy);
            if primary(ui, "Measure this folder", !busy).clicked() {
                measure = true;
            }
        });
        if busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Scanning. The whole Mac can take several minutes.").color(MUTED));
            });
        }
    });
    ui.add_space(12.0);

    let mut open = None;
    let mut reveal_path = None;
    let mut up = false;
    if let Some(report) = &app.space {
        ui.horizontal(|ui| {
            if ghost(ui, "Up one level", !busy && report.root != computer_root() && report.root.parent().is_some()).clicked() {
                up = true;
            }
            ui.label(RichText::new(shorten(&pretty(&app.home, &report.root), 70)).monospace().color(MUTED));
        });
        ui.label(
            RichText::new(format!("{} in total", format_bytes(report.total)))
                .size(22.0)
                .strong()
                .color(LIME),
        );
        ui.add_space(6.0);
        let total = report.total.max(1) as f32;
        let mut kids: Vec<_> = report.children.iter().collect();
        kids.sort_by(|a, b| b.bytes.cmp(&a.bytes));
        let mut major = Vec::new();
        let mut other = Vec::new();
        for node in &kids {
            if node.bytes as f32 / total >= 0.02 {
                major.push(*node);
            } else {
                other.push(*node);
            }
        }
        if kids.is_empty() {
            empty(ui, "Nothing inside", "This folder is empty or cannot be read.");
        } else {
            let mut pick = None;
            ui.horizontal_wrapped(|ui| {
                for node in &major {
                    let share = node.bytes as f32 / total;
                    let selected = app.space_selected.contains(&node.path);
                    if bubble(ui, &file_name(&node.path), share, selected).clicked() {
                        pick = Some(node.path.clone());
                    }
                }
                if !other.is_empty() {
                    let other_bytes = other.iter().fold(0u64, |sum, node| sum.saturating_add(node.bytes));
                    let share = other_bytes as f32 / total;
                    bubble(ui, "Other", share, false);
                }
            });
            if let Some(path) = pick {
                if app.space_selected.contains(&path) {
                    app.space_selected.remove(&path);
                } else {
                    app.space_selected.insert(path);
                }
            }
            ui.add_space(8.0);
            let removable = app.space_selected.iter().filter(|path| !is_spared(path, &app.home)).count();
            ui.horizontal(|ui| {
                if primary(ui, "Review", !busy && removable > 0).clicked() {
                    app.space_review = true;
                }
                if app.space_review && primary(ui, "Move files to Trash", !busy && removable > 0).clicked() {
                    let paths = app.space_selected.iter().cloned().collect();
                    app.confirm = Some(Confirm {
                        heading: "Move the reviewed files to the Trash?".into(),
                        body: "Folders and system locations are spared. You can put files back from the Trash.".into(),
                        action: "Move to Trash",
                        kind: ConfirmKind::TrashSpace(paths),
                    });
                }
            });
            if app.space_review {
                ui.add_space(6.0);
                card(ui, |ui| {
                    ui.label(RichText::new("Review").strong());
                    let selected: Vec<_> = kids.iter().filter(|node| app.space_selected.contains(&node.path)).collect();
                    if selected.is_empty() {
                        ui.label(RichText::new("Nothing selected.").color(MUTED));
                    }
                    for node in selected {
                        let spared = is_spared(&node.path, &app.home);
                        ui.label(RichText::new(format!(
                            "{}   {}{}",
                            format_bytes(node.bytes),
                            file_name(&node.path),
                            if spared { "   spared" } else { "" }
                        )).monospace().size(12.0).color(if spared { MUTED } else { IVORY }));
                    }
                });
            }
            ui.add_space(8.0);
            card(ui, |ui| {
                let rows = if other.is_empty() { major.clone() } else { kids.clone() };
                for node in rows {
                    let share = node.bytes as f32 / total;
                    let in_other = node.bytes as f32 / total < 0.02;
                    row(ui, false, false, |ui| {
                        ui.horizontal(|ui| {
                            let mut on = app.space_selected.contains(&node.path);
                            if ui.checkbox(&mut on, "").changed() {
                                if on {
                                    app.space_selected.insert(node.path.clone());
                                } else {
                                    app.space_selected.remove(&node.path);
                                }
                            }
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 3.0;
                                ui.label(RichText::new(format!(
                                    "{}{}{}",
                                    file_name(&node.path),
                                    if node.is_directory { "  (folder)" } else { "" },
                                    if in_other { "  · Other" } else { "" }
                                )).strong());
                                ui.label(RichText::new(cloud_label(node.cloud)).size(12.0).color(MUTED));
                                meter(ui, share, if share > 0.4 { AMBER } else { LIME });
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if node.is_directory && ghost(ui, "Open", !busy).clicked() {
                                    open = Some(node.path.clone());
                                }
                                if ghost(ui, "Show", true).clicked() {
                                    reveal_path = Some(node.path.clone());
                                }
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.label(RichText::new(format_bytes(node.bytes)).strong().color(LIME));
                                    ui.label(RichText::new(format!("{:.0}%", share * 100.0)).size(12.0).color(MUTED));
                                });
                            });
                        });
                    });
                }
            });
        }
    } else if !busy {
        empty(ui, "Nothing measured yet", "This Mac is the default. Open Space Lens and the scan starts.");
    }

    if let Some(path) = reveal_path {
        reveal(&path);
    }
    if up {
        if let Some(parent) = app.space.as_ref().and_then(|r| r.root.parent()) {
            app.space_root = parent.display().to_string();
            measure = true;
        }
    }
    if let Some(path) = open {
        app.space_root = path.display().to_string();
        measure = true;
    }
    if measure {
        app.start_space();
    }
}

fn cloud_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    page_header(
        ui,
        "Cloud Folders",
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
        "Live Stats",
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

    ui.columns(2, |cols| {
        card(&mut cols[0], |ui| {
            label_small(ui, "PROCESSOR");
            ui.label(RichText::new(format!("{:.0}%", sample.cpu_percent)).size(36.0).strong().color(gauge(cpu)));
            meter(ui, cpu, gauge(cpu));
            ui.label(RichText::new("How busy the CPU is.").color(MUTED).size(13.0));
        });
        card(&mut cols[1], |ui| {
            label_small(ui, "MEMORY");
            ui.label(RichText::new(format!("{:.0}%", mem_pct * 100.0)).size(36.0).strong().color(gauge(mem_pct)));
            meter(ui, mem_pct, gauge(mem_pct));
            ui.label(
                RichText::new(format!(
                    "{} of {} in use",
                    format_bytes(sample.used_memory),
                    format_bytes(sample.total_memory)
                ))
                .color(MUTED)
                .size(13.0),
            );
        });
    });
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

#[derive(Clone, Copy)]
enum Safety {
    ReadOnly,
    Trash,
    Admin,
}

fn safety(ui: &mut egui::Ui, kind: Safety) {
    let (text, color) = match kind {
        Safety::ReadOnly => ("Read-only: nothing is changed", LIME),
        Safety::Trash => ("Moves to Trash: you can undo it", AMBER),
        Safety::Admin => ("May ask for your password", ROSE),
    };
    badge(ui, text, color);
    ui.add_space(6.0);
}

fn page_header(ui: &mut egui::Ui, heading: &str, subtitle: &str) {
    ui.label(RichText::new(heading).size(32.0).strong());
    ui.add_space(2.0);
    ui.label(RichText::new(subtitle).color(MUTED).size(14.5));
    ui.add_space(10.0);
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    label_small(ui, text);
}

fn label_small(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(MUTED).strong());
}

fn step_label(ui: &mut egui::Ui, number: &str, text: &str) {
    ui.horizontal(|ui| {
        badge(ui, number, LIME);
        ui.label(RichText::new(text).strong().size(15.0));
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
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 22))
        .stroke(Stroke::new(1.0, accent))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::symmetric(16.0, 12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(if notice.ok { "Done" } else { "That did not work" })
                            .strong()
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
    egui::Frame::none()
        .fill(Color32::from_rgb(30, 34, 24))
        .stroke(Stroke::new(1.0, Color32::from_rgb(70, 82, 40)))
        .rounding(Rounding::same(18.0))
        .inner_margin(Margin::same(22.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(Rounding::same(14.0))
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
    ui.add_space(2.0);
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
            ui.label(RichText::new(heading).size(16.0).strong());
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
        Color32::from_rgba_unmultiplied(214, 245, 92, 26)
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

fn bubble(ui: &mut egui::Ui, name: &str, share: f32, selected: bool) -> egui::Response {
    let diameter = 36.0 + 70.0 * share.sqrt();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(diameter + 8.0, diameter + 22.0), egui::Sense::click());
    let center = rect.center() - Vec2::new(0.0, 8.0);
    ui.painter().circle_filled(center, diameter / 2.0, if selected { LIME } else { Color32::from_rgb(48, 62, 28) });
    ui.painter().text(
        rect.center_bottom(),
        egui::Align2::CENTER_BOTTOM,
        if name.chars().count() > 12 { format!("{}…", name.chars().take(11).collect::<String>()) } else { name.to_string() },
        FontId::new(11.0, FontFamily::Proportional),
        if selected { LIME_INK } else { MUTED },
    );
    response
}

fn meter(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().max(40.0), 7.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(4.0), LINE);
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction > 0.0 {
        let mut fill = rect;
        fill.set_width((rect.width() * fraction).max(7.0));
        ui.painter().rect_filled(fill, Rounding::same(4.0), color);
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
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(Rounding::same(20.0))
        .inner_margin(Margin::symmetric(12.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, dot);
                ui.label(RichText::new(text).size(12.5));
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
        egui::Button::new(RichText::new(label).size(13.0).color(if active { LIME_INK } else { IVORY }))
            .fill(if active { LIME } else { SURFACE })
            .stroke(Stroke::new(1.0, if active { LIME } else { LINE }))
            .rounding(Rounding::same(16.0))
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(LIME_INK).strong())
            .fill(LIME)
            .rounding(Rounding::same(10.0))
            .min_size(Vec2::new(120.0, 38.0)),
    )
}

fn ghost(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(IVORY))
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, LINE))
            .rounding(Rounding::same(10.0))
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
