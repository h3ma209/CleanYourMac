use eframe::egui::{self, Color32, FontFamily, FontId, Pos2, Rect, Rounding, Stroke, TextStyle, Vec2};

pub const RADIUS_CARD: f32 = 22.0;
pub const RADIUS_PILL: f32 = 24.0;
pub const SIDEBAR_W: f32 = 228.0;

#[derive(Clone, Copy)]
pub struct Palette {
    pub light: bool,
    pub text: Color32,
    pub muted: Color32,
    pub card: Color32,
    pub stroke: Color32,
    pub accent: Color32,
    pub accent_ink: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub glass: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            light: false,
            text: Color32::from_rgb(247, 244, 255),
            muted: Color32::from_rgb(186, 176, 214),
            card: Color32::from_rgba_unmultiplied(255, 255, 255, 28),
            stroke: Color32::from_rgba_unmultiplied(255, 255, 255, 46),
            accent: Color32::from_rgb(62, 224, 197),
            accent_ink: Color32::from_rgb(8, 28, 32),
            good: Color32::from_rgb(88, 224, 154),
            warn: Color32::from_rgb(255, 154, 92),
            glass: Color32::from_rgba_unmultiplied(18, 12, 40, 150),
        }
    }

    pub fn light() -> Self {
        Self {
            light: true,
            text: Color32::from_rgb(32, 24, 58),
            muted: Color32::from_rgb(98, 84, 128),
            card: Color32::from_rgba_unmultiplied(255, 255, 255, 188),
            stroke: Color32::from_rgba_unmultiplied(255, 255, 255, 230),
            accent: Color32::from_rgb(14, 150, 142),
            accent_ink: Color32::from_rgb(255, 255, 255),
            good: Color32::from_rgb(18, 148, 96),
            warn: Color32::from_rgb(196, 92, 42),
            glass: Color32::from_rgba_unmultiplied(255, 255, 255, 120),
        }
    }
}

pub fn apply_type(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(RADIUS_PILL);
    style.visuals.widgets.hovered.rounding = Rounding::same(RADIUS_PILL);
    style.visuals.widgets.active.rounding = Rounding::same(RADIUS_PILL);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(28.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

pub fn paint_backdrop(painter: &egui::Painter, rect: Rect, light: bool, time: f64) {
    let (a, b, c) = if light {
        (
            Color32::from_rgb(236, 228, 255),
            Color32::from_rgb(255, 214, 236),
            Color32::from_rgb(198, 244, 236),
        )
    } else {
        (
            Color32::from_rgb(10, 16, 48),
            Color32::from_rgb(58, 18, 92),
            Color32::from_rgb(18, 72, 88),
        )
    };
    let steps = 28;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let color = if t < 0.55 {
            mix(a, b, t / 0.55)
        } else {
            mix(b, c, (t - 0.55) / 0.45)
        };
        let y0 = rect.top() + rect.height() * t;
        let y1 = y0 + rect.height() / steps as f32 + 1.2;
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(rect.left(), y0), Pos2::new(rect.right(), y1)),
            0.0,
            color,
        );
    }
    let drift = (time * 0.35).sin() as f32;
    glow(
        painter,
        Pos2::new(rect.center().x + 180.0, rect.top() + 120.0 + drift * 12.0),
        180.0,
        if light {
            Color32::from_rgba_unmultiplied(255, 180, 220, 70)
        } else {
            Color32::from_rgba_unmultiplied(180, 80, 255, 55)
        },
    );
    glow(
        painter,
        Pos2::new(rect.left() + 80.0, rect.bottom() - 80.0),
        140.0,
        if light {
            Color32::from_rgba_unmultiplied(80, 210, 200, 60)
        } else {
            Color32::from_rgba_unmultiplied(40, 200, 190, 40)
        },
    );
}

fn glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    for i in 0..6 {
        let t = i as f32 / 6.0;
        let r = radius * (1.0 - t * 0.45);
        let a = (color.a() as f32 * (1.0 - t) * 0.55) as u8;
        painter.circle_filled(
            center,
            r,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a),
        );
    }
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        lerp_u8(a.r(), b.r(), t),
        lerp_u8(a.g(), b.g(), t),
        lerp_u8(a.b(), b.b(), t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

pub fn glass<R>(
    ui: &mut egui::Ui,
    palette: Palette,
    radius: f32,
    margin: f32,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::none()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.stroke))
        .rounding(Rounding::same(radius))
        .inner_margin(egui::Margin::same(margin))
        .shadow(egui::epaint::Shadow {
            offset: Vec2::new(0.0, 10.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_rgba_unmultiplied(20, 8, 40, if palette.light { 28 } else { 70 }),
        })
        .show(ui, body)
}

pub fn pill(ui: &mut egui::Ui, palette: Palette, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(
            egui::RichText::new(label)
                .color(palette.accent_ink)
                .strong()
                .size(16.0),
        )
        .fill(palette.accent)
        .rounding(Rounding::same(RADIUS_PILL))
        .min_size(Vec2::new(148.0, 44.0)),
    )
}

pub fn quiet(ui: &mut egui::Ui, palette: Palette, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(palette.text).size(14.0))
            .fill(palette.glass)
            .stroke(Stroke::new(1.0, palette.stroke))
            .rounding(Rounding::same(RADIUS_PILL))
            .min_size(Vec2::new(96.0, 36.0)),
    )
}

pub fn format_bytes(n: u64) -> String {
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
