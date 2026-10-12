use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Rounding, Stroke, TextStyle, Vec2};
use engine::{
    flush_dns_cache, folder_access, junk_roots, launch_agents, machine_sample, move_root_to_trash,
    reindex_spotlight, run_care, run_scan, scan_space, spotlight_status, survey_leftovers, AccessStatus,
    CareReport, Detection, ExecutionControl, ExtensionWatchDetector, InstalledApp, JunkRoot, LaunchAgent,
    LeftoverHit, MachineSample, ScanConfig, ScanEvent, SpaceReport,
};
use tokio::sync::mpsc;

const INK: Color32 = Color32::from_rgb(14, 15, 18);
const SURFACE: Color32 = Color32::from_rgb(22, 23, 28);
const CARD: Color32 = Color32::from_rgb(30, 31, 37);
const LINE: Color32 = Color32::from_rgb(48, 49, 56);
const IVORY: Color32 = Color32::from_rgb(244, 241, 234);
const MUTED: Color32 = Color32::from_rgb(154, 149, 140);
const LIME: Color32 = Color32::from_rgb(214, 245, 92);
const LIME_INK: Color32 = Color32::from_rgb(28, 36, 8);
const AMBER: Color32 = Color32::from_rgb(232, 168, 74);
const ROSE: Color32 = Color32::from_rgb(232, 112, 106);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Care,
    Protection,
    Cleanup,
    Performance,
    Apps,
    Space,
}

impl Screen {
    fn label(self) -> &'static str {
        match self {
            Self::Care => "Smart Care",
            Self::Protection => "Protection",
            Self::Cleanup => "Cleanup",
            Self::Performance => "Performance",
            Self::Apps => "Applications",
            Self::Space => "Space",
        }
    }

    const ALL: [Screen; 6] = [
        Self::Care,
        Self::Protection,
        Self::Cleanup,
        Self::Performance,
        Self::Apps,
        Self::Space,
    ];
}

enum UiMsg {
    Scan(ScanEvent),
    Care(CareReport),
    Junk(Vec<JunkRoot>),
    Note(String),
    Agents(Vec<LaunchAgent>),
    Apps(Vec<InstalledApp>),
    Leftovers(Vec<LeftoverHit>),
    Sample(MachineSample),
    Space(SpaceReport),
}

