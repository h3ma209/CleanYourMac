//! Lucide stroke icons (ISC), tinted at draw time.
//!
//! Files in `assets/icons/` are the upstream Lucide 0.544.0 SVGs
//! (`fill="none"`, `stroke="currentColor"`).
//! Source: <https://github.com/lucide-icons/lucide/tree/0.544.0/icons>
//! License: <https://github.com/lucide-icons/lucide/blob/0.544.0/LICENSE>
//!
//! egui 0.29 cannot draw SVG. `resvg` would pull a font stack this shell does
//! not need, so the Lucide subset used here (`path`, `circle`, `rect`, `line`)
//! is stroked directly. The caller color replaces `currentColor`. Stroke width
//! stays on `ICON_STROKE` / `ICON_STROKE_SCREEN`.
//!
//! | UI use | File | Lucide name |
//! | --- | --- | --- |
//! | Read-only badge | `eye.svg` | eye |
//! | Trash badge, Free Up Space | `trash-2.svg` | trash-2 |
//! | Admin badge | `lock.svg` | lock |
//! | Error banner, confirm | `triangle-alert.svg` | triangle-alert |
//! | Done banner | `check.svg` | check |
//! | Scan button | `scan.svg` | scan |
//! | `UiIcon::Folder` | `folder.svg` | folder |
//! | Empty state | `info.svg` | info |
//! | Overview | `house.svg` | house |
//! | Apps | `layout-grid.svg` | layout-grid |
//! | Tune-Up | `trending-up.svg` | trending-up |
//! | File Finder | `search.svg` | search |
//! | Threat Scan | `circle-plus.svg` | circle-plus |
//! | Big Downloads | `download.svg` | download |
//! | Space Lens | `chart-pie.svg` | chart-pie |
//! | Cloud Storage | `cloud.svg` | cloud |
//! | System Monitor | `chart-column.svg` | chart-column |
//! | Brand mark | `chart-no-axes-column.svg` | chart-no-axes-column |

use std::sync::OnceLock;

use eframe::egui::{self, pos2, Color32, FontId, Rect, RichText, Sense, Shape, Stroke, Vec2};

use crate::tokens::{self, font_semi};

use super::Screen;

#[derive(Clone, Copy, Debug)]
pub enum UiIcon {
    Eye,
    Trash,
    Lock,
    Alert,
    Check,
    Scan,
    #[allow(dead_code)]
    Folder,
    Info,
}

#[derive(Clone, Copy, Debug)]
enum Glyph {
    Eye,
    Trash,
    Lock,
    Alert,
    Check,
    Scan,
    Folder,
    Info,
    House,
    Grid,
    Trending,
    Search,
    PlusCircle,
    Download,
    Pie,
    Cloud,
    Columns,
    Bars,
}

pub fn icon_with_label(ui: &mut egui::Ui, icon: UiIcon, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.set_height(tokens::S16);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
        paint_ui_icon(ui.painter(), icon, rect, color);
        ui.label(
            RichText::new(label)
                .size(tokens::SMALL)
                .line_height(Some(tokens::lh(tokens::SMALL)))
                .color(color),
        );
    });
}

pub fn paint_ui_icon(painter: &egui::Painter, icon: UiIcon, rect: Rect, color: Color32) {
    let glyph = match icon {
        UiIcon::Eye => Glyph::Eye,
        UiIcon::Trash => Glyph::Trash,
        UiIcon::Lock => Glyph::Lock,
        UiIcon::Alert => Glyph::Alert,
        UiIcon::Check => Glyph::Check,
        UiIcon::Scan => Glyph::Scan,
        UiIcon::Folder => Glyph::Folder,
        UiIcon::Info => Glyph::Info,
    };
    paint_glyph(painter, glyph, rect, color, tokens::ICON_STROKE);
}

