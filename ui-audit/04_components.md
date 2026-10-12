# 04 — Component spec

Call-site counts are `name(` matches in `main.rs` excluding the `fn` definition line. All paths are under `RustUtilityApp/crates/ui/src/` unless noted.

Hover/focus: buttons are `egui::Button`. App code sets fill, stroke, rounding, and `min_size`. It does not set a focus ring. Widget hover/active from `polish_style` (`theme.rs:43-47`) apply to egui widgets that use the default style. `primary` / `ghost` / `chip` pass their own `.fill()`, which replaces the widget fill.

---

### Component: primary

- File: `main.rs:3065-3083`   Used in: 8 call sites
- Variants and sizes: enabled and disabled. One size: `min_size(120, 40)` (`main.rs:3071`, `main.rs:3080`).
- Anatomy: height at least 40; egui `button_padding` (16, 9) (`theme.rs:31`) is extra inside the label if the min size does not already cover it. Radius 500 (`main.rs:3070`). Border: enabled has no `.stroke()` in this function; disabled stroke 1 px `LINE` `#2A2A2A` (`main.rs:3078`). Background enabled `#1DB954` (`main.rs:3069`); disabled `#282828` (`main.rs:3077`). Text: enabled `#FFFFFF` `.strong()`, `TextStyle::Button` 14 px semibold (`theme.rs:61`). Disabled text `#B3B3B3` (`main.rs:3076`). Icon: none. Gap: n/a.
- States:
  - default: fill `#1DB954`, text `#FFFFFF`
  - hover: not set in `primary`. egui may still tint; app does not override hover for this button
  - active: not set here
  - focus: NOT FOUND in this function
  - disabled: `add_enabled(false, …)` fill `#282828`, stroke `#2A2A2A`, text `#B3B3B3` (`main.rs:3074-3081`)
  - loading: NOT FOUND (callers pass `enabled = !busy`)
  - error: NOT FOUND
- Inconsistencies: text on `#1DB954` is `#FFFFFF`. Contrast 2.59:1 (`05_quality_checks.md`).

### Component: ghost

- File: `main.rs:3085-3094`   Used in: 32 call sites
- Variants and sizes: one size, `min_size(70, 32)` (`main.rs:3092`). Enabled flag only.
- Anatomy: height at least 32; radius 500; border 1 px `#2A2A2A`; fill `#181818`; text `#FFFFFF` not `.strong()` (`main.rs:3088-3091`). Icon none.
- States:
  - default: fill `#181818`, text `#FFFFFF`, stroke `#2A2A2A`
  - hover / active / focus: NOT FOUND in this function
  - disabled: `add_enabled(false)` (`main.rs:3086`); fill and text colors are the same `RichText` as enabled (`main.rs:3088`). Disabled look is egui’s default fade, not a second color in this function.
  - loading / error: NOT FOUND
- Inconsistencies: same pill radius 500 as `primary` and `chip`, but height 32 vs primary 40 vs chip min height 32 with min width 0.

### Component: primary_icon

- File: `main.rs:3054-3062`   Used in: 1 (`main.rs:2241`)
- Variants: wraps `primary`. Icon box 15×15 (`main.rs:3058`), gap 6 (`main.rs:3057`). Icon color `#FFFFFF` if enabled else `#B3B3B3` (`main.rs:3055`).
- States: same as `primary`. The icon is not inside the button widget; it sits in a horizontal row beside it (`main.rs:3056`).

### Component: chip

- File: `main.rs:3044-3051`   Used in: 5 call sites (each inside a loop of labels)
- Variants: `active: bool`. `min_size(0, 32)` (`main.rs:3050`).
- Anatomy: height ≥ 32; width = text + button padding (16, 9), no minimum width. Radius 500. Border 1 px: active `#1DB954`, inactive `#2A2A2A` (`main.rs:3048`). Fill active `#1DB954`, inactive `#181818` (`main.rs:3047`). Text 13 px (`main.rs:3046`): active `#FFFFFF`, inactive `#FFFFFF` (`TEXT`).
- States: active vs inactive only. Hover/focus/disabled: NOT FOUND in `chip`. Callers also ignore clicks when `busy` (`main.rs:1628`).
- Inconsistencies: active chip text is `#FFFFFF` on `#1DB954`, same 2.59:1 pair as `primary`.

### Component: field

