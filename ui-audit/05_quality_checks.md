# 05 — Measurable quality checks

Contrast uses WCAG relative luminance. Pairs are colors the code actually assigns together. Glass fill white alpha 10 over `#121212` composites to about rgb(27, 27, 27). Sidebar fill is rgb(8, 8, 10) at alpha 235 (nearly opaque).

## Contrast

| pair | ratio | vs 4.5:1 | source |
|---|---|---|---|
| `#FFFFFF` on `#121212` | 18.73 | pass | `TEXT` / `BG` `main.rs:32`, `main.rs:26` |
| `#B3B3B3` on `#121212` | 8.93 | pass | `SUBTEXT` `main.rs:33` |
| `#1DB954` on `#121212` | 7.24 | pass | `GREEN` `main.rs:34` |
| `#FFFFFF` on `#1DB954` | **2.59** | **fail** (also under 3:1) | `primary` `main.rs:3068-3069`; active `chip` `main.rs:3046-3047` |
| `#F0B429` on `#121212` | 10.05 | pass | `AMBER` `main.rs:36` |
| `#E8706A` on `#121212` | 6.21 | pass | `ROSE` `main.rs:37` |
| `#78BEF0` on `#121212` | 9.30 | pass | `SKY` `main.rs:38` |
| `#FFFFFF` on `#181818` | 17.76 | pass | ghost text on `ELEVATED` `main.rs:3088-3089` |
| `#B3B3B3` on `#282828` | 7.03 | pass | disabled primary `main.rs:3076-3077` |
| `#FFFFFF` on glass-over-`#121212` (27,27,27) | 17.22 | pass | card text |
| `#B3B3B3` on (27,27,27) | 8.21 | pass | card secondary |
| `#1DB954` on (27,27,27) | 6.66 | pass | hero/card accent text |
| `#FFFFFF` on sidebar `#08080A` | 20.01 | pass | `theme.rs:107` |
| `#B3B3B3` on `#08080A` | 9.54 | pass | nav hints |
| `#E8EAED` on `#101214` | 15.58 | pass | `styles.css:4-5` |
| `#B0B6BE` on `#101214` | 9.19 | pass | `styles.css:25` |
| `#FFFFFF` on `#2A6DF4` | 4.57 | pass | `styles.css:59-60` |
| `#E8EAED` on `#1A1D21` | 14.03 | pass | input |
| `#FFFFFF` on `#2C3238` | 12.96 | pass | pause/resume/cancel |

Button text is 14 px (`theme.rs:61`). 14 px is below WCAG large-text size (18 pt, or 14 pt bold). 14 px is 10.5 pt. The 2.59:1 pair fails both 4.5:1 and 3:1.

Selected bubble paints `#FFFFFF` (`GREEN_ON`) as the label (`main.rs:2974`) while the circle fill is `#1DB954` (`main.rs:2963`). The label is placed at `rect.center_bottom()` (`main.rs:2966`), 8 px below the circle center (`main.rs:2962`), so part of the word can sit on the green circle when the diameter is large. Same 2.59:1 pair where they overlap.

## Click targets (desktop floor 32×32; mobile floor 44×44)

The app is desktop. `min_inner_size` width is 720 (`main.rs:216`), so a 390-wide layout is not reachable.

| control | coded min size | under 32×32? | under 44×44? | file:line |
|---|---|---|---|---|
| `primary` | 120 × 40 | no | height 40 < 44 | `main.rs:3071` |
| `ghost` | 70 × 32 | height equals 32 | height 32 < 44 | `main.rs:3092` |
| `chip` | width 0 × height 32 | width can be < 32 | yes | `main.rs:3050` |
| nav row, wide | icon 24 plus padding 8+8 and two text lines (14 + 11.5) | height from content > 32; width is the sidebar content width (268 − 32 margin) | height may be < 44 | `main.rs:1012-1027` |
| checkbox | size not set in app | unknown from app source; label is `""` | unknown | `main.rs:2429` |
| bubble click rect | `(diameter+8) × (diameter+22)`; diameter at share 0 is 36, times 0.72 = 25.92 in compact | compact + small share: width can be 25.92+8 = 33.9 (over 32) and the circle itself is under 32 | yes under 44 | `main.rs:2959-2961` |
| status pill | not a button (`Sense` not click) | n/a | n/a | `main.rs:3025` |
| Electron `button` | padding 10×14, no min-height | height is font + 20 px vertical padding; font is inherited `1.6rem` is h1 only; body is browser default (not set). NOT MEASURED at runtime | NOT MEASURED | `styles.css:53-56` |

## Focus

Search of `crates/ui/src/*.rs` for `focus`: no matches. No `Frame` stroke is applied on focus. egui 0.29 may draw its own focus ring; this repo does not configure one.

`styles.css`: no `:focus` or `:focus-visible` rule.

## Names for icon-only controls