fn main() -> eframe::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([900.0, 600.0]),
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
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 41, 48);
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
    note: String,
    last_path: String,
    scanned: u64,
    detection_count: u64,
    errors: u64,
    detections: Vec<Detection>,
    care: Option<CareReport>,
    junk: Vec<JunkRoot>,
    agents: Vec<LaunchAgent>,
    apps: Vec<InstalledApp>,
    leftovers: Vec<LeftoverHit>,
    selected_app: Option<usize>,
    sample: Option<MachineSample>,
    space: Option<SpaceReport>,
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
        let access = folder_access(std::path::Path::new("/Library/Application Support"));
        Self {
            runtime,
            screen: Screen::Care,
            root: home.display().to_string(),
            home,
            watch: "dmg,pkg".into(),
            status: "idle",
            note: String::new(),
            last_path: String::new(),
            scanned: 0,
            detection_count: 0,
            errors: 0,
            detections: Vec::new(),
            care: None,
            junk: Vec::new(),
            agents: Vec::new(),
            apps: Vec::new(),
            leftovers: Vec::new(),
            selected_app: None,
            sample: None,
            space: None,
            access,
            events: None,
            control: None,
            busy: false,
        }
    }

    fn arm(&mut self) -> mpsc::Sender<UiMsg> {
        let (tx, rx) = mpsc::channel(256);
        self.events = Some(rx);
        self.busy = true;
        self.note.clear();
        tx
    }

    fn start_scan(&mut self) {
        if self.busy {
            return;
        }
        self.detections.clear();
        self.scanned = 0;
        self.detection_count = 0;
        self.errors = 0;
        self.last_path.clear();
        let control = Arc::new(ExecutionControl::new());
        let detector = Arc::new(ExtensionWatchDetector::new(
            self.watch.split(',').map(str::trim).filter(|s| !s.is_empty()),
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
            let message = match move_root_to_trash(&home, &path) {
                Ok(()) => format!("Moved to Trash: {}", path.display()),
                Err(err) => err,
            };
            let _ = tx.send(UiMsg::Note(message)).await;
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
            let message = match kind {
                Tool::Dns => flush_dns_cache().unwrap_or_else(|err| err),
                Tool::SpotlightStatus => spotlight_status().unwrap_or_else(|err| err),
                Tool::Reindex => reindex_spotlight().unwrap_or_else(|err| err),
            };
            let _ = tx.send(UiMsg::Note(message)).await;
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
            self.note = "Select an app first.".into();
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

    fn start_space(&mut self) {
        if self.busy {
            return;
        }
        let root = PathBuf::from(self.root.trim());
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
                UiMsg::Note(message) => {
                    self.note = message;
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
                    self.finish();
                }
                UiMsg::Leftovers(hits) => {
                    self.leftovers = hits;
                    self.finish();
                }
                UiMsg::Sample(sample) => {
                    self.sample = Some(sample);
                    self.finish();
                }
                UiMsg::Space(report) => {
                    self.space = Some(report);
                    self.finish();
                }
            }
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
    }
}

enum Tool {
    Dns,
    SpotlightStatus,
    Reindex,
}

impl eframe::App for ScanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        let pulse = ctx.input(|i| i.time);
        let screen = self.screen;
        let status = self.status;
        let busy = self.busy;
        egui::SidePanel::left("nav")
            .exact_width(232.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(SURFACE).inner_margin(egui::Margin::same(22.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 7.0, LIME);
                    ui.label(RichText::new("SABLE").strong().size(15.0).color(IVORY));
                });
                ui.add_space(6.0);
                ui.label(RichText::new(format!("access {}", self.access.label())).size(12.0).color(MUTED));
                ui.add_space(22.0);
                for item in Screen::ALL {
                    let active = item == screen;
                    let fill = if active { Color32::from_rgb(38, 40, 32) } else { Color32::TRANSPARENT };
                    let text = if active { IVORY } else { MUTED };
                    let response = egui::Frame::none()
                        .fill(fill)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(item.label()).color(text).size(14.0));
                        })
                        .response
                        .interact(egui::Sense::click());
                    if response.clicked() {
                        self.screen = item;
                    }
                    ui.add_space(4.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    status_pill(ui, status, busy, pulse);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(INK).inner_margin(egui::Margin::same(28.0)))
            .show(ctx, |ui| match self.screen {
                Screen::Care => care_screen(ui, self),
                Screen::Protection => protection_screen(ui, self),
                Screen::Cleanup => cleanup_screen(ui, self),
                Screen::Performance => performance_screen(ui, self),
                Screen::Apps => apps_screen(ui, self),
                Screen::Space => space_screen(ui, self),
            });
    }
}

fn care_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Smart Care", "Runs cleanup, protection access, launch agents, and applications together.");
    if primary(ui, "Run care", !app.busy).clicked() {
        app.start_care();
    }
    ui.add_space(16.0);
    if let Some(report) = &app.care {
        for module in &report.modules {
            card(ui, |ui| {
                ui.label(RichText::new(&module.name).strong().color(IVORY));
                ui.label(RichText::new(&module.summary).color(MUTED));
            });
            ui.add_space(8.0);
        }
    } else {
        empty(ui, "No care run yet", "The four modules report here after a scan.");
    }
}