pub fn paint_screen_icon(painter: &egui::Painter, screen: Screen, rect: Rect, color: Color32) {
    let glyph = match screen {
        Screen::Care => Glyph::House,
        Screen::Cleanup => Glyph::Trash,
        Screen::Apps => Glyph::Grid,
        Screen::Performance => Glyph::Trending,
        Screen::Protection => Glyph::Search,
        Screen::Malware => Glyph::PlusCircle,
        Screen::Clutter => Glyph::Download,
        Screen::Space => Glyph::Pie,
        Screen::Cloud => Glyph::Cloud,
        Screen::Assistant => Glyph::Columns,
    };
    paint_glyph(painter, glyph, rect, color, tokens::ICON_STROKE_SCREEN);
}

pub fn paint_brand_mark(painter: &egui::Painter, rect: Rect, _color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
    painter.circle_filled(center, radius, tokens::ACCENT);
    let bar_w = (radius * 0.28).max(2.0);
    let gap = radius * 0.22;
    let heights = [0.42, 0.72, 0.54];
    let total = bar_w * 3.0 + gap * 2.0;
    let left = center.x - total * 0.5;
    for (index, height) in heights.iter().enumerate() {
        let h = radius * 1.15 * height;
        let x = left + index as f32 * (bar_w + gap);
        let bar = Rect::from_min_size(
            pos2(x, center.y + radius * 0.42 - h),
            Vec2::new(bar_w, h),
        );
        painter.rect_filled(bar, tokens::r_pill(), tokens::INK);
    }
}

pub fn heading_font(size: f32) -> FontId {
    font_semi(size)
}

/// Original glass marks for module landings. Geometry is not a traced asset.
#[derive(Clone, Copy, Debug)]
pub enum HeroKind {
    Folder,
    Apps,
    Bolt,
}

pub fn paint_hero(painter: &egui::Painter, kind: HeroKind, rect: Rect, tint: Color32) {
    if rect.width() < 8.0 || rect.height() < 8.0 {
        return;
    }
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.42;
    for step in 0..4 {
        let grow = 1.05 + step as f32 * 0.14;
        let alpha = 28u8.saturating_sub(step * 6);
        painter.circle_filled(center, radius * grow, tokens::with_alpha(tint, alpha));
    }
    match kind {
        HeroKind::Folder => paint_folder_blob(painter, center, radius, tint),
        HeroKind::Apps => paint_apps_blob(painter, center, radius, tint),
        HeroKind::Bolt => paint_bolt_blob(painter, center, radius, tint),
    }
}

fn paint_folder_blob(painter: &egui::Painter, center: egui::Pos2, radius: f32, tint: Color32) {
    let body = Rect::from_center_size(center + Vec2::new(0.0, radius * 0.08), Vec2::splat(radius * 0.92));
    painter.rect_filled(body, radius * 0.18, tokens::with_alpha(Color32::WHITE, 230));
    let tab = Rect::from_min_size(
        body.left_top() + Vec2::new(radius * 0.08, -radius * 0.16),
        Vec2::new(radius * 0.38, radius * 0.22),
    );
    painter.rect_filled(tab, radius * 0.08, tokens::with_alpha(Color32::WHITE, 230));
    painter.rect_filled(
        Rect::from_center_size(body.center() + Vec2::new(0.0, radius * 0.06), Vec2::new(radius * 0.62, radius * 0.08)),
        radius * 0.04,
        tokens::with_alpha(tint, 200),
    );
}

fn paint_apps_blob(painter: &egui::Painter, center: egui::Pos2, radius: f32, tint: Color32) {
    let hex = polygon(center, radius * 0.96, 6, std::f32::consts::FRAC_PI_6);
    painter.add(Shape::convex_polygon(
        hex,
        tokens::with_alpha(tint, 210),
        Stroke::new(2.0, tokens::with_alpha(Color32::WHITE, 160)),
    ));
    let arm = radius * 0.28;
    let stroke = Stroke::new((radius * 0.1).clamp(6.0, 14.0), Color32::WHITE);
    painter.line_segment(
        [center + Vec2::new(-arm, -arm), center + Vec2::new(arm, arm)],
        stroke,
    );
    painter.line_segment(
        [center + Vec2::new(arm, -arm), center + Vec2::new(-arm, arm)],
        stroke,
    );
}