- File: `main.rs:3034-3041`   Used in: 3
- Variants: `interactive(enabled)` (`main.rs:3038`).
- Anatomy: `TextEdit::singleline`, `desired_width` = available width, margin (10, 8) (`main.rs:3039`), font `TextStyle::Monospace` 12.5 px (`theme.rs:67`). No radius or fill in this function (egui widget visuals from `theme.rs:38-46`: inactive fill `CARD` `#282828`, stroke glass white alpha 22).
- States: disabled via `interactive(false)`. Error style: NOT FOUND. Focus style: NOT FOUND in app code.

### Component: TextEdit (search and extensions)

- File: inline, not a shared function. Apps search `main.rs:1327-1330` (width 320, margin 10×8, hint `"Search by app name"`, proportional font). Extensions `main.rs:1652-1658` (width 300, monospace, hint `"dmg, pkg, zip"`).
- Inconsistencies: three text inputs: `field` (mono, full width), apps search (proportional, 320), extensions (mono, 300).

### Component: card

- File: `main.rs:2819-2825`   Used in: 71 call sites. Frame: `theme::glass_card` (`theme.rs:74-86`).
- Anatomy: fill white alpha 10, stroke 1 px white alpha 22, radius 16, inner margin 16, shadow offset (0, 6) blur 14 black alpha 55. Then `add_space(4)` (`main.rs:2824`).
- States: none. Not interactive by itself.

### Component: hero

- File: `main.rs:2812-2817`   Used in: 4. Frame `glass_hero(GREEN)` (`theme.rs:88-103`).
- Anatomy: fill green alpha 16, stroke green alpha 55, radius 18, margin 22, shadow (0, 8) blur 18 black alpha 70.

### Component: page_header

- File: `main.rs:2721-2744`   Used in: 10 (every screen function)
- Anatomy: `glass_card` plus icon 40×40 (`main.rs:2724`), gap 12 (`main.rs:2726`), `"SABLE"` 11 px `#1DB954` strong, title `heading_font(title_size)`, subtitle `subtitle_size` `#B3B3B3`. Bottom space 12 (`main.rs:2743`).
- States: none.

### Component: section / label_small

- File: `section` `main.rs:2746-2748` (7 calls); `label_small` `main.rs:2751-2753` (13 calls)
- Anatomy: `section` adds 4 px then `label_small`. Text 11 px, `#B3B3B3`, `.strong()` (`main.rs:2752`).

### Component: step_label

- File: `main.rs:2755-2759`   Used in: 5
- Anatomy: `badge` with the step number in `LIME`, then title `heading_font(15)` (`main.rs:2758`). Gap is egui item spacing 10 (`theme.rs:30`).

### Component: badge

- File: `main.rs:2762-2771`   Used in: 3 direct calls plus `step_label`
- Anatomy: fill accent alpha 28, stroke 1 px accent alpha 90, radius 20, margin (10, 3) (`main.rs:2764-2767`). Text 12 px in the accent color (`main.rs:2769`).
- Variants: color argument (`LIME`, `SKY`, `ROSE` at call sites).

### Component: safety

- File: `main.rs:2709-2718`   Used in: 9 direct calls, plus inside every `tool_card`
- Variants:
  - ReadOnly: icon `Eye`, text `"Read-only — nothing on disk is changed"`, `#1DB954` (`main.rs:2711`)
  - Trash: icon `Trash`, `"Moves to Trash — you can undo this"`, `#F0B429` (`main.rs:2712`)
  - Admin: icon `Lock`, `"May ask for your Mac password"`, `#E8706A` (`main.rs:2713`)
- Anatomy: `glass_pill` (`theme.rs:112-118`): fill white alpha 8, radius 20, margin (12, 7). Icon box 16×16, label 12.5 px (`icons.rs:19-22`). Then `add_space(6)` (`main.rs:2718`).

### Component: banner

- File: `main.rs:2773-2809`   Used in: 10 (once per screen function except it is not called from `space_screen` — `space_screen` does call `banner` at `main.rs:2191`)
- Variants: `notice.ok` true → accent `LIME`, heading `"Done"`, icon Check. False → `ROSE`, heading `"That did not work"`, icon Alert (`main.rs:2777`, `main.rs:2793`).
- Anatomy: `glass_notice` radius 14, margin (16, 12) (`theme.rs:120-126`). Icon 22×22 (`main.rs:2783`). Gap 8 (`main.rs:2790`). Heading 14 px semibold. Body `#FFFFFF` via `IVORY` (`main.rs:2797`). Dismiss is `ghost("Dismiss")` (`main.rs:2800`).
- States: this is the success and error surface. No hover style of its own.

### Component: empty