- Compact nav: no text, only `on_hover_text` (`main.rs:1048`). Not an `aria-label` (egui has no aria attribute in this code). accesskit is a dependency of egui 0.29.1 (`Cargo.lock:890`) but this app does not set widget labels on the icon.
- Brand mark: `Sense::hover` only (`main.rs:982`, `main.rs:989`).
- Checkbox label `""` (`main.rs:2429`).
- Registry remove control is `ghost(ui, "×", …)` (`main.rs:2143`). The character is the label. No longer text.
- Empty-state icon is decorative beside a heading (`main.rs:2869-2872`).
- Electron: buttons have text content in `index.html:22-25`. Input has a wrapping `<label>` (`index.html:16-18`). No `img` elements. No icon-only buttons.

## Overflow at 390 px

NOT RUN at 390. `with_min_inner_size([720.0, 520.0])` (`main.rs:216`) refuses a 390-wide window. At the allowed compact width (`< 880`, sidebar 76): `content_max` clamp minimum is 280 (`main.rs:130`). Apps search `desired_width(320)` (`main.rs:1329`) is wider than a 280 content column. Extensions field `desired_width(300)` (`main.rs:1654`) is also wider than 280. Those two fields are the fixed widths that exceed the compact content minimum.

Space Lens pane height is 380 in compact (`main.rs:2378`) inside a window whose minimum height is 520 (`main.rs:216`), with header and setup card above the panes, so the panes plus chrome exceed 520. The central panel is a `ScrollArea` (`main.rs:952`), so the overflow scrolls rather than clipping the window.

Electron `.metrics` is `repeat(3, 1fr)` with no breakpoint (`styles.css:71`). At 390 px, three columns remain. NOT RUN, so the pixel overflow was not measured.

## 4 px grid

`add_space` and padding values that are not multiples of 4:

| px | example |
|---|---|
| 2 | `main.rs:918` |
| 3 | badge vertical margin `main.rs:2767` |
| 6 | `main.rs:912` (16 occurrences) |
| 7 | pill vertical margin `theme.rs:117` |
| 9 | button padding y `theme.rs:31` |
| 10 | `theme.rs:30` item spacing (multiple of 2, not of 4) |
| 14 | `main.rs:1113` |
| 18 | `main.rs:857` |
| 22 | hero margin `theme.rs:96`; pad_y `main.rs:129` |
| 26 | confirm margin `main.rs:850` |

## Counts vs the stated ranges

| measure | count | note in the request |
|---|---|---|
| Sable radii | 11 including 1.5 and 500 | "healthy is roughly 4–6" |
| Sable shadows | 2 | "roughly 3–4" |
| Sable font sizes | 20 | "roughly 6–8" |

## Line length and line height

Line height: NOT FOUND in source. Cannot compute a ratio.

Character counts of single-line body strings (spaces included):

| chars | string | file:line |
|---|---|---|
| 88 | "A quick health check of your Mac. This page only looks. It never changes or deletes anything." | `main.rs:1063` |
| 178 | cleanup subtitle | `main.rs:1235` |
| 171 | apps subtitle | `main.rs:1314` |
| 222 | threat disclaimer | `main.rs:1786` |
| 156 | file finder subtitle | `main.rs:1601` |

Those are one `RichText` each, so they wrap to the content width (max 980, `main.rs:130`) rather than staying on one line. The 90-character figure is the string length, not a measured line box. Wrapped line length in px was not measured from the PNG.

## Alignment

Same button row uses different min heights: `primary` 40 (`main.rs:3071`) beside `ghost` 32 (`main.rs:3092`). File Finder places them in one `horizontal` (`main.rs:1663`). The 8 px difference is in the code, not measured off the PNG.

`page_header` ends with `add_space(12)` (`main.rs:2743`). `safety` ends with `add_space(6)` (`main.rs:2718`). Tune-Up calls `page_header` then `banner` and does not call page-level `safety` (`main.rs:1422-1428`), so the gap under its header is 12, while Overview is 12 + pill + 6.

Active nav bar is drawn at `left_top - (4, 0)` (`main.rs:1042`), outside the row’s left padding. The bar is 3 px wide (`main.rs:1043`).

## Missing states by screen

| screen | loading | empty | error |
|---|---|---|---|
| Overview | present `main.rs:1111` | absent | partial (`banner` only) |
| Free Up Space | present `main.rs:1242` | present `main.rs:1242` | partial |
| Apps | present `main.rs:1321` | present `main.rs:1321`, `1365`, `1378`, `1386` | partial |
| Tune-Up | present `main.rs:1566` | present `main.rs:1566` | partial |
| File Finder | present `main.rs:1689` | present `main.rs:1735` | partial (error count stat `main.rs:1713`, no error card) |
| Threat Scan | present `main.rs:1827` | present `main.rs:1838`, `1861` | present strings `main.rs:597`, `main.rs:610` via `banner` |
| Big Downloads | present `main.rs:1926` | present `main.rs:1928` | partial |
| Space Lens | present `main.rs:2262` | present `main.rs:2328`, `2305` | present strings `main.rs:659`, `668`, `699` |
| Cloud | present `main.rs:2546` | present `main.rs:2546` | partial |
| System Monitor | present `main.rs:2602` | present `main.rs:2602` | absent (failure becomes a zero sample `main.rs:522`) |
| Confirm | n/a | n/a | n/a |
| Custom Utility | absent (no spinner in HTML/CSS) | partial (empty `ul`) | present text `renderer.ts:56` |
