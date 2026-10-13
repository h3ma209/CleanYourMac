#![allow(dead_code)]
//! Opaque colors, the 7-size type ramp, the 4px spacing grid, and three radii.
//! `TEXT_3` is `#82879F`, lightened from the requested `#7E839B` (4.29:1 on
//! `SURFACE_RAISED`) so it clears 4.5:1. Measured 4.53:1.

use eframe::egui::{Color32, FontFamily, FontId, Margin, Rounding, Stroke, Vec2};

pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0C, 0x12);
pub const SURFACE: Color32 = Color32::from_rgb(0x15, 0x17, 0x26);
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x1D, 0x20, 0x33);
pub const BORDER: Color32 = Color32::from_rgb(0x2A, 0x2E, 0x45);
pub const TEXT: Color32 = Color32::from_rgb(0xF3, 0xF4, 0xF9);
/// ~70% white on `SURFACE_CARD`. About 8:1.
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xD8, 0xD8, 0xE8);
pub const PLACEHOLDER: Color32 = Color32::from_rgb(0xA8, 0xA8, 0xBC);
pub const TEXT_2: Color32 = TEXT_SECONDARY;
/// Decorative or ≥20px only if left at the requested value. Lightened so small text also clears 4.5:1.
pub const TEXT_3: Color32 = Color32::from_rgb(0x82, 0x87, 0x9F);
pub const WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);

pub const ACCENT: Color32 = Color32::from_rgb(0x65, 0x56, 0xF0);
/// 8 steps darker than `ACCENT` so hover does not lighten the fill. White contrast 5.64:1.
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x5D, 0x4E, 0xE8);
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(0x8B, 0x7F, 0xFF);

pub const SUCCESS: Color32 = Color32::from_rgb(0x3D, 0xDC, 0x97);
pub const WARN: Color32 = Color32::from_rgb(0xF5, 0xB9, 0x42);
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x6B, 0x6B);
pub const INFO: Color32 = Color32::from_rgb(0x5B, 0xB8, 0xFF);

/// Accent at 14% over `SURFACE`. Opaque.
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x20, 0x20, 0x42);
/// Accent at 40% over `SURFACE`. Opaque hero/selection edge.
pub const ACCENT_LINE: Color32 = Color32::from_rgb(0x35, 0x30, 0x77);
/// Danger at 14% over `SURFACE`.
pub const DANGER_SOFT: Color32 = Color32::from_rgb(0x36, 0x23, 0x30);
/// Danger at 40% over `SURFACE`.
pub const DANGER_LINE: Color32 = Color32::from_rgb(0x73, 0x39, 0x42);

pub const SCRIM: Color32 = Color32::from_black_alpha(170);

pub const S4: f32 = 4.0;
pub const S8: f32 = 8.0;
pub const S12: f32 = 12.0;
pub const S16: f32 = 16.0;
pub const S24: f32 = 24.0;
pub const S32: f32 = 32.0;

pub const CAPTION: f32 = 11.0;
pub const SMALL: f32 = 13.0;
pub const BODY: f32 = 14.0;
pub const TITLE: f32 = 16.0;
pub const H2: f32 = 20.0;
pub const STAT: f32 = 28.0;
pub const HERO: f32 = 40.0;
pub const LINE_HEIGHT: f32 = 1.45;

/// Third radius (with `RADIUS_SM` and `RADIUS_MD`). Meters and the nav mark.
pub const RADIUS_XS: f32 = 4.0;
pub const RADIUS_SM: f32 = 8.0;
pub const RADIUS_MD: f32 = 14.0;
pub const RADIUS_PILL: f32 = 999.0;

/// Icon-only corners. Not part of the three UI radii.
/// Lucide paths carry their own radii, so the painter no longer reads these.
#[allow(dead_code)]
pub const ICON_CORNER: f32 = 2.0;
#[allow(dead_code)]
pub const ICON_CORNER_SM: f32 = 1.5;

pub const STROKE_W: f32 = 1.0;
pub const FOCUS_W: f32 = 2.0;
pub const ICON_STROKE: f32 = 1.6;
pub const ICON_STROKE_SCREEN: f32 = 1.75;