fn protection_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Protection", "Walk a folder and flag watched extensions. Files stay in place.");
    card(ui, |ui| {
        ui.label(RichText::new("DISK ACCESS").size(11.0).color(MUTED).strong());
        ui.label(RichText::new(app.access.label()).color(if app.access == AccessStatus::Granted { LIME } else { AMBER }));
        ui.label(RichText::new("A denied status blocks scans that need the whole disk. Detection stays a watch list, not a signature database.").color(MUTED).size(12.0));
    });
    ui.add_space(12.0);
    card(ui, |ui| {
        ui.label(RichText::new("SOURCE").size(11.0).color(MUTED).strong());
        field(ui, &mut app.root, !app.busy);
        ui.add_space(8.0);
        ui.label(RichText::new("WATCH").size(11.0).color(MUTED).strong());
        ui.add(egui::TextEdit::singleline(&mut app.watch).desired_width(220.0).interactive(!app.busy).margin(Vec2::new(10.0, 8.0)).font(TextStyle::Monospace));
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if primary(ui, "Start scan", !app.busy).clicked() {
                app.start_scan();
            }
            let paused = app.control.as_ref().is_some_and(|c| c.is_paused());
            if ghost(ui, if paused { "Resume" } else { "Pause" }, app.busy).clicked() {
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
            if ghost(ui, "Cancel", app.busy).clicked() {
                if let Some(control) = &app.control {
                    control.cancel();
                    app.status = "canceling";
                }
            }
        });
    });
    ui.add_space(12.0);
    ui.columns(3, |cols| {
        stat(&mut cols[0], "Scanned", app.scanned, IVORY);
        stat(&mut cols[1], "Hits", app.detection_count, LIME);
        stat(&mut cols[2], "Errors", app.errors, if app.errors > 0 { ROSE } else { IVORY });
    });
    ui.add_space(8.0);
    card(ui, |ui| {
        let text = if app.last_path.is_empty() { "Waiting for the first file".to_string() } else { app.last_path.clone() };
        ui.label(RichText::new(text).monospace().size(12.5).color(IVORY));
    });
    ui.add_space(8.0);
    if app.detections.is_empty() {
        empty(ui, "Nothing flagged", "Matching extensions appear here.");
    } else {
        scroll_rows(ui, app.detections.iter().map(|hit| {
            format!("{}   {}   .{}", hit.path.display(), format_bytes(hit.size), hit.extension)
        }));
    }
}

fn cleanup_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Cleanup", "Measures cache and log roots. Trash uses Finder and only those roots.");
    if primary(ui, "Survey", !app.busy).clicked() {
        app.start_junk();
    }
    if !app.note.is_empty() {
        ui.label(RichText::new(&app.note).color(AMBER));
    }
    ui.add_space(12.0);
    if app.junk.is_empty() {
        empty(ui, "No survey yet", "User caches, logs, Spotify cache, and Xcode derived data.");
        return;
    }
    let mut trash = None;
    for root in &app.junk {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&root.label).strong());
                    ui.label(RichText::new(root.path.display().to_string()).size(12.0).color(MUTED));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if root.exists && ghost(ui, "Trash", !app.busy).clicked() {
                        trash = Some(root.path.clone());
                    }
                    ui.label(RichText::new(if root.exists { format_bytes(root.bytes) } else { "missing".into() }).color(LIME));
                });
            });
        });
        ui.add_space(8.0);
    }
    if let Some(path) = trash {
        app.trash(path);
    }
}

fn performance_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Performance", "Launch agents, a user DNS flush, and Spotlight status.");
    ui.horizontal(|ui| {
        if primary(ui, "Load agents", !app.busy).clicked() {
            app.load_agents();
        }
        if ghost(ui, "Flush DNS", !app.busy).clicked() {
            app.run_tool(Tool::Dns);
        }
        if ghost(ui, "Spotlight", !app.busy).clicked() {
            app.run_tool(Tool::SpotlightStatus);
        }
        if ghost(ui, "Reindex", !app.busy).clicked() {
            app.run_tool(Tool::Reindex);
        }
        if ghost(ui, "Sample", !app.busy).clicked() {
            app.sample_machine();
        }
    });
    if let Some(sample) = app.sample {
        ui.add_space(8.0);
        ui.columns(2, |cols| {
            stat(&mut cols[0], "CPU", sample.cpu_percent.round() as u64, LIME);
            stat(&mut cols[1], "Memory MB", sample.used_memory / (1024 * 1024), IVORY);
        });
    }
    if !app.note.is_empty() {
        ui.add_space(8.0);
        card(ui, |ui| {
            ui.label(RichText::new(&app.note).color(IVORY));
        });
    }
    ui.add_space(12.0);
    if app.agents.is_empty() {
        empty(ui, "No agents loaded", "Reads Label from LaunchAgents plists.");
    } else {
        scroll_rows(ui, app.agents.iter().map(|agent| format!("{}   {}", agent.label, agent.path.display())));
    }
}

