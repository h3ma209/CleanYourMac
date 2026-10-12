# 01 — Token sources (full text)

Searched under `RustUtilityApp/crates/ui` and `CustomUtilityApp/frontend` for `tailwind.config.*`, `variables.css`, `globals.css`, `tokens.json`: NOT FOUND.

Sources that actually hold colors, type, spacing, radius, and shadow:

1. `RustUtilityApp/crates/ui/src/theme.rs` (entire file, 126 lines)
2. Color constants in `RustUtilityApp/crates/ui/src/main.rs` lines 25–43
3. `CustomUtilityApp/frontend/src/renderer/styles.css` (entire file, 107 lines)

---

## `RustUtilityApp/crates/ui/src/main.rs` lines 25–43

```rust
// Spotify-inspired dark UI (#121212 base, #1DB954 accent).
const BG: Color32 = Color32::from_rgb(18, 18, 18);
const SIDEBAR: Color32 = Color32::from_rgb(0, 0, 0);
const ELEVATED: Color32 = Color32::from_rgb(24, 24, 24);
const CARD: Color32 = Color32::from_rgb(40, 40, 40);
const CARD_HOVER: Color32 = Color32::from_rgb(51, 51, 51);
const LINE: Color32 = Color32::from_rgb(42, 42, 42);
const TEXT: Color32 = Color32::from_rgb(255, 255, 255);
const SUBTEXT: Color32 = Color32::from_rgb(179, 179, 179);
const GREEN: Color32 = Color32::from_rgb(29, 185, 84);
const GREEN_ON: Color32 = Color32::from_rgb(255, 255, 255);
const AMBER: Color32 = Color32::from_rgb(240, 180, 41);
const ROSE: Color32 = Color32::from_rgb(232, 112, 106);
const SKY: Color32 = Color32::from_rgb(120, 190, 240);

const IVORY: Color32 = TEXT;
const MUTED: Color32 = SUBTEXT;
const LIME: Color32 = GREEN;
const LIME_INK: Color32 = GREEN_ON;
```

`cargo run -p ui` warned: `SIDEBAR` never used (`main.rs:27`), `LIME_INK` never used (`main.rs:43`).

---

## `RustUtilityApp/crates/ui/src/theme.rs` (full)

```rust
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
```

`apply_theme` passes `GREEN`, `TEXT`, `CARD` into `polish_style` (`main.rs:229-233`).

---

## `CustomUtilityApp/frontend/src/renderer/styles.css` (full)

```css
:root {
  color-scheme: light dark;
  font-family: "Segoe UI", sans-serif;
  background: #101214;
  color: #e8eaed;
}

body {
  margin: 0;
}

main {
  max-width: 720px;
  margin: 0 auto;
  padding: 32px 20px 48px;
}

h1 {
  margin: 0 0 8px;
  font-size: 1.6rem;
}

header p,
#status {
  color: #b0b6be;
}

label {
  display: block;
  margin: 24px 0 12px;
  font-size: 0.85rem;
}

input {
  display: block;
  width: 100%;
  margin-top: 6px;
  box-sizing: border-box;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid #3c424a;
  background: #1a1d21;
  color: inherit;
  font: inherit;
}

.controls {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
}

button {
  border: 0;
  border-radius: 8px;
  padding: 10px 14px;
  font: inherit;
  cursor: pointer;
  background: #2a6df4;
  color: white;
}

button#pause,
button#resume,
button#cancel {
  background: #2c3238;
}

.metrics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

article {
  background: #1a1d21;
  border-radius: 10px;
  padding: 14px;
}

article span {
  display: block;
  color: #b0b6be;
  font-size: 0.8rem;
}

article strong {
  display: block;
  margin-top: 6px;
  font-size: 1.4rem;
}

ul {
  list-style: none;
  padding: 0;
  margin: 16px 0 0;
  max-height: 220px;
  overflow: auto;
}

li {
  padding: 8px 0;
  border-bottom: 1px solid #2c3238;
  font-family: ui-monospace, monospace;
  font-size: 0.8rem;
  word-break: break-all;
}
```

`:root` sets `color-scheme: light dark` (`styles.css:2`) but every background and text color in the file is a fixed hex. No `@media (prefers-color-scheme)` rule: NOT FOUND in `styles.css`.