- File: `main.rs:2865-2878`   Used in: call sites of `empty(` plus `loading_card` idle path
- Anatomy: `card`, `add_space(14)` top and bottom (`main.rs:2867`, `main.rs:2876`), info icon 28×28 `#B3B3B3` (`main.rs:2869`), gap 6, heading 16 px semibold (`main.rs:2872`), gap 2, body `#B3B3B3`.
- States: static.

### Component: loading_line / loading_card

- File: `loading_line` `main.rs:2880-2885` (1 call). `loading_card` `main.rs:2888-2902` (8 calls).
- Anatomy: `loading_line` is `ui.spinner()` plus `#B3B3B3` text. `loading_card` if busy: card with 12 px space, centered spinner, 6 px, muted text, 12 px (`main.rs:2891-2897`). If not busy and `idle` is non-empty: `empty("Nothing here", idle)` (`main.rs:2900`).
- States: loading vs idle empty. No error variant.

### Component: row

- File: `main.rs:2904-2933`   Used in: 5
- Variants: `selected`, `clickable`.
- Anatomy: inner margin (12, 8), radius 10 (`main.rs:2907-2908`). Fill: selected `rgba(29,185,84,32)` (`main.rs:2922`); hovered `#333333` (`main.rs:2924`); else transparent (`main.rs:2926`). No border. Cursor `PointingHand` if clickable and hovered (`main.rs:2929`).
- States: selected, hovered, idle. Focus: NOT FOUND. Disabled: NOT FOUND (callers skip clicks when busy instead).

### Component: summary_card

- File: `main.rs:2827-2844`   Used in: 6 (Overview, repeated per breakpoint)
- Anatomy: `card` + `label_small` + value 28 px strong + caption 13 px `#B3B3B3` + `ghost` action.

### Component: tool_card

- File: `main.rs:2846-2863`   Used in: 9 (three copies of the same three cards)
- Anatomy: `card`, heading 15 px strong, body 13 px `#B3B3B3`, nested `safety`, `ghost` action.

### Component: disk_access_card

- File: `main.rs:2680-2700`   Used in: 3 (Overview breakpoints)
- Anatomy: label 11 px, value 28 px strong `#1DB954` if granted else `#F0B429` (`main.rs:2684-2686`), caption 13 px `#B3B3B3`, optional `ghost("Open Privacy settings")`.

### Component: stat

- File: `main.rs:3008-3013`   Used in: 10
- Anatomy: `card`, `label_small` of uppercased label (`main.rs:3010`), value 28 px strong in the passed color.

### Component: status_pill

- File: `main.rs:3015-3032`   Used in: 1 (sidebar bottom)
- Variants by `status` string:
  - `"paused"` → `#F0B429`, `"Paused"` (`main.rs:3017`)
  - `"canceling"` → `#E8706A`, `"Stopping..."` (`main.rs:3018`)
  - `"cleaning"` → `#F0B429`, `"Moving to Trash..."` (`main.rs:3019`)
  - else if busy → `#1DB954`, `"Working..."` (`main.rs:3020`)
  - else → `#B3B3B3`, `"Ready"` (`main.rs:3021`)
- Anatomy: `glass_pill`. Dot box 8×8, circle radius 4 (`main.rs:3027-3028`). Label 12.5 px `#FFFFFF`. Busy alpha `0.45 + 0.55 * abs(sin(time * 3))` (`main.rs:3023`).

### Component: sidebar_nav_item

- File: `main.rs:1000-1054`   Used in: the loop over `Screen::ALL` (10 items) `main.rs:913`
- Variants: active / inactive, compact / wide.
- Anatomy wide: icon 24×24 (`main.rs:1027`), gap 10, title `heading_font(14)`, hint 11.5 px `#B3B3B3`. Frame radius 6 (`main.rs:1011`). Padding top/bottom 8; left 14 if active else 10; right 10 (`main.rs:1012-1016`). Active fill white alpha 18 (`main.rs:1004`). Active bar 3 × row height, radius 2, `#1DB954`, drawn 4 px left of the row (`main.rs:1041-1045`).
- Compact: icon 26×26 centered (`main.rs:1022`), no title. Hover text is `"{label}\n{hint}"` (`main.rs:1048`).
- States: active (colors + bar). Hover: cursor `PointingHand` (`main.rs:1051`). Focus: NOT FOUND. Disabled: NOT FOUND.

### Component: sidebar_brand

