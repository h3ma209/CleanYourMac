# 07 — Change surface

Facts about what a visual edit would touch. No redesign is proposed here.

## Must not change if behavior stays

These files own scan, trash, and data flow. They contain no widget layout.

- `RustUtilityApp/crates/engine/` (entire crate). UI calls it from `main.rs` (`main.rs:10-17` imports).
- Inside `main.rs`, the methods that talk to the engine rather than paint: `start_scan` `main.rs:351`, `start_care` `main.rs:386`, `start_junk` `main.rs:399`, `trash` `main.rs:414`, `run_tool` `main.rs:449`, `start_clutter` `main.rs:465`, `trash_download` `main.rs:480`, `start_cloud` `main.rs:500`, `sample_machine` `main.rs:515`, `survey_selected` `main.rs:531`, `load_apps` `main.rs:553`, `trash_space` `main.rs:568`, `start_threat` `main.rs:589`, `trash_threat` `main.rs:616`, `ignore_threat` `main.rs:636`, `start_space` `main.rs:651`, `perform` `main.rs:712`, `drain` `main.rs:722`.
- `Screen` enum and `auto_loads` (`main.rs:46-110`). There is no route table to preserve; this enum is the navigation model.
- `UiMsg` (`main.rs:193-206`).
- Electron behavior: `renderer.ts` event handling (`renderer.ts:38-88`), `preload.ts`, `main.ts`. `styles.css` is not in this list.

## Files that own styling

| file | what it owns |
|---|---|
| `crates/ui/src/theme.rs` | fonts, `item_spacing`, `button_padding`, egui widget colors, glass frames (fill, stroke, radius, shadow, margin) |
| `crates/ui/src/main.rs` lines 25–43 | named colors used by widgets |
| `crates/ui/src/main.rs` lines 123–163 | breakpoints and title sizes |
| `crates/ui/src/main.rs` lines 2680–3094 | widget functions (`primary`, `ghost`, `chip`, `row`, `card`, `page_header`, …) |
| `crates/ui/src/icons.rs` | stroke widths and icon geometry |
| `crates/ui/assets/SableUI-Regular.ttf`, `SableUI-SemiBold.ttf` | the only bundled faces |
| `CustomUtilityApp/frontend/src/renderer/styles.css` | all Electron styling |

Screen functions in `main.rs` (from `care_screen` `main.rs:1058` through `assistant_screen` `main.rs:2676`) also set per-screen font sizes and `add_space` calls. Those are styling mixed into the same file as the engine calls.

## If only the theme file changes

`theme.rs` is the only file named like a theme. `apply_theme` passes colors in from `main.rs` (`main.rs:232`: `polish_style(&mut style, GREEN, TEXT, CARD)`). Editing `theme.rs` without editing `main.rs` therefore:

- Updates every `glass_card`, `glass_hero`, `glass_sidebar`, `glass_pill`, and `glass_notice` (fill, radius, shadow, padding). Those frames wrap every card, the sidebar, safety pills, and notices.
- Updates font family and the five `TextStyle` sizes (`theme.rs:55-67`) for text that does not pass its own `.size()`.
- Updates egui widget hover/active/selection (`theme.rs:43-49`) where a widget does not pass `.fill()`.
- Does not change `primary` / `ghost` / `chip` fills. Those call `.fill(GREEN)` or `.fill(ELEVATED)` in `main.rs:3047-3089`.
- Does not change `BG` on the central panel (`main.rs:948`).
- Does not change the 20 explicit `.size(…)` overrides in screen functions.
- Does not change Custom Utility at all.

Estimate: **about 40%** of Sable’s painted surface moves if only `theme.rs` changes. Reasoning: every card, hero, sidebar, and pill is a `theme.rs` frame (large area), but the brand green, page background, button fills, and most font sizes are literals in `main.rs`. Editing `theme.rs` plus the 13 `from_rgb` constants at `main.rs:26-38` covers those fills too. The leftovers are one-off alphas: scrim `main.rs:839`, nav active `main.rs:1004`, selected row `main.rs:2922`, badge alphas `main.rs:2764-2765`, avatar alpha `main.rs:2940`. That pair of edits is **about 90%** of Sable color. The other **about 10%** is those alphas plus per-screen `.size()` numbers. Electron stays at 0% until `styles.css` is edited.

## Highest-leverage files for a visual pass

Ranked by how many call sites read them:

1. `crates/ui/src/main.rs` — every screen and every widget function (3183 lines).
2. `crates/ui/src/theme.rs` — frames, type ramp, widget chrome (126 lines).
3. `crates/ui/src/main.rs` widget block `2680-3094` — `primary` (8 calls), `ghost` (32), `card` (71), `page_header` (10).
4. `crates/ui/src/main.rs` constants `25-43` — every `GREEN` / `TEXT` / `BG` use.
5. `crates/ui/src/icons.rs` — every nav and safety icon.
6. `crates/ui/assets/SableUI-Regular.ttf` and `SableUI-SemiBold.ttf` — both families (`theme.rs:4-5`).
7. `crates/ui/src/main.rs` `UiLayout` `123-163` — sidebar 76/228/268 and title 22/28/34.
8. `CustomUtilityApp/frontend/src/renderer/styles.css` — 100% of that shell.
9. `CustomUtilityApp/frontend/src/renderer/index.html` — structure only; colors live in the CSS.
10. `crates/ui/src/main.rs` `confirm_modal` `825-878` — the only overlay.

## Limits

- egui 0.29.1 (`Cargo.lock:885`) is immediate mode. There is no CSS cascade and no theme JSON loader in this repo.
- Radius 500 on buttons (`main.rs:3049`) is an egui `Rounding`, not a CSS `999px`, but it is the same idea: pill shape is per call, not a token name.
- Window cannot be narrower than 720 or shorter than 520 (`main.rs:216`).
- Icons are drawn with `Painter` strokes (`icons.rs:29`, `icons.rs:94`). There is no SVG icon set to swap.
- Monospace text uses `FontFamily::Monospace` (`theme.rs:67`) with no bundled mono file.
- Electron buttons are native-in-CSS only; no component library locks their radius. Their radius is `8px` in one rule (`styles.css:55`).
- macOS traffic-light title bar is outside egui. The captured window was 28 pt taller than the inner size (828 vs 800).
