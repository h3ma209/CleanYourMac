# 00 — Summary

Audit date: 2026-10-12. Source of truth is the files cited below. No source files were edited.

## Phase 1 — project facts

Two UIs exist.

### A. Sable (primary)

- Manifest: `RustUtilityApp/Cargo.toml` (workspace members `crates/engine`, `crates/ui`). UI package: `RustUtilityApp/crates/ui/Cargo.toml`.
- Package name `ui`, version `0.1.0`, edition `2021`, description "egui shell for the scan engine" (`crates/ui/Cargo.toml:2-5`).
- Framework: **eframe 0.29.1** (`RustUtilityApp/Cargo.lock:850-851`), which embeds **egui 0.29.1** (`Cargo.lock:885-886`). `Cargo.toml` requests `eframe = "0.29"` (`crates/ui/Cargo.toml:9`).
- Language: Rust. Async: `tokio` 1 with `rt-multi-thread`, `sync` (`crates/ui/Cargo.toml:11`).
- Styling method: egui `Style` + `Frame`. No CSS, no Tailwind, no styled-components.
- Component library: NOT FOUND. Widgets are functions in `crates/ui/src/main.rs`.
- Icon library: NOT FOUND. Icons are painted in `crates/ui/src/icons.rs` (`Stroke::new(1.6, …)` at `icons.rs:29`, screen icons `Stroke::new(1.75, …)` at `icons.rs:94`).
- Font source: embedded files, not Google Fonts, not `@font-face`, not system UI font.
  - `include_bytes!("../assets/SableUI-Regular.ttf")` (`theme.rs:4`)
  - `include_bytes!("../assets/SableUI-SemiBold.ttf")` (`theme.rs:5`)
  - Files exist: `RustUtilityApp/crates/ui/assets/SableUI-Regular.ttf`, `SableUI-SemiBold.ttf`.
  - Family names registered: `"sable_regular"` on `FontFamily::Proportional` (`theme.rs:17-21`), `"semibold"` (`theme.rs:23-24`).
- Animation library: NOT FOUND. Motion in app code is `ui.spinner()` (egui) and a sine alpha on the status dot (`main.rs:3023`). Repaint interval `32` ms while busy (`main.rs:821`).
- Routing library: NOT FOUND. Screen state is `enum Screen` (`main.rs:46-57`) plus `ScanApp.screen`. No URL router.
- State: `ScanApp` struct (`main.rs:236`). No Redux/Zustand.

UI files that render:

```
RustUtilityApp/crates/ui/src/main.rs
RustUtilityApp/crates/ui/src/theme.rs
RustUtilityApp/crates/ui/src/icons.rs
```

Search `find RustUtilityApp/crates/ui -type f` (excluding `target/`): those three plus `Cargo.toml` and the two `.ttf` files. No `.tsx`, `.jsx`, `.vue`, `.svelte`, `.html`, `.css`, `.scss`, `.swift`, `.kt` under `crates/ui`.

Searched for token filenames `tailwind.config.*`, `variables.css`, `globals.css`, `tokens.json` under `RustUtilityApp/crates/ui`: NOT FOUND. Token-like sources that do exist: `theme.rs` (full file in `01_tokens_source.md`) and color constants `main.rs:25-43`.

### B. Custom Utility (secondary Electron shell)

- `CustomUtilityApp/package.json`: NOT FOUND.
- `CustomUtilityApp/frontend/package.json`: name `custom-utility-app`, version `0.1.0`, `electron` `^33.2.0`, `typescript` `^5.7.2` (`package.json:2-14`). Start script: `"start": "npm run build && electron ."` (`package.json:9`).
- Styling: plain CSS `frontend/src/renderer/styles.css`. Component library: NOT FOUND. Icon library: NOT FOUND. Font: `"Segoe UI", sans-serif` (`styles.css:3`). Animation library: NOT FOUND in `styles.css` (no `transition`, `@keyframes`, or animation property).
- UI files:

```
CustomUtilityApp/frontend/src/renderer/index.html
CustomUtilityApp/frontend/src/renderer/styles.css
CustomUtilityApp/frontend/src/renderer/renderer.ts
CustomUtilityApp/frontend/src/main.ts
CustomUtilityApp/frontend/src/preload.ts
```

## Phase 3 — run

Command that was run:

```
cd /Users/hema/Desktop/Experiments/crackmed/RustUtilityApp && cargo run -p ui
```

Process printed `Running …/debug/ui`. Window owner name `ui`, title `Sable`. First bounds from `CGWindowListCopyWindowInfo`: X=86, Y=35, Width=1180, Height=828 (inner size requested `1180×800` at `main.rs:215`; extra 28 pt is the macOS title bar). Capture tool: `screencapture -l <window id>`.

Playwright: NOT USED. This UI is not a web route table. `min_inner_size` is `[720.0, 520.0]` (`main.rs:216`), so a 390×844 viewport cannot be created. A 1440×900 window was not set.

During later clicks the window bounds changed to X=0, Y=30, Width=1352, Height=848. PNGs are all **2704×1696 pixels** (retina backing store). Filename suffix `_1180` is the first window’s point width, not the PNG pixel size.

Custom Utility: NOT RUN. Command not executed: `cd CustomUtilityApp/frontend && npm start`.

States not captured as PNGs: loading spinner, error banner, confirm modal, hover, focus, 390-wide, separate light theme. Light theme: NOT FOUND in Sable code (only dark constants).

## Ten measured findings (Phases 2 and 6)

1. **15** distinct opaque RGB triples in Sable (`main.rs:26-38` and `theme.rs:33-35,107`). **7** more hex colors in `styles.css` that do not match those triples. **2** constants are never read: `SIDEBAR` (`main.rs:27`, compiler warning during `cargo run`) and `LIME_INK` (`main.rs:43`, compiler warning).
2. **20** distinct font sizes in Sable (px): 10, 11, 11.5, 12, 12.5, 13, 14, 14.5, 15, 15.5, 16, 18, 22, 26, 28, 30, 34, 36, 38, 44. Custom Utility adds 4 rem sizes: `1.6rem`, `0.85rem`, `0.8rem`, `1.4rem` (`styles.css:20,31,84,90`).
3. **11** distinct `Rounding::same` values in Sable: 1.5, 2, 3, 6, 9, 10, 14, 16, 18, 20, 500. Custom Utility adds `8px` and `10px` (`styles.css:40,77`).
4. **2** shadow definitions: blur 14 / offset (0, 6) / black alpha 55 (`theme.rs:80-85`); blur 18 / offset (0, 8) / black alpha 70 (`theme.rs:97-102`). Backdrop-filter: NOT FOUND.
5. **White `#FFFFFF` on green `#1DB954` contrast is 2.59:1** (primary button, `main.rs:3068-3069`). WCAG AA needs 4.5:1 for this size (`TextStyle::Button` is 14 px, `theme.rs:61`). 2.59 is also under 3:1.
6. **`#B3B3B3` on `#121212` is 8.93:1** (`SUBTEXT` `main.rs:33` on `BG` `main.rs:26`). Passes 4.5:1. Green `#1DB954` on `#121212` is 7.24:1.
7. Desktop click-target floor used here: 32×32. `ghost` min size is **70×32** (`main.rs:3092`). `primary` min size is **120×40** (`main.rs:3071`). `chip` min size is **(0, 32)** (`main.rs:3050`), so width is not floored at 32. Checkbox: no size in app code (`main.rs:2429`); app does not set `interact_size`.
8. App source contains **0** assignments to a focus stroke or `aria` label. Search pattern `focus|aria` in `crates/ui`: no focus style. Icons use `Sense::hover` (`icons.rs:20`). Compact nav exposes the name only via `on_hover_text` (`main.rs:1048`).
9. Spacing literals that are not multiples of 4: **2, 3, 6, 7, 9, 10, 14, 18, 22, 26** (examples: `add_space(2.0)` `main.rs:918`, `add_space(6.0)` `main.rs:912`, button padding y **9** `theme.rs:31`, pill padding y **7** `theme.rs:117`, badge padding y **3** `main.rs:2767`, confirm margin **26** `main.rs:850`).
10. Subtitle strings on Overview are one paragraph of **88 characters** (`main.rs:1063`: "A quick health check of your Mac. This page only looks. It never changes or deletes anything."). Threat-scan disclaimer is **222 characters** in one `RichText` (`main.rs:1786`). Line-height is never set in app code (NOT FOUND: `line_height` / `leading` in `crates/ui`).