pub const BTN_PAD_X: f32 = 16.0;
pub const BTN_MIN_W: f32 = 120.0;
pub const BTN_MIN_H: f32 = 40.0;
pub const GHOST_MIN_W: f32 = 88.0;
pub const GHOST_MIN_H: f32 = 32.0;
pub const CHIP_H: f32 = 32.0;

pub const WINDOW_W: f32 = 1180.0;
pub const WINDOW_H: f32 = 800.0;
pub const MIN_W: f32 = 720.0;
pub const MIN_H: f32 = 520.0;
pub const BREAK_COMPACT: f32 = 880.0;
pub const BREAK_NARROW: f32 = 1100.0;

pub const SHADOW_BLUR: f32 = 16.0;
pub const SHADOW_ALPHA: u8 = 90;

/// Unified content surfaces (cards, headers). White primary text ~16:1.
pub const SURFACE_CARD: Color32 = Color32::from_rgb(0x2A, 0x10, 0x58);
pub const SURFACE_LINE: Color32 = Color32::from_rgb(0xC4, 0xB4, 0xE8);
pub const INPUT_FILL: Color32 = Color32::from_rgb(0x1E, 0x0C, 0x42);

/// Home cards (alias of `SURFACE_CARD`).
pub const CARD_HOME: Color32 = SURFACE_CARD;
pub const CARD_MUTED: Color32 = Color32::from_rgb(0xE4, 0xD8, 0xFA);
pub const CARD_LINE: Color32 = SURFACE_LINE;
pub const BTN_HOVER: Color32 = Color32::from_rgb(0xE8, 0xE0, 0xF8);
pub const BTN_PRESSED: Color32 = Color32::from_rgb(0xC8, 0xB8, 0xE4);
pub const BTN_DISABLED: Color32 = Color32::from_rgb(0x4E, 0x3E, 0x78);

pub const CTA_H: f32 = 48.0;

/// Labeled rail. Wide enough for “System Monitor” at `SMALL`.
pub const RAIL_W: f32 = 176.0;

pub fn glass_fill() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 38)
}
pub fn glass_line() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 78)
}

/// Full-bleed module gradients. Stops are original Sable hues.
pub const GRAD_TEAL_TOP: Color32 = Color32::from_rgb(0x12, 0x86, 0x80);
pub const GRAD_TEAL_BOTTOM: Color32 = Color32::from_rgb(0x04, 0x3A, 0x44);
pub const GRAD_TEAL_ACCENT: Color32 = Color32::from_rgb(0x3E, 0xF0, 0xD0);

pub const GRAD_BLUE_TOP: Color32 = Color32::from_rgb(0x2A, 0x3E, 0xD6);
pub const GRAD_BLUE_BOTTOM: Color32 = Color32::from_rgb(0x0A, 0x18, 0x62);
pub const GRAD_BLUE_ACCENT: Color32 = Color32::from_rgb(0x7A, 0xB6, 0xFF);

pub const GRAD_ORANGE_TOP: Color32 = Color32::from_rgb(0xE0, 0x72, 0x18);
pub const GRAD_ORANGE_BOTTOM: Color32 = Color32::from_rgb(0x7A, 0x28, 0x06);
pub const GRAD_ORANGE_ACCENT: Color32 = Color32::from_rgb(0xFF, 0xC2, 0x70);

pub const GRAD_PURPLE_TOP: Color32 = Color32::from_rgb(0x6E, 0x32, 0xD4);
pub const GRAD_PURPLE_BOTTOM: Color32 = Color32::from_rgb(0x28, 0x0C, 0x58);
pub const GRAD_PURPLE_ACCENT: Color32 = Color32::from_rgb(0xD0, 0xA0, 0xFF);

pub const GRAD_NEUTRAL_TOP: Color32 = Color32::from_rgb(0x22, 0x20, 0x38);
pub const GRAD_NEUTRAL_BOTTOM: Color32 = Color32::from_rgb(0x0C, 0x0B, 0x16);
pub const GRAD_NEUTRAL_ACCENT: Color32 = ACCENT;