fn apps_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Applications", "Bundle name and identifier from /Applications and your Applications folder.");
    ui.horizontal(|ui| {
        if primary(ui, "Load apps", !app.busy).clicked() {
            app.load_apps();
        }
        if ghost(ui, "Survey leftovers", !app.busy).clicked() {
            app.survey_selected();
        }
    });
    if !app.note.is_empty() {
        ui.label(RichText::new(&app.note).color(AMBER));
    }
    ui.add_space(12.0);
    if app.apps.is_empty() {
        empty(ui, "No apps loaded", "This list does not uninstall anything.");
    } else {
        let mut pick = None;
        card(ui, |ui| {
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for (index, item) in app.apps.iter().enumerate() {
                    let selected = app.selected_app == Some(index);
                    let text = format!("{}   {}", item.name, item.bundle_id);
                    if ui.selectable_label(selected, RichText::new(text).monospace().size(12.5)).clicked() {
                        pick = Some(index);
                    }
                }
            });
        });
        if let Some(index) = pick {
            app.selected_app = Some(index);
            app.leftovers.clear();
        }
    }
    if !app.leftovers.is_empty() {
        ui.add_space(8.0);
        scroll_rows(ui, app.leftovers.iter().map(|hit| format!("{}   {}", hit.root, hit.path.display())));
    }
}

fn space_screen(ui: &mut egui::Ui, app: &mut ScanApp) {
    title(ui, "Space", "Sizes the immediate children of a folder, four at a time.");
    card(ui, |ui| {
        field(ui, &mut app.root, !app.busy);
        ui.add_space(8.0);
        if primary(ui, "Measure", !app.busy).clicked() {
            app.start_space();
        }
    });
    ui.add_space(12.0);
    if let Some(report) = &app.space {
        ui.label(RichText::new(format!("Total {}", format_bytes(report.total))).color(LIME));
        ui.add_space(8.0);
        card(ui, |ui| {
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                let total = report.total.max(1) as f32;
                for node in &report.children {
                    let share = node.bytes as f32 / total;
                    let name = node.path.file_name().and_then(|name| name.to_str()).unwrap_or("item");
                    ui.add(egui::ProgressBar::new(share).text(format!("{name}  {}", format_bytes(node.bytes))).fill(LIME));
                    ui.add_space(6.0);
                }
            });
        });
    } else {
        empty(ui, "No measurement yet", "Pick a folder and measure its children.");
    }
}

fn title(ui: &mut egui::Ui, heading: &str, subtitle: &str) {
    ui.label(RichText::new(heading).size(32.0).strong().color(IVORY));
    ui.add_space(4.0);
    ui.label(RichText::new(subtitle).color(MUTED).size(15.0));
    ui.add_space(16.0);
}

fn empty(ui: &mut egui::Ui, heading: &str, body: &str) {
    card(ui, |ui| {
        ui.add_space(18.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(heading).size(16.0).color(IVORY));
            ui.add_space(4.0);
            ui.label(RichText::new(body).color(MUTED));
        });
        ui.add_space(18.0);
    });
}

fn scroll_rows(ui: &mut egui::Ui, rows: impl Iterator<Item = String>) {
    card(ui, |ui| {
        egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
            for row in rows {
                ui.add_space(4.0);
                ui.label(RichText::new(row).monospace().size(12.5).color(IVORY));
            }
        });
    });
}

fn stat(ui: &mut egui::Ui, label: &str, value: u64, value_color: Color32) {
    card(ui, |ui| {
        ui.label(RichText::new(label.to_uppercase()).size(11.0).color(MUTED).strong());
        ui.label(RichText::new(format_count(value)).size(28.0).strong().color(value_color));
    });
}

fn status_pill(ui: &mut egui::Ui, status: &str, busy: bool, time: f64) {
    let color = match status {
        "scanning" => LIME,
        "paused" => AMBER,
        "canceling" | "cleaning" => ROSE,
        _ => MUTED,
    };
    let alpha = if busy { 0.45 + 0.55 * (time * 3.0).sin().abs() as f32 } else { 1.0 };
    let dot = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (alpha * 255.0) as u8);
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(Rounding::same(20.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, dot);
                ui.label(RichText::new(status).size(12.0).color(IVORY));
            });
        });
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(Rounding::same(14.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, body);
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

fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(LIME_INK).strong())
            .fill(LIME)
            .rounding(Rounding::same(10.0))
            .min_size(Vec2::new(120.0, 36.0)),
    )
}

fn ghost(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(IVORY))
            .fill(SURFACE)
            .stroke(Stroke::new(1.0, LINE))
            .rounding(Rounding::same(10.0))
            .min_size(Vec2::new(96.0, 36.0)),
    )
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
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
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