fn paint_bolt_blob(painter: &egui::Painter, center: egui::Pos2, radius: f32, tint: Color32) {
    let shield = vec![
        center + Vec2::new(0.0, -radius * 0.95),
        center + Vec2::new(radius * 0.78, -radius * 0.45),
        center + Vec2::new(radius * 0.62, radius * 0.35),
        center + Vec2::new(0.0, radius * 0.95),
        center + Vec2::new(-radius * 0.62, radius * 0.35),
        center + Vec2::new(-radius * 0.78, -radius * 0.45),
    ];
    painter.add(Shape::convex_polygon(
        shield,
        tokens::with_alpha(tint, 210),
        Stroke::new(2.0, tokens::with_alpha(Color32::WHITE, 160)),
    ));
    let bolt = vec![
        center + Vec2::new(-radius * 0.06, -radius * 0.42),
        center + Vec2::new(radius * 0.22, -radius * 0.42),
        center + Vec2::new(radius * 0.02, -radius * 0.04),
        center + Vec2::new(radius * 0.24, -radius * 0.04),
        center + Vec2::new(-radius * 0.18, radius * 0.46),
        center + Vec2::new(-radius * 0.02, radius * 0.06),
        center + Vec2::new(-radius * 0.26, radius * 0.06),
    ];
    painter.add(Shape::convex_polygon(bolt, Color32::WHITE, Stroke::NONE));
}

fn polygon(center: egui::Pos2, radius: f32, sides: usize, turn: f32) -> Vec<egui::Pos2> {
    (0..sides)
        .map(|index| {
            let angle = turn + index as f32 * std::f32::consts::TAU / sides as f32;
            center + Vec2::new(angle.cos(), angle.sin()) * radius
        })
        .collect()
}

fn paint_glyph(painter: &egui::Painter, glyph: Glyph, rect: Rect, color: Color32, stroke_px: f32) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    let icon = cached(glyph);
    if icon.vb_w <= 0.0 || icon.vb_h <= 0.0 {
        return;
    }
    let scale = (rect.width() / icon.vb_w).min(rect.height() / icon.vb_h);
    let ox = rect.min.x + (rect.width() - icon.vb_w * scale) * 0.5 - icon.vb_x * scale;
    let oy = rect.min.y + (rect.height() - icon.vb_h * scale) * 0.5 - icon.vb_y * scale;
    let stroke = Stroke::new(stroke_px, color);
    let cap = stroke_px * 0.5;

    for prim in &icon.prims {
        match prim {
            Prim::Circle { cx, cy, r } => {
                painter.circle_stroke(pos2(ox + cx * scale, oy + cy * scale), r * scale, stroke);
            }
            Prim::Dot { x, y } => {
                painter.circle_filled(pos2(ox + x * scale, oy + y * scale), cap, color);
            }
            Prim::Poly { pts, closed } => {
                let mut points: Vec<egui::Pos2> = Vec::with_capacity(pts.len());
                for p in pts {
                    let q = pos2(ox + p[0] * scale, oy + p[1] * scale);
                    if let Some(prev) = points.last() {
                        let dx = prev.x - q.x;
                        let dy = prev.y - q.y;
                        if dx * dx + dy * dy < 0.25 {
                            continue;
                        }
                    }
                    points.push(q);
                }
                if points.len() < 2 {
                    continue;
                }
                if *closed {
                    painter.add(Shape::closed_line(points, stroke));
                } else {
                    let start = points[0];
                    let end = *points.last().unwrap();
                    painter.add(Shape::line(points, stroke));
                    painter.circle_filled(start, cap, color);
                    painter.circle_filled(end, cap, color);
                }
            }
        }
    }
}

macro_rules! slot {
    ($svg:expr) => {{
        static SLOT: OnceLock<Icon> = OnceLock::new();
        SLOT.get_or_init(|| parse_svg($svg))
    }};
}

