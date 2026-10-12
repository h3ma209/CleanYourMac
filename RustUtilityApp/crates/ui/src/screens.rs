use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};

use crate::theme::{self, Palette, RADIUS_CARD, RADIUS_PILL, SIDEBAR_W};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Module {
    SmartCare,
    Cleanup,
    Protection,
    Performance,
    Applications,
    Clutter,
}

impl Module {
    pub const ALL: [Module; 6] = [
        Module::SmartCare,
        Module::Cleanup,
        Module::Protection,
        Module::Performance,
        Module::Applications,
        Module::Clutter,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Module::SmartCare => "Smart Care",
            Module::Cleanup => "Cleanup",
            Module::Protection => "Protection",
            Module::Performance => "Performance",
            Module::Applications => "Applications",
            Module::Clutter => "My Clutter",
        }
    }
}

pub enum HomeAction {
    None,
    Scan,
    OpenCleanup,
}

pub struct ReviewItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub selected: bool,
}

pub fn sidebar(ui: &mut egui::Ui, palette: Palette, active: Module, light: &mut bool) -> Option<Module> {
    let mut picked = None;
    theme::glass(ui, palette, 24.0, 16.0, |ui| {
        ui.set_min_width(SIDEBAR_W - 8.0);
        ui.set_height(ui.available_height());
        ui.horizontal(|ui| {
            orb(ui, palette.accent, 14.0, 0.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Sable").strong().size(16.0).color(palette.text));
                ui.label(RichText::new("Care for your Mac").size(12.0).color(palette.muted));
            });
        });
        ui.add_space(22.0);
        for module in Module::ALL {
            if nav_item(ui, palette, module, module == active).clicked() {
                picked = Some(module);
            }
            ui.add_space(4.0);
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let label = if *light { "Dark look" } else { "Light look" };
            if theme::quiet(ui, palette, label).clicked() {
                *light = !*light;
            }
        });
    });
    picked
}

fn nav_item(ui: &mut egui::Ui, palette: Palette, module: Module, active: bool) -> egui::Response {
    let fill = if active {
        if palette.light {
            Color32::from_rgba_unmultiplied(255, 255, 255, 210)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 36)
        }
    } else {
        Color32::TRANSPARENT
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::click());
    if active || response.hovered() {
        ui.painter().rect_filled(rect, RADIUS_PILL, fill);
    }
    let icon_center = Pos2::new(rect.left() + 22.0, rect.center().y);
    ui.painter()
        .circle_filled(icon_center, 9.0, icon_color(module, palette));
    ui.painter().circle_stroke(
        icon_center,
        9.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
    );
    ui.painter().text(
        Pos2::new(rect.left() + 42.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        module.label(),
        egui::FontId::new(14.5, egui::FontFamily::Proportional),
        if active { palette.text } else { palette.muted },
    );
    response
}

fn icon_color(module: Module, palette: Palette) -> Color32 {
    match module {
        Module::SmartCare => palette.accent,
        Module::Cleanup => Color32::from_rgb(120, 168, 255),
        Module::Protection => palette.good,
        Module::Performance => Color32::from_rgb(255, 196, 92),
        Module::Applications => Color32::from_rgb(186, 140, 255),
        Module::Clutter => Color32::from_rgb(255, 140, 176),
    }
}

pub fn menu_widget(ui: &mut egui::Ui, palette: Palette, storage: &str, memory: &str, battery: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        chip(ui, palette, battery);
        chip(ui, palette, memory);
        chip(ui, palette, storage);
    });
}

fn chip(ui: &mut egui::Ui, palette: Palette, text: &str) {
    egui::Frame::none()
        .fill(palette.glass)
        .stroke(Stroke::new(1.0, palette.stroke))
        .rounding(egui::Rounding::same(RADIUS_PILL))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).color(palette.text));
        });
}

pub fn hero(ui: &mut egui::Ui, palette: Palette, time: f64, scanning: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(320.0, 230.0), Sense::hover());
    let bob = (time * 1.3).sin() as f32 * if scanning { 10.0 } else { 6.0 };
    let pulse = if scanning {
        1.0 + 0.06 * (time * 3.2).sin() as f32
    } else {
        1.0
    };
    let center = Pos2::new(rect.center().x, rect.center().y + bob * 0.3);
    let painter = ui.painter();
    painter.circle_filled(
        Pos2::new(center.x, center.y + 70.0),
        70.0 * pulse,
        Color32::from_rgba_unmultiplied(0, 0, 0, if palette.light { 18 } else { 50 }),
    );
    orb_at(painter, Pos2::new(center.x - 78.0, center.y + 18.0 - bob), 36.0, Color32::from_rgb(126, 168, 255), pulse);
    orb_at(painter, Pos2::new(center.x + 82.0, center.y + 8.0 + bob), 32.0, palette.good, pulse);
    orb_at(painter, center, 58.0 * pulse, palette.accent, pulse);
}

fn orb(ui: &mut egui::Ui, color: Color32, radius: f32, _time: f64) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(radius * 2.0 + 4.0), Sense::hover());
    orb_at(ui.painter(), rect.center(), radius, color, 1.0);
}

fn orb_at(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, pulse: f32) {
    let r = radius * pulse;
    painter.circle_filled(center, r, color);
    painter.circle_filled(
        Pos2::new(center.x - r * 0.28, center.y - r * 0.32),
        r * 0.28,
        Color32::from_rgba_unmultiplied(255, 255, 255, 150),
    );
    painter.circle_stroke(
        center,
        r,
        Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 255, 255, 80)),
    );
}

