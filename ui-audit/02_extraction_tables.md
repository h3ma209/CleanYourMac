# 02 — Mechanical extraction

Scope: `RustUtilityApp/crates/ui/src/main.rs`, `theme.rs`, `icons.rs`, and `CustomUtilityApp/frontend/src/renderer/styles.css`.

Tailwind classes (`bg-*`, `text-*`, `rounded-*`, `sm:`): NOT FOUND. No Tailwind config in either UI.

Named CSS colors: `white` once (`styles.css:60`). `Color32::WHITE` once (`theme.rs:47`). `Color32::TRANSPARENT` twice (`main.rs:1006`, `main.rs:2926`).

`hsl()` / `hsla()`: NOT FOUND.

---

## a) Colors

Hex in comments (not executed):

| value | count | example |
|---|---|---|
| `#121212` | 1 | `main.rs:25` (comment; equals `from_rgb(18,18,18)`) |
| `#1DB954` | 1 | `main.rs:25` (comment; equals `from_rgb(29,185,84)`) |

Opaque `Color32::from_rgb` (each definition count is 1; aliases point at these):

| rgb | hex | token | defined | notes |
|---|---|---|---|---|
| 18,18,18 | #121212 | `BG` | `main.rs:26` | central panel fill `main.rs:948` |
| 0,0,0 | #000000 | `SIDEBAR` | `main.rs:27` | never read (compiler warning) |
| 24,24,24 | #181818 | `ELEVATED` | `main.rs:28` | ghost/chip inactive fill |
| 40,40,40 | #282828 | `CARD` | `main.rs:29` | passed to `polish_style` `main.rs:232` |
| 51,51,51 | #333333 | `CARD_HOVER` | `main.rs:30` | row hover `main.rs:2924`; bubble idle `main.rs:2963` |
| 42,42,42 | #2A2A2A | `LINE` | `main.rs:31` | strokes, meter track `main.rs:2989` |
| 255,255,255 | #FFFFFF | `TEXT`, `GREEN_ON` | `main.rs:32`, `main.rs:35` | two literals, same triple |
| 179,179,179 | #B3B3B3 | `SUBTEXT` | `main.rs:33` | `MUTED` alias `main.rs:41` |
| 29,185,84 | #1DB954 | `GREEN` | `main.rs:34` | `LIME` alias `main.rs:42` |
| 240,180,41 | #F0B429 | `AMBER` | `main.rs:36` | |
| 232,112,106 | #E8706A | `ROSE` | `main.rs:37` | |
| 120,190,240 | #78BEF0 | `SKY` | `main.rs:38` | |
| 14,14,16 | #0E0E10 | (none) | `theme.rs:33` and `theme.rs:34` | window_fill and panel_fill |
| 10,10,12 | #0A0A0C | (none) | `theme.rs:35` | extreme_bg_color |
| 8,8,10 | #08080A | (none) | `theme.rs:107` | sidebar fill RGB, alpha 235 |

Alpha literals (count 1 unless noted):

| call | file:line |
|---|---|
| `from_black_alpha(170)` | `main.rs:839` scrim |
| `from_rgba_unmultiplied(255,255,255,18)` | `main.rs:1004` active nav fill |
| `from_rgba_unmultiplied(29,185,84,32)` | `main.rs:2922` selected row (hard-coded green, not `GREEN`) |
| `from_rgba_unmultiplied(0,0,0,120)` | `theme.rs:37` code bg |
| `from_rgba_unmultiplied(255,255,255,16)` | `theme.rs:43` widget hover |
| `from_rgba_unmultiplied(255,255,255,22)` | `theme.rs:71` glass stroke |
| `from_rgba_unmultiplied(255,255,255,10)` | `theme.rs:76` card fill |
| `from_black_alpha(55)` | `theme.rs:84` card shadow |
| accent alpha 16 / 55 | `theme.rs:90`, `theme.rs:93` hero |
| `from_black_alpha(70)` | `theme.rs:101` hero shadow |
| `from_rgba_unmultiplied(255,255,255,14)` | `theme.rs:108` sidebar stroke |
| `from_rgba_unmultiplied(255,255,255,8)` | `theme.rs:114` pill fill |
| accent alpha 18 / 90 | `theme.rs:122-123` notice |
| accent alpha 48 | `theme.rs:48` selection |
| `from_rgba_unmultiplied(color, 28)` and `(color, 90)` | `main.rs:2764-2765` badge |
| `from_rgba_unmultiplied(color, 34)` | `main.rs:2940` avatar |
| status dot alpha from `sin` | `main.rs:3023-3024` |