- File: `main.rs:979-998`   Used in: 1
- Wide: mark 32×32 `#1DB954` (`main.rs:989-990`), gap 10, `"Sable"` `heading_font(18)` `#FFFFFF`, `"For your Mac"` 11.5 px `#B3B3B3` (`main.rs:994-995`).
- Compact: mark 34×34, hover text `"Sable — Mac cleaner and tune-up"` (`main.rs:982-984`). The mark is `Sense::hover` only, not click (`main.rs:982`).

### Component: meter / meter_with_width

- File: `main.rs:2982-2996`   `meter` 3 calls, `meter_with_width` 3 calls
- Anatomy: height 6 (`main.rs:2988`). Default width `available.clamp(48, 220)` (`main.rs:2983`). Track radius 3, fill `#2A2A2A` (`main.rs:2989`). Fill width `max(4, width * fraction)` (`main.rs:2993`), color argument. No text.

### Component: gauge

- File: `main.rs:2998-3006`   Not a widget. Returns `#E8706A` if fraction > 0.85, `#F0B429` if > 0.65, else `#1DB954`.

### Component: avatar

- File: `main.rs:2935-2948`   Used in: 2
- Anatomy: 34×34, radius 9, fill accent alpha 34 (`main.rs:2936-2940`). One uppercase letter, `FontId::proportional(16)` (`main.rs:2947`). Not a photo. No image `alt`.

### Component: bubble

- File: `main.rs:2958-2980`   Used in: 2
- Anatomy: diameter `(36 + 70 * share.sqrt()) * scale` (`main.rs:2960`), scale 1 or 0.72. Click rect `(diameter + 8, diameter + 22)` (`main.rs:2961`). Circle fill `#1DB954` if selected else `#333333` (`main.rs:2963`). Stroke 1 px `#1DB954` or `#2A2A2A` (`main.rs:2964`). Label under the circle, 11 px or 10 px (`main.rs:2973`), color `#FFFFFF` if selected else `#B3B3B3`. Name truncated at 12 characters with `…` (`main.rs:2968-2969`).
- States: selected, hovered (cursor only `main.rs:2976`). Focus: NOT FOUND.

### Component: confirm modal

- File: `main.rs:825-878`   Used in: 1 (`main.rs:973`)
- Anatomy: see `03_screens.md`. Buttons are `primary` and `ghost`, not a third button style.
- States: open or absent. Escape closes (`main.rs:867`).

### Component: checkbox (egui)

- File: inline `main.rs:2429`   Used in: Space Lens list only
- Anatomy: `ui.checkbox(&mut on, "")`. Empty label string. No `min_size` in app code. Size is egui’s default `interact_size`, which this app does not set (`polish_style` does not assign `interact_size`).
- States: `add_enabled_ui(!spared && !node.is_directory)` (`main.rs:2428`). Focus/aria: NOT FOUND. The `""` label is the accessible name passed to egui.

### Component: icon_with_label / paint_ui_icon / paint_screen_icon / paint_brand_mark

- File: `icons.rs`
- `icon_with_label`: icon 16×16, label 12.5 px (`icons.rs:17-22`). Stroke width 1.6 (`icons.rs:29`).
- `paint_screen_icon`: stroke 1.75 (`icons.rs:94`). Callers allocate 24, 26, or 40 px boxes.
- `paint_brand_mark`: three bars, width 3.2, gap 5, corner radius 2 (`icons.rs:178-186`).
- `UiIcon::Folder` is never constructed (compiler warning, `icons.rs:13`).

### Component: Custom Utility button / input / article

- File: `styles.css:34-67`, `styles.css:75-79`
- One `button` rule: radius 8 px, padding 10×14, background `#2a6df4`, color `white`, border 0 (`styles.css:53-61`).
- Variant: `#pause`, `#resume`, `#cancel` background `#2c3238` (`styles.css:63-67`). No hover, focus, or disabled rule. `:hover` / `:focus` / `:disabled`: NOT FOUND in `styles.css`.
- `input`: radius 8 px, padding 10×12, border 1 px `#3c424a`, background `#1a1d21` (`styles.css:34-45`).
- `article`: radius 10 px, padding 14 px, background `#1a1d21` (`styles.css:75-78`).

### Inconsistencies across buttons

Three Sable button functions (`primary`, `ghost`, `chip`) plus egui `Button` defaults from `theme.rs`, plus the Electron `button` rule which uses `#2a6df4` and radius 8 px instead of `#1DB954` and radius 500. Sable has no shared token struct; colors are `const`s in `main.rs` and separate literals in `theme.rs`.