fn cached(glyph: Glyph) -> &'static Icon {
    match glyph {
        Glyph::Eye => slot!(include_str!("../assets/icons/eye.svg")),
        Glyph::Trash => slot!(include_str!("../assets/icons/trash-2.svg")),
        Glyph::Lock => slot!(include_str!("../assets/icons/lock.svg")),
        Glyph::Alert => slot!(include_str!("../assets/icons/triangle-alert.svg")),
        Glyph::Check => slot!(include_str!("../assets/icons/check.svg")),
        Glyph::Scan => slot!(include_str!("../assets/icons/scan.svg")),
        Glyph::Folder => slot!(include_str!("../assets/icons/folder.svg")),
        Glyph::Info => slot!(include_str!("../assets/icons/info.svg")),
        Glyph::House => slot!(include_str!("../assets/icons/house.svg")),
        Glyph::Grid => slot!(include_str!("../assets/icons/layout-grid.svg")),
        Glyph::Trending => slot!(include_str!("../assets/icons/trending-up.svg")),
        Glyph::Search => slot!(include_str!("../assets/icons/search.svg")),
        Glyph::PlusCircle => slot!(include_str!("../assets/icons/circle-plus.svg")),
        Glyph::Download => slot!(include_str!("../assets/icons/download.svg")),
        Glyph::Pie => slot!(include_str!("../assets/icons/chart-pie.svg")),
        Glyph::Cloud => slot!(include_str!("../assets/icons/cloud.svg")),
        Glyph::Columns => slot!(include_str!("../assets/icons/chart-column.svg")),
        Glyph::Bars => slot!(include_str!("../assets/icons/chart-no-axes-column.svg")),
    }
}

struct Icon {
    vb_x: f32,
    vb_y: f32,
    vb_w: f32,
    vb_h: f32,
    prims: Vec<Prim>,
}

enum Prim {
    Poly { pts: Vec<[f32; 2]>, closed: bool },
    Circle { cx: f32, cy: f32, r: f32 },
    Dot { x: f32, y: f32 },
}

struct Pen {
    prims: Vec<Prim>,
    pts: Vec<[f32; 2]>,
}

impl Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish(false);
        self.pts.push([x, y]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if self.pts.is_empty() {
            self.pts.push([x, y]);
            return;
        }
        let last = *self.pts.last().unwrap();
        if (last[0] - x).abs() < 1e-4 && (last[1] - y).abs() < 1e-4 {
            return;
        }
        self.pts.push([x, y]);
    }

    fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        let (x0, y0) = if let Some(p) = self.pts.last() {
            (p[0], p[1])
        } else {
            self.pts.push([x, y]);
            return;
        };
        const STEPS: usize = 12;
        for step in 1..=STEPS {
            let t = step as f32 / STEPS as f32;
            let u = 1.0 - t;
            let uu = u * u;
            let tt = t * t;
            let px = uu * u * x0 + 3.0 * uu * t * c1x + 3.0 * u * tt * c2x + tt * t * x;
            let py = uu * u * y0 + 3.0 * uu * t * c1y + 3.0 * u * tt * c2y + tt * t * y;
            self.line_to(px, py);
        }
    }

    fn close(&mut self) {
        self.finish(true);
    }

    fn finish(&mut self, closed: bool) {
        if self.pts.len() >= 2 && closed {
            let a = self.pts[0];
            let b = *self.pts.last().unwrap();
            if (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) < 0.04 {
                self.pts.pop();
            }
        }
        if self.pts.len() == 2 && !closed {
            let a = self.pts[0];
            let b = self.pts[1];
            let d2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
            if d2 < 0.09 {
                self.prims.push(Prim::Dot { x: a[0], y: a[1] });
                self.pts.clear();
                return;
            }
        }
        if self.pts.len() >= 2 {
            self.prims.push(Prim::Poly {
                pts: std::mem::take(&mut self.pts),
                closed,
            });
        }
        self.pts.clear();
    }
}

