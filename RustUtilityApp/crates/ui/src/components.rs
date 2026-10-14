//! Shared buttons, chips, badges, surfaces, and list action columns.

use eframe::egui::{self, Align, Align2, Color32, Layout, RichText, Rect, Sense, Shape, Stroke, Ui, Vec2};

use crate::icons::{icon_with_label, paint_screen_icon, UiIcon};
use crate::tokens::*;

pub const ACTION_COL_W: f32 = 128.0;
pub const SIZE_COL_W: f32 = 88.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnKind {
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

pub fn surface_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(SURFACE_CARD)
        .stroke(stroke(SURFACE_LINE))
        .rounding(r_md())
        .inner_margin(m_all(S16))
}

pub fn paint_app_background(painter: &egui::Painter, rect: Rect) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let panel = rect.shrink(S8);
    painter.rect_filled(panel, r_sm(), BG);
}

pub fn accent_for_screen(_screen: crate::Screen) -> Color32 {
    ACCENT
}

#[derive(Clone, Copy)]
pub enum SafetyKind {
    ReadOnly,
    Trash,
    Admin,
}

pub fn safety_badge(ui: &mut Ui, kind: SafetyKind) {
    let (icon, text, bg, fg) = match kind {
        SafetyKind::ReadOnly => (
            UiIcon::Eye,
            "Read-only — nothing on disk is changed",
            mix(SUCCESS, SURFACE_CARD, 35),
            SUCCESS,
        ),
        SafetyKind::Trash => (
            UiIcon::Trash,
            "Moves to Trash — you can undo this",
            mix(WARN, SURFACE_CARD, 35),
            WARN,
        ),
        SafetyKind::Admin => (
            UiIcon::Lock,
            "May ask for your Mac password",
            mix(DANGER, SURFACE_CARD, 35),
            DANGER,
        ),
    };
    egui::Frame::none()
        .fill(bg)
        .stroke(stroke(with_alpha(fg, 180)))
        .rounding(r_sm())
        .inner_margin(m_xy(S12, S8))
        .show(ui, |ui| {
            icon_with_label(ui, icon, text, fg);
        });
}

pub fn compact_page_header(
    ui: &mut Ui,
    screen: crate::Screen,
    title_size: f32,
    subtitle: &str,
    safety: SafetyKind,
    accent: Color32,
    header_action: Option<(&str, bool)>,
) -> bool {
    let mut action = false;
    surface_frame().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(S32), Sense::hover());
            ui.painter().rect_filled(icon_rect, r_sm(), with_alpha(accent, 48));
            let glyph = Rect::from_center_size(icon_rect.center(), Vec2::splat(18.0));
            paint_screen_icon(ui.painter(), screen, glyph, accent);
            ui.add_space(S12);
            ui.vertical(|ui| {
                ui.set_max_width((ui.available_width() - 260.0).max(200.0));
                ui.label(
                    RichText::new(screen.label())
                        .font(crate::icons::heading_font(title_size))
                        .color(WHITE),
                );
                ui.add_space(S4);
                ui.label(
                    RichText::new(subtitle)
                        .color(TEXT_SECONDARY)
                        .size(SMALL)
                        .line_height(Some(lh(SMALL))),
                );
                ui.add_space(S8);
                safety_badge(ui, safety);
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some((label, enabled)) = header_action {
                    if button(ui, label, label, BtnKind::Primary, enabled, accent).clicked() {
                        action = true;
                    }
                }
            });
        });
    });
    ui.add_space(S12);
    action
}

pub fn button(ui: &mut Ui, salt: &str, label: &str, kind: BtnKind, enabled: bool, accent: Color32) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_semi(BODY), button_fg(kind, enabled));
    let w = (galley.size().x + BTN_PAD_X * 2.0).max(BTN_MIN_W);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, BTN_MIN_H), if enabled { Sense::click() } else { Sense::hover() });
    paint_button(ui, rect, &response, label, kind, enabled, accent);
    let _ = salt;
    response
}

pub fn button_sized(
    ui: &mut Ui,
    salt: &str,
    label: &str,
    kind: BtnKind,
    enabled: bool,
    width: f32,
    accent: Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, BTN_MIN_H), if enabled { Sense::click() } else { Sense::hover() });
    paint_button(ui, rect, &response, label, kind, enabled, accent);
    let _ = salt;
    response
}

fn button_fg(kind: BtnKind, enabled: bool) -> Color32 {
    if !enabled {
        return TEXT_SECONDARY;
    }
    match kind {
        BtnKind::Primary => INK,
        BtnKind::Secondary | BtnKind::Destructive | BtnKind::Ghost => WHITE,
    }
}

fn paint_button(ui: &mut Ui, rect: Rect, response: &egui::Response, label: &str, kind: BtnKind, enabled: bool, accent: Color32) {
    let pressed = enabled && response.is_pointer_button_down_on();
    let hot = enabled && (response.hovered() || pressed);
    let (fill, border, fg) = button_colors(kind, enabled, hot, pressed, accent);
    ui.painter().rect_filled(rect, r_pill(), fill);
    if border.width > 0.0 {
        ui.painter().rect_stroke(rect, r_pill(), border);
    }
    if enabled && response.has_focus() {
        ui.painter().rect_stroke(rect.expand(S4), r_pill(), focus_stroke());
    }
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_semi(BODY), fg);
    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