pub const GRAD_GREEN_TOP: Color32 = Color32::from_rgb(0x1C, 0x86, 0x4A);
pub const GRAD_GREEN_BOTTOM: Color32 = Color32::from_rgb(0x0A, 0x36, 0x24);
pub const GRAD_GREEN_ACCENT: Color32 = Color32::from_rgb(0x86, 0xF0, 0xB0);

pub const GRAD_SLATE_TOP: Color32 = Color32::from_rgb(0x2A, 0x48, 0x86);
pub const GRAD_SLATE_BOTTOM: Color32 = Color32::from_rgb(0x10, 0x1A, 0x36);
pub const GRAD_SLATE_ACCENT: Color32 = Color32::from_rgb(0xA8, 0xC0, 0xFF);

pub const GRAD_WINE_TOP: Color32 = Color32::from_rgb(0x86, 0x2C, 0x48);
pub const GRAD_WINE_BOTTOM: Color32 = Color32::from_rgb(0x36, 0x0E, 0x20);
pub const GRAD_WINE_ACCENT: Color32 = Color32::from_rgb(0xFF, 0xA0, 0xB4);

pub const GRAD_SKY_TOP: Color32 = Color32::from_rgb(0x18, 0x6A, 0xBE);
pub const GRAD_SKY_BOTTOM: Color32 = Color32::from_rgb(0x08, 0x28, 0x52);
pub const GRAD_SKY_ACCENT: Color32 = Color32::from_rgb(0x96, 0xD8, 0xFF);

pub const GRAD_INDIGO_TOP: Color32 = Color32::from_rgb(0x36, 0x2C, 0x84);
pub const GRAD_INDIGO_BOTTOM: Color32 = Color32::from_rgb(0x12, 0x0E, 0x34);
pub const GRAD_INDIGO_ACCENT: Color32 = Color32::from_rgb(0xC8, 0xBC, 0xFF);

/// Ink for labels sitting on a white pill.
pub const INK: Color32 = Color32::from_rgb(0x1A, 0x12, 0x32);

pub fn lh(size: f32) -> f32 {
    size * LINE_HEIGHT
}

pub fn font_regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn font_semi(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn r_xs() -> Rounding {
    Rounding::same(RADIUS_XS)
}
pub fn r_sm() -> Rounding {
    Rounding::same(RADIUS_SM)
}
pub fn r_md() -> Rounding {
    Rounding::same(RADIUS_MD)
}
pub fn r_pill() -> Rounding {
    Rounding::same(RADIUS_PILL)
}
#[allow(dead_code)]
pub fn r_icon() -> Rounding {
    Rounding::same(ICON_CORNER)
}
#[allow(dead_code)]
pub fn r_icon_sm() -> Rounding {
    Rounding::same(ICON_CORNER_SM)
}

pub fn stroke(color: Color32) -> Stroke {
    Stroke::new(STROKE_W, color)
}
pub fn no_stroke() -> Stroke {
    Stroke::NONE
}
pub fn focus_stroke() -> Stroke {
    Stroke::new(FOCUS_W, ACCENT_TEXT)
}

pub fn m_all(v: f32) -> Margin {
    Margin::same(v)
}
pub fn m_xy(x: f32, y: f32) -> Margin {
    Margin::symmetric(x, y)
}

pub fn v2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

pub fn shadow() -> eframe::egui::epaint::Shadow {
    eframe::egui::epaint::Shadow {
        offset: Vec2::new(0.0, S8),
        blur: SHADOW_BLUR,
        spread: 0.0,
        color: Color32::from_black_alpha(SHADOW_ALPHA),
    }
}

pub fn mix(fg: Color32, bg: Color32, percent: u8) -> Color32 {
    let t = percent as f32 / 100.0;
    let ch = |a: u8, b: u8| (a as f32 * t + b as f32 * (1.0 - t)).round() as u8;
    Color32::from_rgb(ch(fg.r(), bg.r()), ch(fg.g(), bg.g()), ch(fg.b(), bg.b()))
}

pub fn badge_fill(color: Color32) -> Color32 {
    mix(color, SURFACE, 14)
}
pub fn badge_line(color: Color32) -> Color32 {
    mix(color, SURFACE, 40)
}
pub fn avatar_fill(color: Color32) -> Color32 {
    mix(color, SURFACE, 14)
}
pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
