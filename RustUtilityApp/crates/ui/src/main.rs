use std::sync::Arc;

use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Rounding, Stroke, TextStyle, Vec2};
use engine::{
    run_scan, Detection, ExecutionControl, ExtensionWatchDetector, ScanConfig, ScanEvent,
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

fn main() -> eframe::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([860.0, 560.0]),
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
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(26.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

struct ScanApp {
    runtime: tokio::runtime::Runtime,
    root: String,
    watch: String,
    status: &'static str,
    last_path: String,
    scanned: u64,
    detection_count: u64,
    errors: u64,
    detections: Vec<Detection>,
    events: Option<mpsc::Receiver<ScanEvent>>,
    control: Option<Arc<ExecutionControl>>,
    busy: bool,
}

impl ScanApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            runtime,
            root: std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            watch: "dmg,pkg".into(),
            status: "idle",
            last_path: String::new(),
            scanned: 0,
            detection_count: 0,
            errors: 0,
            detections: Vec::new(),
            events: None,
            control: None,
            busy: false,
        }
    }

    fn start(&mut self) {
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
        let (tx, rx) = mpsc::channel(256);
        let control_task = Arc::clone(&control);
        self.runtime.spawn(async move {
            run_scan(config, detector, control_task, tx).await;
        });
        self.control = Some(control);
        self.events = Some(rx);
        self.busy = true;
        self.status = "scanning";
    }

    fn drain(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.events.as_mut() else {
            return;
        };
        while let Ok(event) = rx.try_recv() {
            match event {
                ScanEvent::Progress { path } => {
                    self.last_path = path.display().to_string();
                }
                ScanEvent::Checked { path, .. } => {
                    self.scanned += 1;
                    self.last_path = path.display().to_string();
                }
                ScanEvent::Detection(hit) => {
                    self.scanned += 1;
                    self.detection_count += 1;
                    self.last_path = hit.path.display().to_string();
                    self.detections.push(hit);
                }
                ScanEvent::Error { path, message } => {
                    self.errors += 1;
                    self.last_path = format!("{} ({message})", path.display());
                }
                ScanEvent::Finished {
                    scanned,
                    detections,
                    errors,
                } => {
                    self.scanned = scanned;
                    self.detection_count = detections;
                    self.errors = errors;
                    self.busy = false;
                    self.status = "idle";
                    self.control = None;
                }
            }
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
    }
}

impl eframe::App for ScanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        let pulse = ctx.input(|i| i.time);
        sidebar(ctx, self.status, self.busy, pulse);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(INK)
                    .inner_margin(egui::Margin::same(28.0)),
            )
            .show(ctx, |ui| {
                header(ui);
                ui.add_space(18.0);
                controls(ui, self);
                ui.add_space(16.0);
                metrics(ui, self);
                ui.add_space(12.0);
                current_path(ui, self);
                ui.add_space(16.0);
                results(ui, self);
            });
    }
}

fn sidebar(ctx: &egui::Context, status: &str, busy: bool, time: f64) {
    egui::SidePanel::left("nav")
        .exact_width(232.0)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(SURFACE)
                .inner_margin(egui::Margin::same(22.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 7.0, LIME);
                ui.add_space(2.0);
                ui.label(RichText::new("SABLE").strong().size(15.0).color(IVORY));
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Directory review").size(12.0).color(MUTED));
            ui.add_space(28.0);
            nav_row(ui, "Scan", true);
            nav_row(ui, "Review", false);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Hits stay on disk").size(11.0).color(MUTED));
                ui.add_space(10.0);
                status_pill(ui, status, busy, time);
            });
        });
}

fn nav_row(ui: &mut egui::Ui, label: &str, active: bool) {
    let fill = if active {
        Color32::from_rgb(38, 40, 32)
    } else {
        Color32::TRANSPARENT
    };
    let text = if active { IVORY } else { MUTED };
    egui::Frame::none()
        .fill(fill)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(label).color(text).size(14.0));
        });
    ui.add_space(4.0);
}

