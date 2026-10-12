use eframe::egui::{self, pos2, Color32, FontFamily, FontId, Rect, RichText, Rounding, Sense, Shape, Stroke, Vec2};

use super::Screen;

#[derive(Clone, Copy, Debug)]
pub enum UiIcon {
    Eye,
    Trash,
    Lock,
    Alert,
    Check,
    Scan,
    Folder,
    Info,
}

pub fn icon_with_label(ui: &mut egui::Ui, icon: UiIcon, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.set_height(18.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
        paint_ui_icon(ui.painter(), icon, rect, color);
        ui.label(RichText::new(label).size(12.5).color(color));
    });
}

pub fn paint_ui_icon(painter: &egui::Painter, icon: UiIcon, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.42;
    let stroke = Stroke::new(1.6, color);
    match icon {
        UiIcon::Eye => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.circle_filled(c, r * 0.28, color);
        }
        UiIcon::Trash => {
            let bin = Rect::from_center_size(pos2(c.x, c.y + r * 0.25), Vec2::new(r * 1.1, r));
            painter.rect_stroke(bin, Rounding::same(1.5), stroke);
            painter.line_segment([pos2(c.x - r * 0.5, c.y - r * 0.45), pos2(c.x + r * 0.5, c.y - r * 0.45)], stroke);
        }
        UiIcon::Lock => {
            painter.rect_stroke(
                Rect::from_center_size(pos2(c.x, c.y + r * 0.2), Vec2::new(r * 0.9, r * 0.75)),
                Rounding::same(2.0),
                stroke,
            );
            painter.circle_stroke(pos2(c.x, c.y - r * 0.35), r * 0.35, stroke);
        }
        UiIcon::Alert => {
            painter.add(Shape::line(
                vec![
                    pos2(c.x, c.y - r),
                    pos2(c.x + r * 0.9, c.y + r * 0.75),
                    pos2(c.x - r * 0.9, c.y + r * 0.75),
                    pos2(c.x, c.y - r),
                ],
                stroke,
            ));
            painter.line_segment([pos2(c.x, c.y - r * 0.15), pos2(c.x, c.y + r * 0.25)], stroke);
            painter.circle_filled(pos2(c.x, c.y + r * 0.45), 1.6, color);
        }
        UiIcon::Check => {
            painter.add(Shape::line(
                vec![
                    pos2(c.x - r * 0.55, c.y),
                    pos2(c.x - r * 0.05, c.y + r * 0.45),
                    pos2(c.x + r * 0.65, c.y - r * 0.55),
                ],
                stroke,
            ));
        }
        UiIcon::Scan => {
            painter.circle_stroke(c, r * 0.75, stroke);
            painter.line_segment([pos2(c.x - r * 0.2, c.y + r * 0.2), pos2(c.x + r * 0.55, c.y - r * 0.55)], stroke);
        }
        UiIcon::Folder => {
            painter.rect_stroke(
                Rect::from_center_size(pos2(c.x, c.y + r * 0.15), Vec2::new(r * 1.1, r * 0.75)),
                Rounding::same(2.0),
                stroke,
            );
            painter.line_segment([pos2(c.x - r * 0.35, c.y - r * 0.15), pos2(c.x + r * 0.35, c.y - r * 0.15)], stroke);
        }
        UiIcon::Info => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.circle_filled(pos2(c.x, c.y - r * 0.35), 1.5, color);
            painter.line_segment([pos2(c.x, c.y - r * 0.05), pos2(c.x, c.y + r * 0.45)], stroke);
        }
    }
}