fn parse_svg(svg: &str) -> Icon {
    let mut vb_x = 0.0;
    let mut vb_y = 0.0;
    let mut vb_w = 24.0;
    let mut vb_h = 24.0;
    let mut pen = Pen {
        prims: Vec::new(),
        pts: Vec::new(),
    };

    for tag in xml_tags(svg) {
        let name = tag.split_whitespace().next().unwrap_or("");
        match name {
            "svg" => {
                if let Some(view) = attr(tag, "viewBox") {
                    if let Some((x, y, w, h)) = four_floats(view) {
                        vb_x = x;
                        vb_y = y;
                        vb_w = w;
                        vb_h = h;
                    }
                }
            }
            "path" => {
                if let Some(d) = attr(tag, "d") {
                    parse_path(&mut pen, d);
                }
            }
            "circle" => {
                pen.finish(false);
                if let (Some(cx), Some(cy), Some(r)) =
                    (attr_f(tag, "cx"), attr_f(tag, "cy"), attr_f(tag, "r"))
                {
                    pen.prims.push(Prim::Circle { cx, cy, r });
                }
            }
            "rect" => {
                if let (Some(w), Some(h)) = (attr_f(tag, "width"), attr_f(tag, "height")) {
                    let x = attr_f(tag, "x").unwrap_or(0.0);
                    let y = attr_f(tag, "y").unwrap_or(0.0);
                    let rx = attr_f(tag, "rx").unwrap_or(0.0);
                    let ry = attr_f(tag, "ry").unwrap_or(rx);
                    rounded_rect(&mut pen, x, y, w, h, rx, ry);
                }
            }
            "line" => {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                    attr_f(tag, "x1"),
                    attr_f(tag, "y1"),
                    attr_f(tag, "x2"),
                    attr_f(tag, "y2"),
                ) {
                    pen.move_to(x1, y1);
                    pen.line_to(x2, y2);
                    pen.finish(false);
                }
            }
            _ => {}
        }
    }
    pen.finish(false);

    Icon {
        vb_x,
        vb_y,
        vb_w,
        vb_h,
        prims: pen.prims,
    }
}

fn xml_tags(svg: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if rest.starts_with('!') || rest.starts_with('?') || rest.starts_with('/') {
            continue;
        }
        let Some(end) = rest.find('>') else {
            break;
        };
        let mut tag = &rest[..end];
        if let Some(stripped) = tag.strip_suffix('/') {
            tag = stripped.trim_end();
        }
        tags.push(tag);
        rest = &rest[end + 1..];
    }
    tags
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let needle = name.as_bytes();
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        let boundary = i == 0 || bytes[i - 1].is_ascii_whitespace();
        if boundary && tag[i..].starts_with(name) && bytes.get(i + needle.len()) == Some(&b'=') {
            let mut j = i + needle.len() + 1;
            let quote = *bytes.get(j)?;
            if quote != b'"' && quote != b'\'' {
                return None;
            }
            j += 1;
            let start = j;
            while j < bytes.len() && bytes[j] != quote {
                j += 1;
            }
            return Some(&tag[start..j]);
        }
        i += 1;
    }
    None
}

fn attr_f(tag: &str, name: &str) -> Option<f32> {
    attr(tag, name)?.parse().ok()
}

fn four_floats(text: &str) -> Option<(f32, f32, f32, f32)> {
    let mut nums = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<f32>().ok());
    Some((nums.next()??, nums.next()??, nums.next()??, nums.next()??))
}

fn rounded_rect(pen: &mut Pen, x: f32, y: f32, w: f32, h: f32, rx: f32, ry: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let rx = rx.max(0.0).min(w * 0.5);
    let ry = ry.max(0.0).min(h * 0.5);
    if rx < 0.01 && ry < 0.01 {
        pen.move_to(x, y);
        pen.line_to(x + w, y);
        pen.line_to(x + w, y + h);
        pen.line_to(x, y + h);
        pen.close();
        return;
    }
    pen.move_to(x + rx, y);
    pen.line_to(x + w - rx, y);
    arc_to(pen, rx, ry, 0.0, false, true, x + w, y + ry);
    pen.line_to(x + w, y + h - ry);
    arc_to(pen, rx, ry, 0.0, false, true, x + w - rx, y + h);
    pen.line_to(x + rx, y + h);
    arc_to(pen, rx, ry, 0.0, false, true, x, y + h - ry);
    pen.line_to(x, y + ry);
    arc_to(pen, rx, ry, 0.0, false, true, x + rx, y);
    pen.close();
}