fn status_pill(ui: &mut egui::Ui, status: &str, busy: bool, time: f64) {
    let color = match status {
        "scanning" => LIME,
        "paused" => AMBER,
        "canceling" => ROSE,
        _ => MUTED,
    };
    let alpha = if busy {
        0.45 + 0.55 * (time * 3.0).sin().abs() as f32
    } else {
        1.0
    };
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

fn header(ui: &mut egui::Ui) {
    ui.label(
        RichText::new("Scan a folder")
            .size(32.0)
            .strong()
            .color(IVORY),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new("Walk the tree, flag watched extensions, leave every file where it is.")
            .color(MUTED)
            .size(15.0),
    );
}

fn controls(ui: &mut egui::Ui, app: &mut ScanApp) {
    card(ui, |ui| {
        ui.label(RichText::new("SOURCE").size(11.0).color(MUTED).strong());
        ui.add_space(4.0);
        field(ui, &mut app.root, !app.busy);
        ui.add_space(12.0);
        ui.label(RichText::new("WATCH").size(11.0).color(MUTED).strong());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.watch)
                    .desired_width(220.0)
                    .interactive(!app.busy)
                    .margin(Vec2::new(10.0, 8.0))
                    .font(TextStyle::Monospace),
            );
            ui.label(RichText::new("comma separated, no dots").size(12.0).color(MUTED));
        });
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if primary(ui, "Start scan", !app.busy).clicked() {
                app.start();
            }
            let paused = app.control.as_ref().is_some_and(|c| c.is_paused());
            let pause_label = if paused { "Resume" } else { "Pause" };
            if ghost(ui, pause_label, app.busy).clicked() {
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
}

fn metrics(ui: &mut egui::Ui, app: &ScanApp) {
    ui.columns(3, |cols| {
        stat(&mut cols[0], "Scanned", app.scanned, IVORY);
        stat(&mut cols[1], "Hits", app.detection_count, LIME);
        stat(&mut cols[2], "Errors", app.errors, if app.errors > 0 { ROSE } else { IVORY });
    });
}

fn stat(ui: &mut egui::Ui, label: &str, value: u64, value_color: Color32) {
    card(ui, |ui| {
        ui.label(RichText::new(label.to_uppercase()).size(11.0).color(MUTED).strong());
        ui.add_space(6.0);
        ui.label(RichText::new(format_count(value)).size(28.0).strong().color(value_color));
    });
}

fn current_path(ui: &mut egui::Ui, app: &ScanApp) {
    let text = if app.last_path.is_empty() {
        "Waiting for the first file".to_string()
    } else {
        app.last_path.clone()
    };
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("NOW").size(11.0).color(MUTED).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(text).monospace().size(12.5).color(IVORY));
        });
    });
}

fn results(ui: &mut egui::Ui, app: &ScanApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Review").size(18.0).strong().color(IVORY));
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("{} listed", app.detections.len()))
                .size(12.0)
                .color(MUTED),
        );
    });
    ui.add_space(8.0);
    card(ui, |ui| {
        if app.detections.is_empty() {
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Nothing flagged").size(16.0).color(IVORY));
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Start a scan. Matching extensions appear here.")
                        .color(MUTED),
                );
            });
            ui.add_space(28.0);
            return;
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("PATH").size(11.0).color(MUTED).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("SIZE").size(11.0).color(MUTED).strong());
            });
        });
        ui.add_space(6.0);
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(280.0)
            .show(ui, |ui| {
                for hit in &app.detections {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let name = hit
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file");
                        ui.label(RichText::new(name).strong().color(IVORY));
                        ui.label(
                            RichText::new(format!(".{}", hit.extension))
                                .size(12.0)
                                .color(LIME_INK)
                                .background_color(LIME),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format_bytes(hit.size))
                                    .monospace()
                                    .color(MUTED),
                            );
                        });
                    });
                    ui.label(
                        RichText::new(hit.path.display().to_string())
                            .size(12.0)
                            .color(MUTED),
                    );
                    ui.add_space(6.0);
                }
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
