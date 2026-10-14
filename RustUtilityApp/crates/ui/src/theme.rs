use eframe::egui::{self, FontData, FontDefinitions, FontFamily, Stroke, Style, TextStyle};

use crate::tokens::{self, font_regular, font_semi, focus_stroke, m_all, m_xy, r_md, r_pill, r_sm, stroke};

const FONT_REGULAR: &[u8] = include_bytes!("../assets/CleanYourMac-Regular.ttf");
const FONT_SEMIBOLD: &[u8] = include_bytes!("../assets/CleanYourMac-SemiBold.ttf");

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("clean_your_mac_regular".into(), FontData::from_static(FONT_REGULAR));
    fonts
        .font_data
        .insert("clean_your_mac_semibold".into(), FontData::from_static(FONT_SEMIBOLD));

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "clean_your_mac_regular".into());
    fonts
        .families
        .insert(FontFamily::Name("semibold".into()), vec!["clean_your_mac_semibold".into()]);

    ctx.set_fonts(fonts);
}

pub fn polish_style(style: &mut Style) {
    style.spacing.item_spacing = tokens::v2(tokens::S8, tokens::S8);
    style.spacing.button_padding = tokens::v2(tokens::S16, tokens::S8);
    style.spacing.scroll.bar_width = tokens::S4;
    style.visuals.window_fill = tokens::BG;
    style.visuals.panel_fill = tokens::BG;
    style.visuals.extreme_bg_color = tokens::BG;
    style.visuals.faint_bg_color = tokens::SURFACE;
    style.visuals.code_bg_color = tokens::BG;
    style.visuals.override_text_color = Some(tokens::WHITE);

    let glass = tokens::glass_fill();
    let glass_hot = tokens::with_alpha(tokens::WHITE, 52);
    let line = stroke(tokens::glass_line());
    style.visuals.widgets.noninteractive.bg_fill = glass;
    style.visuals.widgets.noninteractive.weak_bg_fill = glass;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(tokens::STROKE_W, tokens::WHITE);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.noninteractive.rounding = r_sm();

    style.visuals.widgets.inactive.bg_fill = glass;
    style.visuals.widgets.inactive.weak_bg_fill = glass;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(tokens::STROKE_W, tokens::WHITE);
    style.visuals.widgets.inactive.bg_stroke = line;
    style.visuals.widgets.inactive.rounding = r_sm();

    style.visuals.widgets.hovered.bg_fill = glass_hot;
    style.visuals.widgets.hovered.weak_bg_fill = glass_hot;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(tokens::STROKE_W, tokens::WHITE);
    style.visuals.widgets.hovered.bg_stroke = line;
    style.visuals.widgets.hovered.rounding = r_sm();

    // egui maps keyboard focus onto the active visual (`Response::has_focus`).
    style.visuals.widgets.active.bg_fill = glass_hot;
    style.visuals.widgets.active.weak_bg_fill = glass;
    style.visuals.widgets.active.fg_stroke = Stroke::new(tokens::STROKE_W, tokens::WHITE);
    style.visuals.widgets.active.bg_stroke = focus_stroke();
    style.visuals.widgets.active.rounding = r_sm();

    style.visuals.widgets.open.bg_fill = glass_hot;
    style.visuals.widgets.open.weak_bg_fill = glass;
    style.visuals.widgets.open.fg_stroke = Stroke::new(tokens::STROKE_W, tokens::WHITE);
    style.visuals.widgets.open.bg_stroke = focus_stroke();
    style.visuals.widgets.open.rounding = r_sm();

    style.visuals.selection.bg_fill = tokens::ACCENT_SOFT;
    style.visuals.selection.stroke = focus_stroke();

    style.text_styles.insert(TextStyle::Heading, font_semi(tokens::H2));
    style.text_styles.insert(TextStyle::Body, font_regular(tokens::BODY));
    style.text_styles.insert(TextStyle::Button, font_semi(tokens::BODY));
    style.text_styles.insert(TextStyle::Small, font_regular(tokens::CAPTION));
    style.text_styles.insert(TextStyle::Monospace, font_regular(tokens::SMALL));
}

pub fn surface_card() -> egui::Frame {
    egui::Frame::none()
        .fill(tokens::SURFACE_CARD)
        .stroke(stroke(tokens::SURFACE_LINE))
        .rounding(r_md())
        .inner_margin(m_all(tokens::S16))
}

pub fn hero_card() -> egui::Frame {
    egui::Frame::none()
        .fill(tokens::glass_fill())
        .stroke(stroke(tokens::glass_line()))
        .rounding(r_md())
        .inner_margin(m_all(tokens::S24))
}

pub fn modal_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(tokens::SURFACE_RAISED)
        .stroke(stroke(tokens::BORDER))
        .rounding(r_md())
        .inner_margin(m_all(tokens::S24))
        .shadow(tokens::shadow())
}

pub fn pill_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(tokens::glass_fill())
        .stroke(stroke(tokens::glass_line()))
        .rounding(r_pill())
        .inner_margin(m_xy(tokens::S12, tokens::S8))
}

pub fn notice_frame(accent: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(tokens::with_alpha(accent, 36))
        .stroke(stroke(tokens::with_alpha(tokens::WHITE, 90)))
        .rounding(r_md())
        .inner_margin(m_xy(tokens::S16, tokens::S12))
}
