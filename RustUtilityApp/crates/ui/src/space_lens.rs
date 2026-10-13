//! Squarified treemap layout for Space Lens. Area is strictly proportional to bytes.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

use crate::tokens::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpaceCategory {
    User,
    Apps,
    System,
    Caches,
    Other,
    Free,
    Unaccounted,
}

impl SpaceCategory {
    pub fn color(self) -> Color32 {
        match self {
            SpaceCategory::User => Color32::from_rgb(0x5B, 0xB8, 0xFF),
            SpaceCategory::Apps => Color32::from_rgb(0x7A, 0xB6, 0xFF),
            SpaceCategory::System => Color32::from_rgb(0xA6, 0xAB, 0xC2),
            SpaceCategory::Caches => Color32::from_rgb(0x86, 0xF0, 0xB0),
            SpaceCategory::Other => Color32::from_rgb(0xD0, 0xA0, 0xFF),
            SpaceCategory::Free => Color32::from_rgb(0x38, 0x38, 0x50),
            SpaceCategory::Unaccounted => Color32::from_rgb(0x6E, 0x32, 0xD4),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SpaceCategory::User => "User data",
            SpaceCategory::Apps => "Applications",
            SpaceCategory::System => "System",
            SpaceCategory::Caches => "Caches",
            SpaceCategory::Other => "Other",
            SpaceCategory::Free => "Free space",
            SpaceCategory::Unaccounted => "Not scanned",
        }
    }
}

pub fn category_for(path: &Path, home: &Path) -> SpaceCategory {
    let text = path.to_string_lossy();
    if text.contains("/Applications") || text.ends_with(".app") {
        return SpaceCategory::Apps;
    }
    if text.contains("/Library/Caches") || text.contains("/Caches/") {
        return SpaceCategory::Caches;
    }
    if text.starts_with("/System") || text.starts_with("/usr") || text.starts_with("/private") {
        return SpaceCategory::System;
    }
    if path.starts_with(home) {
        return SpaceCategory::User;
    }
    SpaceCategory::Other
}

pub fn is_hidden_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

pub fn is_protected(path: &Path, home: &Path) -> bool {
    let text = path.to_string_lossy();
    if text == "/System" || text.starts_with("/System/")
        || text == "/usr" || text.starts_with("/usr/")
        || text == "/private" || text.starts_with("/private/")
        || text == "/Library" || text.starts_with("/Library/")
    {
        return true;
    }
    if path.starts_with(home.join("Library")) {
        return false;
    }
    false
}

/// One treemap tile. `key` is `Some(path)` or synthetic id for free/unaccounted/other.
#[derive(Clone)]
pub struct TreemapTile {
    pub key: Option<PathBuf>,
    pub label: String,
    pub bytes: u64,
    pub category: SpaceCategory,
    pub is_directory: bool,
    pub unreadable: bool,
}

/// Slice-and-dice treemap. Returns one rect per item; empty if total bytes is 0.
pub fn layout_treemap(items: &[u64], area: Rect) -> Vec<Rect> {
    let total: u64 = items.iter().sum();
    if total == 0 || items.is_empty() || area.width() < 2.0 || area.height() < 2.0 {
        return vec![Rect::from_min_max(Pos2::ZERO, Pos2::ZERO); items.len()];
    }
    let mut rects = vec![Rect::from_min_max(Pos2::ZERO, Pos2::ZERO); items.len()];
    let mut stack: Vec<(usize, usize, Rect)> = vec![(0, items.len(), area)];
    while let Some((start, end, rect)) = stack.pop() {
        if start >= end {
            continue;
        }
        if end - start == 1 {
            rects[start] = rect;
            continue;
        }
        let slice: u64 = items[start..end].iter().sum();
        if slice == 0 {
            for i in start..end {
                rects[i] = rect;
            }
            continue;
        }
        let horizontal = rect.width() >= rect.height();
        let mut i = start;
        let mut acc = 0u64;
        let half_target = slice / 2;
        while i < end {
            acc += items[i];
            i += 1;
            if acc >= half_target || i == end {
                break;
            }
        }
        let split = i.min(end);
        let first_sum: u64 = items[start..split].iter().sum();
        let ratio = first_sum as f32 / slice as f32;
        let (first_rect, second_rect) = if horizontal {
            let w1 = rect.width() * ratio;
            (
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + w1, rect.max.y)),
                Rect::from_min_max(Pos2::new(rect.min.x + w1, rect.min.y), rect.max),
            )
        } else {
            let h1 = rect.height() * ratio;
            (
                Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + h1)),
                Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + h1), rect.max),
            )
        };
        stack.push((split, end, second_rect));
        stack.push((start, split, first_rect));
    }
    rects
}

pub fn fit_label(painter: &egui::Painter, text: &str, max_w: f32, font: egui::FontId) -> String {
    if max_w < 24.0 {
        return String::new();
    }
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), WHITE);
    if galley.size().x <= max_w {
        return text.to_owned();
    }
    for n in (4..text.len()).rev() {
        let short = format!("{}…", &text[..n]);
        let g = painter.layout_no_wrap(short.clone(), font.clone(), WHITE);
        if g.size().x <= max_w {
            return short;
        }
    }
    String::new()
}

pub fn disk_bar_color(fraction: f32) -> Color32 {
    if fraction >= 0.95 {
        DANGER
    } else if fraction >= 0.85 {
        WARN
    } else {
        SUCCESS
    }
}

pub fn thick_meter(ui: &mut egui::Ui, fraction: f32, width: f32) {
    let h = S8;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), egui::Sense::hover());
    ui.painter().rect_filled(rect, r_xs(), with_alpha(WHITE, 40));
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.0 {
        let mut fill = rect;
        fill.set_width((rect.width() * f).max(S4));
        ui.painter().rect_filled(fill, r_xs(), disk_bar_color(f));
    }
}