fn parse_path(pen: &mut Pen, d: &str) {
    let bytes = d.as_bytes();
    let mut i = 0;
    let mut cmd = 0u8;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;

    loop {
        skip_seps(bytes, &mut i);
        if i >= bytes.len() {
            break;
        }
        if bytes[i].is_ascii_alphabetic() {
            cmd = bytes[i];
            i += 1;
        } else if cmd == 0 {
            break;
        }

        match cmd {
            b'Z' | b'z' => {
                pen.close();
                cx = sx;
                cy = sy;
            }
            b'M' | b'm' => {
                let Some(x) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y) = read_number(bytes, &mut i) else {
                    break;
                };
                let (x, y) = if cmd == b'm' {
                    (cx + x, cy + y)
                } else {
                    (x, y)
                };
                pen.move_to(x, y);
                cx = x;
                cy = y;
                sx = x;
                sy = y;
                cmd = if cmd == b'm' { b'l' } else { b'L' };
            }
            b'L' | b'l' => {
                let Some(x) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y) = read_number(bytes, &mut i) else {
                    break;
                };
                let (x, y) = if cmd == b'l' {
                    (cx + x, cy + y)
                } else {
                    (x, y)
                };
                draw_line(pen, &mut cx, &mut cy, x, y);
            }
            b'H' | b'h' => {
                let Some(x) = read_number(bytes, &mut i) else {
                    break;
                };
                let x = if cmd == b'h' { cx + x } else { x };
                let y = cy;
                draw_line(pen, &mut cx, &mut cy, x, y);
            }
            b'V' | b'v' => {
                let Some(y) = read_number(bytes, &mut i) else {
                    break;
                };
                let y = if cmd == b'v' { cy + y } else { y };
                let x = cx;
                draw_line(pen, &mut cx, &mut cy, x, y);
            }
            b'C' | b'c' => {
                let Some(x1) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y1) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(x2) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y2) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(x) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y) = read_number(bytes, &mut i) else {
                    break;
                };
                let (x1, y1, x2, y2, x, y) = if cmd == b'c' {
                    (cx + x1, cy + y1, cx + x2, cy + y2, cx + x, cy + y)
                } else {
                    (x1, y1, x2, y2, x, y)
                };
                if pen.pts.is_empty() {
                    pen.pts.push([cx, cy]);
                }
                pen.cubic_to(x1, y1, x2, y2, x, y);
                cx = x;
                cy = y;
            }
            b'A' | b'a' => {
                let Some(rx) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(ry) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(rot) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(large) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(sweep) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(x) = read_number(bytes, &mut i) else {
                    break;
                };
                let Some(y) = read_number(bytes, &mut i) else {
                    break;
                };
                let (x, y) = if cmd == b'a' {
                    (cx + x, cy + y)
                } else {
                    (x, y)
                };
                if pen.pts.is_empty() {
                    pen.pts.push([cx, cy]);
                }
                arc_to(pen, rx, ry, rot, large >= 0.5, sweep >= 0.5, x, y);
                cx = x;
                cy = y;
            }
            _ => break,
        }
    }
    pen.finish(false);
}

fn draw_line(pen: &mut Pen, cx: &mut f32, cy: &mut f32, x: f32, y: f32) {
    if pen.pts.is_empty() {
        pen.pts.push([*cx, *cy]);
    }
    pen.line_to(x, y);
    *cx = x;
    *cy = y;
}

fn skip_seps(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() {
        match bytes[*i] {
            b' ' | b',' | b'\n' | b'\r' | b'\t' => *i += 1,
            _ => break,
        }
    }
}

