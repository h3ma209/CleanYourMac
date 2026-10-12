use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Margin, Rounding, Stroke, Style, TextStyle, Vec2,
};

const FONT_REGULAR: &[u8] = include_bytes!("../assets/SableUI-Regular.ttf");
const FONT_SEMIBOLD: &[u8] = include_bytes!("../assets/SableUI-SemiBold.ttf");

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("sable_regular".into(), FontData::from_static(FONT_REGULAR));
    fonts
        .font_data
        .insert("sable_semibold".into(), FontData::from_static(FONT_SEMIBOLD));

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "sable_regular".into());
    fonts
        .families
        .insert(FontFamily::Name("semibold".into()), vec!["sable_semibold".into()]);

    ctx.set_fonts(fonts);
}

pub fn polish_style(style: &mut Style, accent: Color32, text: Color32, card: Color32) {
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(16.0, 9.0);
    style.spacing.scroll.bar_width = 6.0;
    style.visuals.window_fill = Color32::from_rgb(14, 14, 16);
    style.visuals.panel_fill = Color32::from_rgb(14, 14, 16);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 10, 12);
    style.visuals.faint_bg_color = card;
    style.visuals.code_bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, 120);
    style.visuals.widgets.noninteractive.bg_fill = card;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);
    style.visuals.widgets.inactive.bg_fill = card;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, glass_stroke_color());
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 16);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent);
    style.visuals.widgets.active.bg_fill = accent;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 48);
    style.visuals.selection.stroke = Stroke::new(1.0, accent);
    style.visuals.override_text_color = Some(text);

    let semi = FontFamily::Name("semibold".into());
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(28.0, semi.clone()));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.5, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(14.0, semi));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(11.5, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace));
}

fn glass_stroke_color() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 22)
}

pub fn glass_card() -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 10))
        .stroke(Stroke::new(1.0, glass_stroke_color()))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::same(16.0))
        .shadow(egui::epaint::Shadow {
            offset: Vec2::new(0.0, 6.0),
            blur: 14.0,
            spread: 0.0,
            color: Color32::from_black_alpha(55),
        })
}

pub fn glass_hero(accent: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 16))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 55),
        ))
        .rounding(Rounding::same(18.0))
        .inner_margin(Margin::same(22.0))
        .shadow(egui::epaint::Shadow {
            offset: Vec2::new(0.0, 8.0),
            blur: 18.0,
            spread: 0.0,
            color: Color32::from_black_alpha(70),
        })
}

pub fn glass_sidebar() -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(8, 8, 10, 235))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14)))
        .inner_margin(Margin::same(0.0))
}

pub fn glass_pill() -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 8))
        .stroke(Stroke::new(1.0, glass_stroke_color()))
        .rounding(Rounding::same(20.0))
        .inner_margin(Margin::symmetric(12.0, 7.0))
}

pub fn glass_notice(accent: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 18))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 90)))
        .rounding(Rounding::same(14.0))
        .inner_margin(Margin::symmetric(16.0, 12.0))
}