pub fn home(
    ui: &mut egui::Ui,
    palette: Palette,
    time: f64,
    scanning: bool,
    done: bool,
    status: &str,
    cleanup_line: &str,
    protection_line: &str,
    performance_line: &str,
) -> HomeAction {
    let mut action = HomeAction::None;
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        hero(ui, palette, time, scanning);
        ui.add_space(8.0);
        let title = if done && !scanning {
            "Your Mac is in great shape"
        } else if scanning {
            "Taking a gentle look"
        } else {
            "Ready when you are"
        };
        ui.label(RichText::new(title).size(34.0).strong().color(palette.text));
        ui.add_space(4.0);
        let status_color = if status.contains("couldn") {
            palette.warn
        } else {
            palette.muted
        };
        ui.label(RichText::new(status).size(15.0).color(status_color));
        ui.add_space(16.0);
        if theme::pill(ui, palette, if scanning { "Scanning…" } else { "Scan" }, !scanning).clicked()
        {
            action = HomeAction::Scan;
        }
    });
    if done && !scanning {
        ui.add_space(22.0);
        ui.columns(3, |cols| {
            if summary(&mut cols[0], palette, "Cleanup", cleanup_line).clicked() {
                action = HomeAction::OpenCleanup;
            }
            let _ = summary(&mut cols[1], palette, "Protection", protection_line);
            let _ = summary(&mut cols[2], palette, "Performance", performance_line);
        });
        ui.add_space(18.0);
        assistant(ui, palette);
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            if theme::pill(ui, palette, "Run", true).clicked() {
                action = HomeAction::OpenCleanup;
            }
        });
    }
    action
}

fn summary(ui: &mut egui::Ui, palette: Palette, title: &str, line: &str) -> egui::Response {
    let mut review = None;
    theme::glass(ui, palette, RADIUS_CARD, 16.0, |ui| {
        ui.set_min_height(108.0);
        ui.label(RichText::new(title).strong().size(16.0).color(palette.text));
        ui.add_space(4.0);
        ui.label(RichText::new(line).size(13.5).color(palette.muted));
        ui.add_space(12.0);
        review = Some(theme::quiet(ui, palette, "Review"));
    });
    review.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
}

fn assistant(ui: &mut egui::Ui, palette: Palette) {
    theme::glass(ui, palette, RADIUS_CARD, 16.0, |ui| {
        ui.horizontal(|ui| {
            orb(ui, palette.accent, 16.0, 0.0);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("A small note").size(12.0).color(palette.muted));
                ui.label(
                    RichText::new("I can walk you through cleanup whenever you want. You choose what goes.")
                        .size(14.5)
                        .color(palette.text),
                );
            });
        });
    });
}

pub fn cleanup(
    ui: &mut egui::Ui,
    palette: Palette,
    items: &mut [ReviewItem],
    note: &str,
) -> bool {
    let mut clean = false;
    ui.label(RichText::new("Cleanup").size(34.0).strong().color(palette.text));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Review first. Nothing is removed until you say so.")
            .size(15.0)
            .color(palette.muted),
    );
    ui.add_space(16.0);
    if items.is_empty() {
        theme::glass(ui, palette, RADIUS_CARD, 28.0, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(RichText::new("Nothing to tidy").size(20.0).color(palette.text));
                ui.label(
                    RichText::new("Run a scan from Smart Care and I’ll line up anything worth a look.")
                        .color(palette.muted),
                );
                ui.add_space(12.0);
            });
        });
        return false;
    }
    let selected: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
    let count = items.iter().filter(|i| i.selected).count();
    egui::ScrollArea::vertical()
        .max_height(340.0)
        .show(ui, |ui| {
            for item in items.iter_mut() {
                theme::glass(ui, palette, 16.0, 12.0, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut item.selected, "");
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&item.name).strong().color(palette.text));
                            ui.label(RichText::new(&item.path).size(12.0).color(palette.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(theme::format_bytes(item.size))
                                    .color(palette.text),
                            );
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{count} selected · {}", theme::format_bytes(selected)))
                .color(palette.muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::pill(ui, palette, "Remove selected", count > 0).clicked() {
                clean = true;
            }
        });
    });
    if !note.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new(note).color(palette.good));
    }
    clean
}

pub fn quiet_module(ui: &mut egui::Ui, palette: Palette, module: Module) {
    ui.label(
        RichText::new(module.label())
            .size(34.0)
            .strong()
            .color(palette.text),
    );
    ui.add_space(12.0);
    theme::glass(ui, palette, RADIUS_CARD, 24.0, |ui| {
        ui.add_space(8.0);
        let (title, line) = match module {
            Module::Protection => ("All clear", "Nothing in this pass needs your attention."),
            Module::Performance => ("Feeling light", "Memory and startup look comfortable."),
            Module::Applications => ("Apps look tidy", "No leftover installers turned up."),
            Module::Clutter => ("Clutter can wait", "Downloads and duplicates look calm."),
            _ => ("", ""),
        };
        ui.label(RichText::new(title).size(22.0).strong().color(palette.text));
        ui.label(RichText::new(line).color(palette.muted));
        ui.add_space(8.0);
    });
}

pub fn paint_full(ui: &egui::Ui, light: bool, time: f64) {
    let rect = ui.max_rect();
    // Expand a hair so the gradient covers the window edge.
    let rect = Rect::from_min_max(rect.min, rect.max);
    theme::paint_backdrop(ui.painter(), rect, light, time);
}