CSS hex (`styles.css`):

| value | count | example |
|---|---|---|
| `#b0b6be` | 2 | `styles.css:25`, `styles.css:83` |
| `#1a1d21` | 2 | `styles.css:42`, `styles.css:76` |
| `#2c3238` | 2 | `styles.css:66`, `styles.css:103` |
| `#101214` | 1 | `styles.css:4` |
| `#e8eaed` | 1 | `styles.css:5` |
| `#3c424a` | 1 | `styles.css:41` |
| `#2a6df4` | 1 | `styles.css:59` |
| `white` | 1 | `styles.css:60` |

### Distinct color counts

- Sable opaque triples: **15** (table above). `#FFFFFF` is stored twice (`TEXT` and `GREEN_ON`).
- Sable extra alpha-only uses of those hues: listed above; they are not new hues.
- Custom Utility hex (excluding `white`): **7**.
- Combined distinct hues across both UIs: **22** (15 + 7). No shared hex between the two palettes.

### Bypass of the named constants

Hard-coded where a constant already exists:

| literal | duplicates | file:line |
|---|---|---|
| `from_rgba_unmultiplied(29, 185, 84, 32)` | `GREEN` rgb | `main.rs:2922` |
| `Color32::from_rgb(14, 14, 16)` twice | no shared const with `BG` | `theme.rs:33-34` |
| `from_rgb(8, 8, 10)` | not `SIDEBAR` (`#000000`) | `theme.rs:107` |
| comment `#121212` / `#1DB954` | `BG` / `GREEN` | `main.rs:25` |

`IVORY`, `MUTED`, `LIME` are aliases (`main.rs:40-42`), not new colors. `LIME_INK` and `SIDEBAR` are unused.

### Values written once (literal count = 1)

Every `from_rgb` constant except `#FFFFFF` (count 2) and `#0E0E10` (count 2) is written once, then referenced by name. Literals that are not a reused const and appear once:

- scrim alpha 170 (`main.rs:839`)
- nav white alpha 18 (`main.rs:1004`)
- row green alpha 32 (`main.rs:2922`)
- each `theme.rs` alpha listed in the alpha table
- each Custom Utility hex with count 1: `#101214`, `#e8eaed`, `#3c424a`, `#2a6df4`

`#000000` (`SIDEBAR`) is written once and referenced **zero** times.

---

## b) Typography

Font families:

| family | count of registration | file:line |
|---|---|---|
| `sable_regular` → Proportional | 1 | `theme.rs:17-21` |
| `semibold` named family | 1 | `theme.rs:23-24` |
| `FontFamily::Monospace` | 1 | `theme.rs:67` (egui built-in, no bundled mono file) |
| `"Segoe UI", sans-serif` | 1 | `styles.css:3` |
| `ui-monospace, monospace` | 1 | `styles.css:104` |

Sable `.size(N)` / `FontId::new(N)` literal counts (script over the three `.rs` files):

| px | count | example |
|---|---|---|
| 13.0 | 19 | `main.rs:1204` |
| 12.0 | 13 | `main.rs:938` |
| 12.5 | 7 | `main.rs:2117` |
| 11.5 | 5 | `main.rs:995` |
| 28.0 | 4 | `theme.rs:55`, `main.rs:2684` |
| 16.0 | 3 | `main.rs:1289` |
| 11.0 | 3 | `main.rs:2138` |
| 38.0 | 2 | `main.rs:1252` |
| 15.5 | 1 | `main.rs:1270` |
| 26.0 | 1 | `main.rs:2109` |
| 22.0 | 1 | `main.rs:2282` |
| 18.0 | 1 | `main.rs:2571` (also brand `heading_font(18.0)` `main.rs:994`) |
| 36.0 | 1 | `main.rs:2615` |
| 15.0 | 1 | `main.rs:2856` |
| 14.5 | 1 | `theme.rs:58` |
| 14.0 | 1 | `theme.rs:61` (also `heading_font(14.0)` `main.rs:1032`) |
| 10.0 | 1 | `main.rs:2973` compact bubble label |

Sizes set without a `.size()` literal (still real sizes):

| px | where |
|---|---|
| 34 / 28 / 22 | `title_size()` `main.rs:151-158` |
| 14.5 / 13 | `subtitle_size()` `main.rs:161-162` |
| 44 / 30 | hero bytes `main.rs:1081` (`44` wide, `30` compact) |
| 12.5 | `TextStyle::Monospace` `theme.rs:67` |