fn button_colors(kind: BtnKind, enabled: bool, hot: bool, pressed: bool, accent: Color32) -> (Color32, Stroke, Color32) {
    if !enabled {
        return (BTN_DISABLED, no_stroke(), TEXT_SECONDARY);
    }
    match kind {
        BtnKind::Primary => {
            let fill = if pressed { BTN_PRESSED } else if hot { BTN_HOVER } else { ACCENT };
            (fill, no_stroke(), INK)
        }
        BtnKind::Secondary => {
            let fill = if pressed {
                with_alpha(accent, 180)
            } else if hot {
                with_alpha(accent, 120)
            } else {
                with_alpha(accent, 80)
            };
            (fill, stroke(SURFACE_LINE), WHITE)
        }
        BtnKind::Destructive => {
            let fill = if pressed {
                mix(DANGER, INK, 20)
            } else if hot {
                DANGER
            } else {
                mix(DANGER, SURFACE_CARD, 30)
            };
            (fill, stroke(with_alpha(DANGER, 200)), WHITE)
        }
        BtnKind::Ghost => {
            let fill = if pressed {
                with_alpha(WHITE, 64)
            } else if hot {
                with_alpha(WHITE, 40)
            } else {
                with_alpha(WHITE, 16)
            };
            (fill, stroke(SURFACE_LINE), WHITE)
        }
    }
}

pub fn chip(ui: &mut Ui, salt: &str, label: &str, active: bool, accent: Color32) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_semi(SMALL), if active { INK } else { WHITE });
    let w = galley.size().x + BTN_PAD_X * 2.0 + if active { S12 } else { 0.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, CHIP_H), Sense::click());
    let hot = response.hovered() || response.is_pointer_button_down_on();
    let (fill, border, fg) = if active {
        (ACCENT, if response.has_focus() { focus_stroke() } else { no_stroke() }, INK)
    } else if hot {
        (with_alpha(accent, 80), stroke(SURFACE_LINE), WHITE)
    } else {
        (with_alpha(WHITE, 12), stroke(SURFACE_LINE), WHITE)
    };
    ui.painter().rect_filled(rect, r_pill(), fill);
    if border.width > 0.0 {
        ui.painter().rect_stroke(rect, r_pill(), border);
    }
    if active {
        ui.painter().text(
            rect.left_center() + Vec2::new(S12, 0.0),
            Align2::CENTER_CENTER,
            "✓",
            font_semi(CAPTION),
            INK,
        );
    }
    ui.painter().text(
        rect.center() + Vec2::new(if active { S4 } else { 0.0 }, 0.0),
        Align2::CENTER_CENTER,
        label,
        font_semi(SMALL),
        fg,
    );
    let _ = salt;
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn tab_bar(ui: &mut Ui, id: &str, labels: &[&str], selected: &mut usize, accent: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = S8;
        for (i, label) in labels.iter().enumerate() {
            if chip(ui, &format!("{id}_{i}"), label, *selected == i, accent).clicked() {
                *selected = i;
            }
        }
    });
    ui.add_space(S8);
}

pub fn label_small(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(CAPTION)
            .line_height(Some(lh(CAPTION)))
            .color(TEXT_SECONDARY)
            .strong(),
    );
}

pub fn list_actions(
    ui: &mut Ui,
    size_label: &str,
    show_enabled: bool,
    trash_enabled: bool,
    accent: Color32,
    salt: &str,
    on_show: &mut bool,
    on_trash: &mut bool,
) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = S8;
        if button_sized(
            ui,
            &format!("{salt}_trash"),
            "Move to Trash",
            BtnKind::Destructive,
            trash_enabled,
            ACTION_COL_W,
            accent,
        )
        .clicked()
        {
            *on_trash = true;
        }
        if button_sized(ui, &format!("{salt}_show"), "Show", BtnKind::Secondary, show_enabled, ACTION_COL_W, accent).clicked() {
            *on_show = true;
        }
        ui.allocate_ui_with_layout(Vec2::new(SIZE_COL_W, BTN_MIN_H), Layout::left_to_right(Align::Center), |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label(RichText::new(size_label).strong().size(BODY).color(WHITE).monospace());
            });
        });
    });
}

pub fn text_field(ui: &mut Ui, text: &mut String, hint: &str, enabled: bool, monospace: bool) {
    let slot = ui.painter().add(Shape::Noop);
    let mut edit = egui::TextEdit::singleline(text)
        .desired_width(ui.available_width())
        .interactive(enabled)
        .margin(m_xy(S12, S8))
        .hint_text(RichText::new(hint).color(PLACEHOLDER));
    if monospace {
        edit = edit.font(egui::TextStyle::Monospace);
    }
    let response = ui.add(edit);
    let hot = response.hovered() || response.has_focus();
    let fill = if response.has_focus() {
        with_alpha(WHITE, 20)
    } else if hot {
        with_alpha(WHITE, 14)
    } else {
        INPUT_FILL
    };
    let border = if response.has_focus() { focus_stroke() } else { stroke(SURFACE_LINE) };
    ui.painter().set(
        slot,
        Shape::Rect(egui::epaint::RectShape::new(response.rect, r_sm(), fill, border)),
    );
}

pub fn scroll_list<F: FnOnce(&mut Ui)>(ui: &mut Ui, id: &str, min_height: f32, body: F) {
    surface_frame().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        egui::ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false; 2])
            .max_height(min_height.max(120.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width() - S8);
                body(ui);
            });
    });
}