fn read_number(bytes: &[u8], i: &mut usize) -> Option<f32> {
    skip_seps(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    let start = *i;
    if bytes[*i] == b'+' || bytes[*i] == b'-' {
        *i += 1;
    }
    let mut saw_digit = false;
    let mut saw_dot = false;
    while *i < bytes.len() {
        let c = bytes[*i];
        if c.is_ascii_digit() {
            saw_digit = true;
            *i += 1;
        } else if c == b'.' && !saw_dot {
            saw_dot = true;
            *i += 1;
        } else {
            break;
        }
    }
    if !saw_digit {
        *i = start;
        return None;
    }
    if *i < bytes.len() && (bytes[*i] == b'e' || bytes[*i] == b'E') {
        let mark = *i;
        *i += 1;
        if *i < bytes.len() && (bytes[*i] == b'+' || bytes[*i] == b'-') {
            *i += 1;
        }
        let exp_at = *i;
        while *i < bytes.len() && bytes[*i].is_ascii_digit() {
            *i += 1;
        }
        if *i == exp_at {
            *i = mark;
        }
    }
    std::str::from_utf8(&bytes[start..*i]).ok()?.parse().ok()
}

/// SVG endpoint-parameterized arc, sampled as cubics. `rx`/`ry` are radii.
fn arc_to(
    pen: &mut Pen,
    mut rx: f32,
    mut ry: f32,
    phi_deg: f32,
    large: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
) {
    let (x1, y1) = match pen.pts.last() {
        Some(p) => (p[0], p[1]),
        None => {
            pen.line_to(x2, y2);
            return;
        }
    };
    if (x1 - x2).abs() < 1e-5 && (y1 - y2).abs() < 1e-5 {
        return;
    }
    rx = rx.abs();
    ry = ry.abs();
    if rx < 1e-5 || ry < 1e-5 {
        pen.line_to(x2, y2);
        return;
    }

    let phi = phi_deg.to_radians();
    let (sin_phi, cos_phi) = (phi.sin(), phi.cos());
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    let mut rxsq = rx * rx;
    let mut rysq = ry * ry;
    let x1psq = x1p * x1p;
    let y1psq = y1p * y1p;
    let lambda = x1psq / rxsq + y1psq / rysq;
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
        rxsq = rx * rx;
        rysq = ry * ry;
    }

    let den = rxsq * y1psq + rysq * x1psq;
    if den.abs() < 1e-12 {
        pen.line_to(x2, y2);
        return;
    }
    let mut rad = ((rxsq * rysq - rxsq * y1psq - rysq * x1psq) / den)
        .max(0.0)
        .sqrt();
    if large == sweep {
        rad = -rad;
    }
    let cxp = rad * rx * y1p / ry;
    let cyp = rad * -ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

    let theta1 = vector_angle(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dtheta = vector_angle(
        (x1p - cxp) / rx,
        (y1p - cyp) / ry,
        (-x1p - cxp) / rx,
        (-y1p - cyp) / ry,
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= std::f32::consts::TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += std::f32::consts::TAU;
    }

    let segments = ((dtheta.abs() / std::f32::consts::FRAC_PI_2).ceil() as usize).max(1);
    let delta = dtheta / segments as f32;
    let k = (4.0 / 3.0) * (delta / 4.0).tan();
    let mut t1 = theta1;
    for _ in 0..segments {
        let t2 = t1 + delta;
        let (s1, c1) = (t1.sin(), t1.cos());
        let (s2, c2) = (t2.sin(), t2.cos());
        let e2 = map_ellipse(cx, cy, rx, ry, cos_phi, sin_phi, c1 - k * s1, s1 + k * c1);
        let e3 = map_ellipse(cx, cy, rx, ry, cos_phi, sin_phi, c2 + k * s2, s2 - k * c2);
        let e4 = map_ellipse(cx, cy, rx, ry, cos_phi, sin_phi, c2, s2);
        pen.cubic_to(e2.0, e2.1, e3.0, e3.1, e4.0, e4.1);
        t1 = t2;
    }
}

fn map_ellipse(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    cos_phi: f32,
    sin_phi: f32,
    x: f32,
    y: f32,
) -> (f32, f32) {
    (
        cx + rx * cos_phi * x - ry * sin_phi * y,
        cy + rx * sin_phi * x + ry * cos_phi * y,
    )
}

fn vector_angle(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
    if len < 1e-6 {
        return 0.0;
    }
    let mut ang = ((ux * vx + uy * vy) / len).clamp(-1.0, 1.0).acos();
    if ux * vy - uy * vx < 0.0 {
        ang = -ang;
    }
    ang
}

#[cfg(test)]
mod tests {
    use super::*;

    const GLYPHS: [Glyph; 18] = [
        Glyph::Eye,
        Glyph::Trash,
        Glyph::Lock,
        Glyph::Alert,
        Glyph::Check,
        Glyph::Scan,
        Glyph::Folder,
        Glyph::Info,
        Glyph::House,
        Glyph::Grid,
        Glyph::Trending,
        Glyph::Search,
        Glyph::PlusCircle,
        Glyph::Download,
        Glyph::Pie,
        Glyph::Cloud,
        Glyph::Columns,
        Glyph::Bars,
    ];

    #[test]
    fn quarter_arc_stays_on_unit_circle() {
        let mut pen = Pen {
            prims: Vec::new(),
            pts: vec![[1.0, 0.0]],
        };
        arc_to(&mut pen, 1.0, 1.0, 0.0, false, true, 0.0, 1.0);
        pen.finish(false);
        let Prim::Poly { pts, .. } = &pen.prims[0] else {
            panic!("arc did not emit a polyline");
        };
        let end = pts.last().unwrap();
        assert!((end[0] - 0.0).abs() < 0.02 && (end[1] - 1.0).abs() < 0.02);
        for p in pts {
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!((r - 1.0).abs() < 0.03, "point {p:?} radius {r}");
        }
    }

    #[test]
    fn check_keeps_its_corners() {
        let icon = cached(Glyph::Check);
        let Prim::Poly { pts, closed } = &icon.prims[0] else {
            panic!("check is not a stroke");
        };
        assert!(!closed);
        assert!(pts
            .iter()
            .any(|p| (p[0] - 20.0).abs() < 0.01 && (p[1] - 6.0).abs() < 0.01));
        assert!(pts
            .iter()
            .any(|p| (p[0] - 9.0).abs() < 0.01 && (p[1] - 17.0).abs() < 0.01));
        assert!(pts
            .iter()
            .any(|p| (p[0] - 4.0).abs() < 0.01 && (p[1] - 12.0).abs() < 0.01));
    }

    #[test]
    fn info_dot_is_a_dot() {
        let icon = cached(Glyph::Info);
        assert!(icon
            .prims
            .iter()
            .any(|prim| matches!(prim, Prim::Dot { .. })));
        assert!(icon
            .prims
            .iter()
            .any(|prim| matches!(prim, Prim::Circle { .. })));
    }

    #[test]
    fn every_glyph_stays_in_viewbox() {
        for glyph in GLYPHS {
            let icon = cached(glyph);
            assert!(!icon.prims.is_empty(), "glyph {glyph:?} parsed empty");
            for prim in &icon.prims {
                match prim {
                    Prim::Poly { pts, .. } => {
                        assert!(pts.len() >= 2);
                        for p in pts {
                            assert!(p[0].is_finite() && p[1].is_finite(), "{p:?}");
                            assert!((-1.0..26.0).contains(&p[0]), "{glyph:?} x {}", p[0]);
                            assert!((-1.0..26.0).contains(&p[1]), "{glyph:?} y {}", p[1]);
                        }
                    }
                    Prim::Circle { cx, cy, r } => {
                        assert!(cx.is_finite() && cy.is_finite() && *r > 0.0);
                    }
                    Prim::Dot { x, y } => {
                        assert!(x.is_finite() && y.is_finite());
                    }
                }
            }
        }
    }
}