Font weight: no numeric `font-weight`. Semibold is the named family (`theme.rs:52`). `.strong()` is used on many labels (example `main.rs:1084`). CSS `font-weight`: NOT FOUND in `styles.css`.

Line height: NOT FOUND (`line_height`, `leading`) in `crates/ui` and `styles.css`.

Letter spacing: NOT FOUND.

CSS font-size:

| value | count | file:line |
|---|---|---|
| `0.8rem` | 2 | `styles.css:84`, `styles.css:105` |
| `1.6rem` | 1 | `styles.css:20` |
| `0.85rem` | 1 | `styles.css:31` |
| `1.4rem` | 1 | `styles.css:90` |

Distinct Sable font sizes: **20** (see `00_summary.md`).

---

## c) Spacing

`add_space` argument counts in `main.rs`:

| value | count | example |
|---|---|---|
| `8.0` | 20 | `main.rs:922` |
| `6.0` | 16 | `main.rs:912` |
| `10.0` | 8 | `main.rs:991` |
| `12.0` | 7 | `main.rs:1376` |
| `14.0` | 6 | `main.rs:1113` |
| `4.0` | 5 | `main.rs:854` |
| `2.0` | 3 | `main.rs:918` |
| `18.0` | 1 | `main.rs:857` |
| `20.0` | 1 | `main.rs:2438` |
| `if layout.compact { 14.0 } else { 20.0 }` | 1 | `main.rs:910` |

Other spacing literals:

| value | what | file:line |
|---|---|---|
| item spacing (10, 10) | egui | `theme.rs:30` |
| button padding (16, 9) | egui | `theme.rs:31` |
| scroll bar width 6 | egui | `theme.rs:32` |
| content pad_x 14 / 20 / 32 | layout | `main.rs:128` |
| content pad_y 12 / 22 | layout | `main.rs:129` |
| sidebar margin x 10 or 16, y 18 | frame | `main.rs:906` |
| card inner margin 16 | glass_card | `theme.rs:79` |
| hero inner margin 22 | | `theme.rs:96` |
| pill margin (12, 7) | | `theme.rs:117` |
| notice margin (16, 12) | | `theme.rs:125` |
| confirm inner margin 26 | | `main.rs:850` |
| row margin (12, 8) | | `main.rs:2907` |
| nav margin top/bottom 8, left 14 or 10, right 6 or 10 | | `main.rs:1012-1017` |
| badge margin (10, 3) | | `main.rs:2767` |
| text field margin (10, 8) | | `main.rs:1330` |
| card bottom `add_space(4)` | | `main.rs:2824` |

CSS padding/margin/gap:

| value | count | file:line |
|---|---|---|
| `margin: 0` | 2 | `styles.css:9`, `styles.css:19` |
| padding `32px 20px 48px` | 1 | `styles.css:15` |
| margin `0 0 8px` | 1 | `styles.css:19` |
| margin `24px 0 12px` | 1 | `styles.css:30` |
| margin-top `6px` | 2 | `styles.css:38`, `styles.css:89` |
| padding `10px 12px` | 1 | `styles.css:39` |
| gap `8px` | 1 | `styles.css:49` |
| margin-bottom `20px` | 1 | `styles.css:50` |
| padding `10px 14px` | 1 | `styles.css:56` |
| gap `12px` | 1 | `styles.css:72` |
| padding `14px` | 1 | `styles.css:78` |
| margin `16px 0 0` | 1 | `styles.css:96` |
| padding `8px 0` | 1 | `styles.css:102` |

Tailwind `p-*` / `m-*` / `gap-*`: NOT FOUND.

Distinct Sable `add_space` numbers: **9** (2, 4, 6, 8, 10, 12, 14, 18, 20). Including padding literals that are not in that set: 3, 7, 9, 16, 22, 26, 32.

---

## d) Radius

`Rounding::same` in `main.rs`, `theme.rs`, `icons.rs`:

| px | count | example |
|---|---|---|
| 2.0 | 5 | `main.rs:1045` active bar |
| 500.0 | 4 | `main.rs:3049` chip; also `3070`, `3079`, `3091` buttons |
| 1.5 | 4 | `icons.rs:37` (icon corners, not UI chrome) |
| 20.0 | 2 | `main.rs:2766` badge; `theme.rs:116` pill |
| 10.0 | 2 | `main.rs:2908`, `main.rs:2928` row |
| 3.0 | 2 | `main.rs:2989`, `main.rs:2994` meter |
| 6.0 | 1 | `main.rs:1011` nav item |
| 9.0 | 1 | `main.rs:2939` avatar |
| 16.0 | 1 | `theme.rs:78` card |
| 18.0 | 1 | `theme.rs:95` hero |
| 14.0 | 1 | `theme.rs:124` notice |