pub fn paint_screen_icon(painter: &egui::Painter, screen: Screen, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.36;
    let stroke = Stroke::new(1.75, color);
    match screen {
        Screen::Care => {
            painter.add(Shape::line(
                vec![
                    pos2(c.x - r, c.y + r * 0.15),
                    pos2(c.x, c.y - r),
                    pos2(c.x + r, c.y + r * 0.15),
                ],
                stroke,
            ));
            let body = Rect::from_center_size(pos2(c.x, c.y + r * 0.72), Vec2::new(r * 1.35, r * 0.95));
            painter.rect_stroke(body, Rounding::same(1.5), stroke);
        }
        Screen::Cleanup => {
            let bin = Rect::from_center_size(pos2(c.x, c.y + r * 0.35), Vec2::new(r * 1.2, r * 1.05));
            painter.rect_stroke(bin, Rounding::same(2.0), stroke);
            painter.line_segment([pos2(c.x - r * 0.55, c.y - r * 0.55), pos2(c.x + r * 0.55, c.y - r * 0.55)], stroke);
            painter.line_segment([pos2(c.x - r * 0.25, c.y - r * 0.85), pos2(c.x + r * 0.25, c.y - r * 0.85)], stroke);
        }
        Screen::Apps => {
            let d = r * 0.42;
            for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let sq = Rect::from_center_size(pos2(c.x + dx * d * 0.95, c.y + dy * d * 0.95), Vec2::splat(d));
                painter.rect_stroke(sq, Rounding::same(1.5), stroke);
            }
        }
        Screen::Performance => {
            painter.add(Shape::line(
                vec![
                    pos2(c.x - r, c.y + r * 0.6),
                    pos2(c.x - r * 0.15, c.y - r * 0.1),
                    pos2(c.x + r * 0.35, c.y + r * 0.35),
                    pos2(c.x + r, c.y - r),
                ],
                stroke,
            ));
        }
        Screen::Protection => {
            painter.circle_stroke(c, r * 0.85, stroke);
            let handle = pos2(c.x + r * 0.75, c.y + r * 0.75);
            painter.line_segment([pos2(c.x + r * 0.35, c.y + r * 0.35), handle], stroke);
        }
        Screen::Malware => {
            painter.circle_stroke(c, r * 0.9, stroke);
            painter.line_segment([pos2(c.x - r * 0.35, c.y), pos2(c.x + r * 0.35, c.y)], stroke);
            painter.line_segment([pos2(c.x, c.y - r * 0.35), pos2(c.x, c.y + r * 0.35)], stroke);
        }
        Screen::Clutter => {
            painter.line_segment([pos2(c.x, c.y - r), pos2(c.x, c.y + r * 0.15)], stroke);
            painter.add(Shape::line(
                vec![
                    pos2(c.x - r * 0.55, c.y - r * 0.15),
                    pos2(c.x, c.y + r * 0.35),
                    pos2(c.x + r * 0.55, c.y - r * 0.15),
                ],
                stroke,
            ));
            painter.line_segment([pos2(c.x - r, c.y + r), pos2(c.x + r, c.y + r)], stroke);
        }
        Screen::Space => {
            painter.circle_stroke(c, r * 0.9, stroke);
            painter.line_segment([c, pos2(c.x, c.y - r * 0.9)], stroke);
            painter.line_segment([c, pos2(c.x + r * 0.75, c.y + r * 0.2)], stroke);
        }
        Screen::Cloud => {
            let cloud_c = pos2(c.x, c.y + r * 0.1);
            painter.circle_stroke(cloud_c + Vec2::new(-r * 0.45, 0.0), r * 0.45, stroke);
            painter.circle_stroke(cloud_c + Vec2::new(r * 0.35, -r * 0.05), r * 0.55, stroke);
            painter.line_segment(
                [pos2(c.x - r * 1.05, c.y + r * 0.35), pos2(c.x + r * 1.05, c.y + r * 0.35)],
                stroke,
            );
        }
        Screen::Assistant => {
            for (i, h) in [(-0.55, 0.45), (0.0, 0.85), (0.55, 0.6)] {
                let x = c.x + i * r;
                let bar = Rect::from_center_size(pos2(x, c.y + r * 0.15), Vec2::new(r * 0.22, r * h));
                painter.rect_filled(bar, Rounding::same(1.5), color);
            }
        }
    }
}

pub fn paint_brand_mark(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let gap = 5.0;
    for (i, h) in [0.55, 0.95, 0.45].iter().enumerate() {
        let x = c.x + (i as f32 - 1.0) * gap;
        let h = rect.height() * h;
        let r = Rect::from_center_size(pos2(x, c.y), Vec2::new(3.2, h));
        painter.rect_filled(r, Rounding::same(2.0), color);
    }
}

pub fn heading_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}
