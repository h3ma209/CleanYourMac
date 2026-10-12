mod screens;
mod stats;
mod theme;

use std::sync::Arc;

use eframe::egui;
use engine::{run_scan, ExecutionControl, ExtensionWatchDetector, RemoveTask, ScanConfig, ScanEvent};
use tokio::sync::mpsc;

use screens::{HomeAction, Module, ReviewItem};
use stats::Glance;
use theme::Palette;

fn main() -> eframe::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sable",
        options,
        Box::new(|cc| {
            theme::apply_type(&cc.egui_ctx);
            Ok(Box::new(ScanApp::new(runtime)))
        }),
    )
}

struct ScanApp {
    runtime: tokio::runtime::Runtime,
    light: bool,
    module: Module,
    root: String,
    watch: String,
    scanning: bool,
    done: bool,
    status: String,
    scanned: u64,
    errors: u64,
    items: Vec<ReviewItem>,
    note: String,
    events: Option<mpsc::Receiver<ScanEvent>>,
    control: Option<Arc<ExecutionControl>>,
    glance: Glance,
}

impl ScanApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            runtime,
            light: false,
            module: Module::SmartCare,
            root: std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            watch: "dmg,pkg".into(),
            scanning: false,
            done: false,
            status: "One scan, then you decide what stays.".into(),
            scanned: 0,
            errors: 0,
            items: Vec::new(),
            note: String::new(),
            events: None,
            control: None,
            glance: Glance::sample(),
        }
    }

    fn palette(&self) -> Palette {
        if self.light {
            Palette::light()
        } else {
            Palette::dark()
        }
    }

    fn start(&mut self) {
        if self.scanning {
            return;
        }
        self.items.clear();
        self.note.clear();
        self.scanned = 0;
        self.errors = 0;
        self.done = false;
        self.status = "Looking through your files…".into();
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
        self.scanning = true;
    }

    fn drain(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.events.as_mut() else {
            return;
        };
        while let Ok(event) = rx.try_recv() {
            match event {
                ScanEvent::Progress { path } => {
                    self.status = format!("Checking {}", file_name(&path.display().to_string()));
                }
                ScanEvent::Checked { .. } => {
                    self.scanned += 1;
                }
                ScanEvent::Detection(hit) => {
                    self.scanned += 1;
                    let path = hit.path.display().to_string();
                    self.items.push(ReviewItem {
                        name: file_name(&path),
                        path,
                        size: hit.size,
                        selected: true,
                    });
                }
                ScanEvent::Error { .. } => {
                    self.errors += 1;
                }
                ScanEvent::Finished {
                    scanned,
                    detections,
                    errors,
                } => {
                    self.scanned = scanned;
                    self.errors = errors;
                    self.scanning = false;
                    self.done = true;
                    self.control = None;
                    self.status = if detections == 0 {
                        "Your Mac is in great shape".into()
                    } else {
                        "A few things could use a tidy. You’re in control.".into()
                    };
                }
            }
            ctx.request_repaint();
        }
        if self.scanning {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        }
    }

    fn cleanup_line(&self) -> String {
        if self.items.is_empty() {
            "Nothing to clean".into()
        } else {
            let bytes: u64 = self.items.iter().map(|i| i.size).sum();
            format!("{} items · {}", self.items.len(), theme::format_bytes(bytes))
        }
    }
}

impl eframe::App for ScanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain(ctx);
        let time = ctx.input(|i| i.time);
        let palette = self.palette();
        let light = self.light;
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                screens::paint_full(ui, light, time);
                ui.add_space(14.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(16.0);
                    let picked = screens::sidebar(ui, palette, self.module, &mut self.light);
                    if let Some(module) = picked {
                        self.module = module;
                    }
                    ui.add_space(18.0);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 16.0);
                        screens::menu_widget(
                            ui,
                            palette,
                            &self.glance.storage,
                            &self.glance.memory,
                            &self.glance.battery,
                        );
                        ui.add_space(8.0);
                        match self.module {
                            Module::SmartCare => {
                                let cleanup = self.cleanup_line();
                                let protection = if self.done && self.errors == 0 {
                                    "No threats in this pass"
                                } else if self.errors > 0 {
                                    "A couple of files couldn’t be read"
                                } else {
                                    "Scan to check in"
                                };
                                let performance = if self.done {
                                    "Plenty of room to breathe"
                                } else {
                                    "Scan to see how things feel"
                                };
                                match screens::home(
                                    ui,
                                    palette,
                                    time,
                                    self.scanning,
                                    self.done,
                                    &self.status,
                                    &cleanup,
                                    protection,
                                    performance,
                                ) {
                                    HomeAction::Scan => self.start(),
                                    HomeAction::OpenCleanup => self.module = Module::Cleanup,
                                    HomeAction::None => {}
                                }
                            }
                            Module::Cleanup => {
                                if screens::cleanup(ui, palette, &mut self.items, &self.note) {
                                    let bytes: u64 = self
                                        .items
                                        .iter()
                                        .filter(|i| i.selected)
                                        .map(|i| i.size)
                                        .sum();
                                    let selected: Vec<_> = self
                                        .items
                                        .iter()
                                        .filter(|i| i.selected)
                                        .map(|i| engine::Detection {
                                            path: i.path.clone().into(),
                                            size: i.size,
                                            extension: String::new(),
                                            note: "review".into(),
                                        })
                                        .collect();
                                    let plan = RemoveTask::plan(&selected);
                                    self.note = format!(
                                        "Reviewed {} items ({}). They’re still on your Mac.",
                                        plan.paths.len(),
                                        theme::format_bytes(bytes)
                                    );
                                }
                            }
                            other => screens::quiet_module(ui, palette, other),
                        }
                    });
                });
            });
    }
}

fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("File")
        .to_string()
}