CSS:

| value | count | file:line |
|---|---|---|
| `8px` | 2 | `styles.css:40`, `styles.css:55` |
| `10px` | 1 | `styles.css:77` |

Distinct Sable radii excluding icon-only 1.5: **10**. With 1.5: **11**.

---

## e) Shadow, blur, opacity

| property | value | file:line |
|---|---|---|
| shadow offset | (0, 6) | `theme.rs:81` |
| shadow blur | 14 | `theme.rs:82` |
| shadow spread | 0 | `theme.rs:83` |
| shadow color | black alpha 55 | `theme.rs:84` |
| shadow offset | (0, 8) | `theme.rs:98` |
| shadow blur | 18 | `theme.rs:99` |
| shadow color | black alpha 70 | `theme.rs:101` |

`box-shadow`, `backdrop-filter`, CSS `opacity`, CSS `blur`: NOT FOUND in `styles.css`.

Distinct shadows: **2**.

---

## f) Motion

| what | value | file:line |
|---|---|---|
| status dot alpha | `0.45 + 0.55 * abs(sin(time * 3.0))` when busy, else `1.0` | `main.rs:3023` |
| busy repaint | `Duration::from_millis(32)` | `main.rs:821` |
| `ui.spinner()` | egui default, no duration in app code | examples `main.rs:1692`, `main.rs:1830`, `main.rs:2251`, `main.rs:2893` |
| CSS `transition` / `@keyframes` / `animation` | NOT FOUND | `styles.css` |

Easing function name: NOT FOUND. Duration token: only the 32 ms repaint.

---

## g) z-order, breakpoints, fixed sizes

`z-index` CSS: NOT FOUND.

egui order (not numeric z-index):

| value | file:line |
|---|---|
| `Order::Middle` scrim | `main.rs:835` |
| `Order::Foreground` confirm window | `main.rs:848` |
| `Order::Tooltip` space hover card | `main.rs:2070` |

Breakpoints (`UiLayout::from_viewport`, `main.rs:124-129`):

| condition | sidebar_w | pad_x | pad_y |
|---|---|---|---|
| width `< 880` (`compact`) | 76 | 14 | 12 |
| width `< 1100` (`narrow`) | 228 | 20 | 22 |
| else | 268 | 32 | 22 |

`grid_cols` (`main.rs:141-148`): compact → 1 column; narrow → `min(prefer, 2)`; else `prefer`.

`@media` and `sm:`/`md:`/`lg:`: NOT FOUND.

Fixed widths/heights (points):

| value | file:line |
|---|---|
| window inner 1180 × 800 | `main.rs:215` |
| min inner 720 × 520 | `main.rs:216` |
| content_max clamp 280…980 | `main.rs:130` |
| confirm width 320 compact / 420 else | `main.rs:832` |
| search field desired width 320 | `main.rs:1329` |
| extensions field desired width 300 | `main.rs:1654` |
| apps list max height 290 | `main.rs:1339` |
| selection scroll max height 220 | `main.rs:2123` |
| space panes height 380 compact / 480 else | `main.rs:2378` |
| list/bubble scroll max height 420 | `main.rs:2418`, `main.rs:2505` |
| registry width 248 if content width > 780 and not compact | `main.rs:2371-2373` |
| hover card max width 360 | `main.rs:2074` |
| primary min 120 × 40 | `main.rs:3071` |
| ghost min 70 × 32 | `main.rs:3092` |
| chip min height 32, min width 0 | `main.rs:3050` |
| meter height 6, track width clamp 48…220 or caller width | `main.rs:2983`, `main.rs:2988` |
| Custom Utility `main` max-width 720px | `styles.css:13` |
| Custom Utility list max-height 220px | `styles.css:98` |

---

## Totals

| measure | Sable | Custom Utility |
|---|---|---|
| distinct opaque colors | 15 | 7 hex + `white` |
| distinct font sizes | 20 px | 4 rem |
| distinct radii | 11 (incl. icon 1.5 and pill 500) | 2 (`8px`, `10px`) |
| distinct shadows | 2 | 0 |
| distinct `add_space` values | 9 | n/a (CSS margins listed above) |
