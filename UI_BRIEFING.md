# UI Briefing — Sable (RustUtilityApp)

This document describes the **production desktop UI** for **Sable**, a macOS utility app in `RustUtilityApp/`. A separate minimal **Custom Utility** Electron shell lives in `CustomUtilityApp/frontend/` (engine dev UI); it is documented in §2.1 and §3 appendix.

---

## 1. APP OVERVIEW

- **What it does:** Sable is a Mac cleaner and tune-up desktop app. Users pick a feature from a left sidebar (Overview, Free Up Space, Apps, Tune-Up, File Finder, Threat Scan, Big Downloads, Space Lens, Cloud Storage, System Monitor). Most screens load data on first visit; destructive actions move files to the Trash only after confirmation.
- **Who uses it:** macOS users who want read-only health checks, cache cleanup, large-file review, disk mapping, malware hash comparison, and light maintenance (DNS flush, Spotlight status/reindex).
- **Main flow:** App opens on **Overview** → auto-runs care + junk sizing → user navigates via sidebar → feature screens auto-fetch or user starts scans → results in cards/lists → Trash actions open a confirm modal → success/error **notice banner** at top of content.
- **Platform:** Native **desktop** (macOS), Rust + **eframe 0.29** / **egui** immediate-mode GUI. Not web or mobile.
- **Constraints:**
  - Default window **1180×800** px; minimum **720×520** px (`main.rs`).
  - Responsive breakpoints: **compact** if width `< 880`, **narrow** if `< 1100` (`UiLayout::from_viewport`).
  - Requires desktop session; uses `open` / System Settings URLs for Finder and Privacy panes.
  - **No light mode** in code — dark theme only.
  - **Full Disk Access** affects copy and gating on Space Lens and File Finder; sidebar shows access state.
  - Offline-capable for most features; Threat Scan may download MalwareBazaar hash list when stale (network in engine, not UI).

---

## 2. TECH STACK FOR THE UI

### 2.1 Sable (primary)

| Layer | Choice |
|--------|--------|
| Language | Rust 2021 |
| Framework | **eframe** 0.29 (winit + egui) |
| Styling | No CSS/Tailwind. **egui `Style` + `Frame`**; tokens split between `main.rs` color constants and `theme.rs` glass frames |
| Component library | None — custom widgets as functions in `main.rs` |
| Icons | **Custom vector icons** drawn with `egui::Painter` in `icons.rs` (`UiIcon`, `paint_screen_icon`, `paint_brand_mark`) — stroke ~1.6–1.75 px |
| Fonts | **SableUI-Regular.ttf**, **SableUI-SemiBold.ttf** embedded via `include_bytes!` in `theme.rs`; semibold registered as `FontFamily::Name("semibold")` |
| Animation | egui **spinner**; status pill dot alpha pulses with `sin(time * 3.0)` when `busy`; no tween library |

**UI code locations:**

```text
RustUtilityApp/crates/ui/src/main.rs    — app shell, all screens, widgets (~3183 lines)
RustUtilityApp/crates/ui/src/theme.rs   — fonts, polish_style, glass_* frames
RustUtilityApp/crates/ui/src/icons.rs   — icon painting, heading_font helper
RustUtilityApp/crates/ui/assets/        — SableUI font files (referenced by theme.rs)
RustUtilityApp/crates/ui/Cargo.toml     — eframe, engine, tokio
```

Engine (non-UI): `RustUtilityApp/crates/engine/`.

### 2.2 Custom Utility (secondary / dev shell)

| Layer | Choice |
|--------|--------|
| Platform | Electron renderer |
| Files | `CustomUtilityApp/frontend/src/renderer/index.html`, `styles.css`, `renderer.ts` |
| Styling | Plain CSS in `:root` and class selectors |
| Fonts | `"Segoe UI", sans-serif` |
| Icons | None |

---

## 3. SCREEN INVENTORY

Global shell (all screens): **left `SidePanel` nav** + **scrollable `CentralPanel` content**. No tabs inside content except implicit steps on wizard-like screens.

### 3.0 App shell — `ScanApp::update` in `main.rs`

**Purpose:** Frame for entire app.

**Layout:**
- Left sidebar width: **268 px** (wide), **228 px** (narrow), **76 px** (compact).
- Content horizontal padding: **32 / 20 / 14 px**; vertical **22 / 12 px**.
- Content max width: `(viewport - sidebar - 2×pad_x)` clamped **280–980 px**.
- Central area: `ScrollArea::vertical`, id `main_{screen.label()}`.

**Components (visual order):**

**Sidebar (`SidePanel::left("nav")`):**
1. Brand block — compact: 34×34 brand mark; wide: 32×32 mark + **"Sable"** (18 px semibold) + **"For your Mac"** (11.5 px `SUBTEXT`).
2. Section label **"TOOLS"** or **"MENU"** (compact) — `label_small` 11 px uppercase-style.
3. Ten nav rows (`Screen::ALL`) — icon 24×24 (wide) or 26×26 centered (compact); title 14 px semibold + tagline 11.5 px; active: green 3× bar left, bg `rgba(255,255,255,18)`, icon/title active colors.
4. Bottom: **status pill** (`Ready` / `Working...` / `Paused` / etc.) + disk access (text or 10 px dot).

**Central panel:**
- Screen-specific content (below).
- Overlays: **confirm modal** (when `app.confirm` set).

**States:**
- **Busy:** sidebar status animates; many actions disabled via `enabled` on buttons; spinners on loading cards.
- **Notice:** `banner()` — success (`Done`) or error (`That did not work`).

---

### 3.1 Overview — `care_screen`, `Screen::Care`

**Path:** `RustUtilityApp/crates/ui/src/main.rs` (`care_screen`).

**Purpose:** Read-only Mac health summary; entry point.

**Layout:** Vertical stack: page header card → safety pill → optional notice banner → hero → 3-column (or stacked) summary cards → “Where to go next” rows → ghost **Run the check again**.

**Components:**
1. **page_header** — 40 px screen icon, **"SABLE"** 11 px green, title **"Overview"**, subtitle (care copy).
2. **safety** — Read-only pill: eye icon + **"Read-only — nothing on disk is changed"** (`LIME`).
3. **banner** — if notice present.
4. **hero** (`glass_hero(GREEN)`):
   - If reclaim bytes known: **"YOU CAN FREE UP"**, byte size (30 px compact / 44 px wide) green, explanatory line, **primary "Review and clean up"**.
   - If loading: **loading_line** — spinner + **"Checking your Mac..."**.
5. **Summary grid** (when `care` report loaded):
   - **summary_card "APPS"** — count, **"Browse apps"**.
   - **summary_card "STARTUP ITEMS"** — count, **"See startup items"**.
   - **disk_access_card** — **"DISK ACCESS"**, **Full** / **Limited** 28 px, explanation, optional **Open Privacy settings**.
6. **section "WHERE TO GO NEXT"** — three **row** items (Clutter, Space, Apps) with green **"Open"** on right.
7. **ghost "Run the check again"** — clears loaded Care + Cleanup.

**Empty / loading / error:**
- Hero loading line while care/junk not ready.
- Errors via global **banner**, not inline on Overview.

---

### 3.2 Free Up Space — `cleanup_screen`, `Screen::Cleanup`

**Purpose:** List cache/log **junk roots** with sizes; move groups to Trash.

**Layout:** Header → safety (Trash) → banner → hero total → section **"WHAT TAKES THE SPACE"** → one **card** per junk root.

**Components per junk card:**
- Title 15.5 px strong, `describe_junk` caption 13 px muted.
- Right: **Move to Trash**, **Show**, byte label 16 px `LIME` — or **"Not on this Mac"**.
- **meter** bar under row if exists.

**Hero:** **"TOTAL YOU CAN CLEAN"**, 38 px bytes, **ghost "Measure again"**.

**States:**
- **loading_card:** busy → **"Measuring caches and logs..."**; idle empty → **"Nothing to show yet."** (via `loading_card` idle branch title **"Nothing here"**).
- Confirm modal on trash (dynamic heading/body).

---

### 3.3 Apps — `apps_screen`, `Screen::Apps`

**Purpose:** Installed apps list + Library leftovers for selected app.

**Layout:** Header → Read-only safety → banner → search row → scrollable app list card → detail section.

**Components:**
1. **TextEdit** search, `desired_width(320)`, hint **"Search by app name"**, margin `(10,8)`; label `{n} apps`.
2. **ScrollArea** max height **290** — **row** per app: **avatar** 34 px, name, bundle id 12 px.
3. Detail: section **"FILES {NAME} KEEPS IN YOUR LIBRARY"** or **empty "Pick an app above"**.
4. Leftovers: **loading_card** / **empty** / list with **Show** per hit.
5. **ghost "Show the app in Finder"**.

**States:**
- Loading apps: **"Looking for installed apps..."** / idle **"No apps found in /Applications or ~/Applications."**
- Search no match: **"No app matches your search."**
- Leftovers loading: **"Searching your Library..."**

---

### 3.4 Tune-Up — `performance_screen`, `Screen::Performance`

**Purpose:** Maintenance tools + login startup items list.

**Note:** No page-level **safety()** badge; each tool card embeds safety.

**Layout:** Header → banner → **"FIXES AND MAINTENANCE"** (1–3 **tool_card** columns) → **"APPS THAT START AUTOMATICALLY"**.

**tool_card** each: title 15 px, body 13 px, embedded **safety** pill, **ghost** action (**Flush DNS**, **Check status**, **Rebuild index**).

**Startup list:** intro label; **card** of **row** items — avatar (lime/sky), label, account scope, **Show**.

**States:**
- Agents loading: **"Reading startup items..."** / **"No startup items found for your account."**
- Rebuild → **Confirm** modal (**"Rebuild"** / **"Cancel"**).

---

### 3.5 File Finder — `protection_screen`, `Screen::Protection`

**Purpose:** Extension-based folder scan (not antivirus); read-only listing.

**Layout:** Header → Read-only safety → banner → optional FDA warning **card** → wizard **card** → progress/stats → results.

**Wizard card:**
1. **step_label "1"** **"Where should it look?"** — chips Downloads/Documents/Desktop/Home folder → **field** (monospace path).
2. **step_label "2"** **"Which kinds of files?"** — chips Installers/Archives/Disk images/Videos/Documents → **Extensions:** monospace **TextEdit** hint **"dmg, pkg, zip"**.
3. Buttons: **primary "Start searching"**, **ghost Pause/Resume**, **ghost Stop**.

**During scan card:** spinner + progress text; last path monospace 12 px.

**After scan:** **stat** cards Files checked / Matches / Could not read (grid by breakpoint).

**Results:** section **"{N} MATCHING FILES"**; rows with name, folder, **Show**, size, **badge** extension (`.ext` in `SKY`).

**States:**
- **empty "Nothing searched yet"** / **"No matching files"**
- List capped at **300** + **"and {n} more"**

---

### 3.6 Threat Scan — `malware_screen`, `Screen::Malware`

**Purpose:** Hash scan vs MalwareBazaar; Trash on confirm.

**Layout:** Header → Trash safety → banner → disclaimer **card** → scan **card** → results.

**Disclaimer card:** **"What this scan does not do"** + long muted copy + **Open Privacy settings**.

**Scan card:** step 1 chips (Downloads, Desktop, Documents, Launch Agents) + **field**; note on signature download; **primary "Scan"**.

**Busy:** spinner + **"Updating signatures if needed, then hashing files."**

**Results:** **stat "Files hashed"**; signatures note card; empty **"No known malware hashes"** or **section "{N} MATCHES"** with **Trash**, **Ignore**, **Show**, **badge "hash match"** (`ROSE`).

**States:**
- **empty "Nothing scanned yet"**
- Errors via **banner** (invalid folder, scan stopped, etc.)

---

### 3.7 Big Downloads — `clutter_screen`, `Screen::Clutter`

**Purpose:** Files ≥ **50 MB** in Downloads; Trash one-by-one.

**Layout:** Header → Trash safety → banner → hero or empty → list **card**.

**Hero:** **"LARGE FILES FOUND"**, 38 px total, file count; **ghost "Look again"**.

**Rows:** name, path, **meter_with_width**, size, **Show**, **Move to Trash**.

**States:**
- Loading: **"Looking through Downloads..."**
- Empty success: **"Your Downloads folder is tidy"** + **Look again**

---

### 3.8 Space Lens — `space_screen`, `Screen::Space`

**Purpose:** Folder size map with list, bubble chart, selection registry.

**Layout:** Header (no safety badge) → banner → optional **space_lens_permission_card** → setup **card** → results or empty.

**Setup card:**
- **step_label "1" "Select storage"** — chips Macintosh HD, Home, Documents, Downloads, Desktop; **collapsing "Custom folder"** + field.
- **step_label "2" "Scan"** — target path monospace; **primary_icon Scan** / **ghost Scan again**; spinner text when busy.

**Results:**
- **ghost "Up one level"** + current path; total size 22 px green.
- **space_lens_results_grid:**
  - List pane width ~46% of pair (clamp 200–420), bubble pane remainder; side **selection registry** 248 px if width > 780 and not compact.
  - Pane height **480** (380 compact).

**List pane:** **"LIST — LARGEST FIRST"**; rows with checkbox (files only, not spared dirs), name, type · modified, meter (amber if share > 0.4), size; double-click dir to drill; secondary click reveal Finder.

**Bubble pane:** **"LENS — PROPORTIONAL MAP"**; wrapped **bubble** circles; **Other** bucket; double-click dir drills.

**Selection registry:** **"SELECTION"**, bytes 26 px green, item list scroll max **220**, **primary "Remove"**, **ghost "Clear selection"**, protected-path copy.

**Hover:** floating **space_lens_hover_card** (tooltip Area) follows pointer +14,+14.

**States:**
- **loading_card "Building your storage map…"**
- **empty "No map yet"** + **primary Scan**
- **empty "Nothing to show"** for empty folder

---

### 3.9 Cloud Storage — `cloud_screen`, `Screen::Cloud`

**Purpose:** Local sizes for iCloud/cloud folders.

**Layout:** Header → Read-only safety → banner → one **card** per cloud folder.

**Card:** label 16 px, path or **"This service is not set up on this Mac."**, **Show**, bytes 18 px `LIME`, **meter** (`SKY`).

**States:** **loading_card "Measuring cloud folders..."** / **"No cloud folders found."**; **ghost "Measure again"**.

---

### 3.10 System Monitor — `assistant_screen`, `Screen::Assistant`

**Purpose:** One-shot CPU/memory sample.

**Layout:** Header → Read-only safety → banner → 1 or 2 **stat_card** columns → verdict **card** → **ghost "Refresh"**.

**stat_card:** label uppercase small, value **36 px** `gauge(frac)`, **meter**, caption 13 px.

**Verdict card:** plain-language line colored `AMBER` or `LIME`.

**States:** **loading_card "Taking a reading..."** / **"No reading yet."**

---

### 3.11 Confirm modal — `confirm_modal`

**Purpose:** Confirm Trash, threat trash, Spotlight reindex.

**Layout:** Full-screen scrim `rgba(0,0,0,170)` → centered **Window** (no title bar), `glass_card`, width **420** (320 compact).

**Components:** Alert icon + heading; body muted; **primary** (action label e.g. **"Move to Trash"**, **"Rebuild"**) + **ghost "Cancel"**; **Escape** cancels.

---

### 3.12 Appendix: Custom Utility single page

**Path:** `CustomUtilityApp/frontend/src/renderer/index.html`

**Purpose:** Minimal scan control panel for custom engine; **does not delete files** (per header copy).

**Layout:** Centered **main** max-width **720 px**, padding **32×20×48**.

**Components:** **h1** Custom Utility; subtitle; **label** Root folder + **input#root**; **controls** — Start Scan, Pause, Resume, Cancel; **metrics** 3-column grid (Files scanned, Files / second, Threats); **#status**; **#list** scroll max-height **220**.

**States (renderer.ts):** status strings e.g. **"Engine online. Choose a folder and start."**, **"Scanning."**, **"Paused."**, **"Scan finished."**, threat lines prepended to list.

---

## 4. DESIGN TOKENS AS THEY EXIST TODAY

### 4.1 Colors (Sable)

**Primary palette (`main.rs` constants — used heavily in widgets):**

| Token | RGB / hex | Usage |
|--------|-----------|--------|
| `BG` | `18,18,18` → **#121212** | Central panel fill |
| `SIDEBAR` | `0,0,0` → **#000000** | Defined; sidebar uses `glass_sidebar` instead |
| `ELEVATED` | `24,24,24` → **#181818** | Ghost/chip inactive fills |
| `CARD` | `40,40,40` → **#282828** | Disabled primary, bubble default fill |
| `CARD_HOVER` | `51,51,51` → **#333333** | Row hover, bubble idle |
| `LINE` | `42,42,42` → **#2A2A2A** | Borders, meter track |
| `TEXT` | **#FFFFFF** | Primary text |
| `SUBTEXT` / `MUTED` | `179,179,179` → **#B3B3B3** | Secondary copy |
| `GREEN` / `LIME` | `29,185,84` → **#1DB954** | Accent, primary buttons, active nav |
| `GREEN_ON` / `LIME_INK` | **#FFFFFF** | Text on green buttons |
| `AMBER` | `240,180,41` → **#F0B429** | Warnings, Trash safety, limited access |
| `ROSE` | `232,112,106` → **#E8706A** | Errors, admin safety, hash badge |
| `SKY` | `120,190,240` → **#78BEF0** | Extension badges, cloud meters, system startup avatars |

**Derived / alpha (examples):**
- Nav active bg: `rgba(255,255,255,18)`
- Row selected: `rgba(29,185,84,32)`
- Avatar fill: `rgba(accent,34)`
- Badge fill/stroke: accent @ 28 / 90 alpha
- Confirm scrim: black alpha **170**
- Status dot pulse: accent @ variable alpha

**`theme.rs` egui `Visuals` (applied via `polish_style`):**

| Token | Value | Usage |
|--------|--------|--------|
| `window_fill` / `panel_fill` | **#0E0E10** (`14,14,16`) | egui panels |
| `extreme_bg_color` | **#0A0A0C** | Deep bg |
| `faint_bg_color` / widget inactive fill | **`CARD` #282828** | Widget defaults |
| `code_bg_color` | `rgba(0,0,0,120)` | Code style bg |
| Hover widget fill | `rgba(255,255,255,16)` | egui hover |
| Active widget fill | **`GREEN`** | egui active |
| Selection | green @ 48 alpha | Text selection |
| `glass_stroke_color` | `rgba(255,255,255,22)` | Card/sidebar strokes |

**Glass frames (`theme.rs`):**
- **glass_card:** fill `rgba(255,255,255,10)`, stroke glass, radius **16**, margin **16**, shadow offset `(0,6)` blur **14**, color black α55
- **glass_hero:** accent fill α16, stroke accent α55, radius **18**, margin **22**, shadow blur **18** α70
- **glass_sidebar:** fill `rgba(8,8,10,235)`, stroke `rgba(255,255,255,14)`
- **glass_pill:** fill `rgba(255,255,255,8)`, radius **20**, margin **12×7**
- **glass_notice:** accent fill α18, stroke α90, radius **14**, margin **16×12**

**Light/dark:** Dark only. Comment in code: *"Spotify-inspired dark UI (#121212 base, #1DB954 accent)."*

### 4.2 Custom Utility CSS colors

| Variable / rule | Hex | Usage |
|-----------------|-----|--------|
| `:root background` | **#101214** | Page |
| `:root color` | **#E8EAED** | Text |
| Muted copy | **#B0B6BE** | header p, #status, article span |
| Input bg/border | **#1A1D21** / **#3C424A** | Fields |
| Primary button | **#2A6DF4** | Start Scan |
| Secondary buttons | **#2C3238** | pause/resume/cancel |
| article card | **#1A1D21** | Metrics |
| list border | **#2C3238** | li separators |

`color-scheme: light dark` declared but styles are dark-fixed.

### 4.3 Typography (Sable)

| Style | Size | Family | Where |
|--------|------|--------|--------|
| `TextStyle::Heading` | **28** | semibold | egui default heading |
| Page title | **22 / 28 / 34** | semibold (`layout.title_size()`) | page_header |
| Subtitle | **13 / 14.5** | proportional | page_header |
| `TextStyle::Body` | **14.5** | SableUI regular | body |
| `TextStyle::Button` | **14** | semibold | buttons |
| `TextStyle::Small` | **11.5** | regular | small labels |
| `TextStyle::Monospace` | **12.5** | monospace | paths, fields |
| `label_small` | **11** strong | SUBTEXT | section labels |
| Brand | **18** semibold | sidebar title |
| Hero totals | **38–44** | strong | cleanup/clutter |
| Stat values | **28–36** | strong | stats, monitor |

Line height: egui defaults (not explicitly set).

### 4.4 Spacing (Sable)

- Global item spacing: **`Vec2(10, 10)`** (`theme.rs`)
- Button padding: **`(16, 9)`**
- Scroll bar width: **6**
- Recurring: **4, 6, 8, 10, 12, 14, 18, 22** px vertical gaps in screens
- Row inner margin: **12×8**
- Nav item margin: **8** vertical, **10–14** horizontal
- Content pad_x/y: see §3.0

### 4.5 Radius, shadows, borders

- Pills/buttons/chips: **`Rounding::same(500.0)`** (full pill)
- Cards: **16**; hero **18**; rows **10**; nav **6**; badges **20**; meters **3**; avatar **9**
- Strokes: commonly **1.0** px; ghost/fields use **`LINE`**
- Blur: shadow on glass_card/hero only (epaint Shadow)

### 4.6 Icons

- UI icons: **16×16** box, stroke **1.6**
- Screen nav icons: **24–26** px box, stroke **1.75**
- Page header screen icon: **40×40**
- Brand mark: three bars width **3.2**, gap **5**, heights **55%/95%/45%** of rect
- Empty state info icon: **28×28**

### 4.7 Motion

- **Spinner:** egui default on loading states
- **status_pill:** dot alpha `(0.45 + 0.55 * |sin(time*3)|)` when busy
- **Repaint:** `request_repaint_after(32ms)` while busy
- No page transitions; instant screen swap on nav click

---

## 5. COMPONENT INVENTORY

All in `main.rs` unless noted. States use egui `Button` / `add_enabled`.

| Component | Variants / sizing | Styling |
|-----------|-------------------|---------|
| **primary** | enabled/disabled; min **120×40**, pill radius 500 | enabled: fill `GREEN`, text `GREEN_ON` strong; disabled: fill `CARD`, stroke `LINE`, text `SUBTEXT` |
| **ghost** | enabled gate; min **70×32** | fill `ELEVATED`, stroke `LINE`, text `TEXT` |
| **primary_icon** | icon 15×15 + primary | Same as primary |
| **chip** | active/inactive; min height **32** | active: fill+stroke `GREEN`, text white; inactive: `ELEVATED` + `LINE` |
| **field** | monospace TextEdit, margin **10×8**, full width | Uses egui widget styling from theme |
| **row** | selected, clickable | radius 10; selected `rgba(29,185,84,32)`; hover `CARD_HOVER`; pointer cursor if clickable |
| **card** | — | `theme::glass_card()` + 4 px bottom space |
| **hero** | — | `theme::glass_hero(GREEN)` |
| **page_header** | compact title sizes | `glass_card` + icon + SABLE label |
| **section** | — | 4 px top space + `label_small` |
| **label_small** | — | 11 px SUBTEXT strong |
| **step_label** | — | `badge` number + 15 px semibold title |
| **badge** | any accent color | fill α28, stroke α90, radius 20, text 12 px |
| **safety** | ReadOnly / Trash / Admin | `glass_pill` + icon; colors LIME / AMBER / ROSE |
| **banner** | ok / error | `glass_notice`; headings **Done** / **That did not work**; **Dismiss** ghost |
| **empty** | — | centered Info icon 28 px, heading 16 px semibold, body MUTED |
| **loading_line** | — | spinner + MUTED text |
| **loading_card** | busy vs idle | busy: spinner in card; idle: `empty("Nothing here", idle)` |
| **summary_card** | — | card + 28 px value + ghost action |
| **tool_card** | — | card + embedded safety + ghost |
| **disk_access_card** | granted bool | 28 px Full/Limited green/amber |
| **stat** | — | uppercase label + 28 px count |
| **status_pill** | status string + busy | `glass_pill`; dot 4 px radius; labels Ready/Working…/Paused/Stopping…/Moving to Trash… |
| **meter** / **meter_with_width** | height **6** | track `LINE`, fill accent; width clamp 48–220 or explicit |
| **gauge** | fraction | >0.85 ROSE, >0.65 AMBER, else LIME |
| **avatar** | 34×34 | fill accent α34, letter 16 px proportional |
| **bubble** | compact scale **0.72** | diameter `(36 + 70*sqrt(share))*scale`; fill GREEN or CARD_HOVER |
| **sidebar_nav_item** | compact/wide/active | see §3.0 |
| **sidebar_brand** | compact/wide | paint_brand_mark GREEN |
| **icon_with_label** | `icons.rs` | 16 px icon, 12.5 px label |
| **confirm_modal** | — | scrim + glass_card window |

**Inconsistencies (factual):**
- **Two background systems:** `BG` **#121212** on central panel vs egui `window_fill` **#0E0E10** and `glass_sidebar` **rgba(8,8,10,235)**.
- **`SIDEBAR` #000000** constant unused for actual sidebar frame.
- **Alias colors:** `IVORY`=`TEXT`, `MUTED`=`SUBTEXT`, `LIME`=`GREEN` — same values, mixed names in code.
- **Theme hover** uses white α16; **row hover** uses `CARD_HOVER` **#333333** — different hover languages.
- **Performance screen** omits top-level safety badge unlike other destructive-capable areas (only per-tool pills).
- **Space Lens** has no Trash safety pill at page level (copy in selection registry only).
- **Hard-coded greens** in row selected `29,185,84` duplicate `GREEN` constant.
- **Custom Utility** palette unrelated to Sable Spotify tokens.

---

## 6. INTERACTION AND CONTENT

### 6.1 Navigation

- **Model:** Left sidebar only; `ScanApp.screen` enum — no URLs, no back stack except Space Lens drill-up.
- **Switching screens:** Click nav item → `go(screen)` clears notice.
- **Auto-load:** On idle, loads current screen + Care + Cleanup if not in `loaded` set and `screen.auto_loads()` (false for Protection, Malware, Space).
- **Cross-links:** Overview summary cards and “Where to go next” rows navigate to Apps, Performance, Clutter, Space.

### 6.2 Key interactions

| Action | Behavior |
|--------|----------|
| Primary / ghost buttons | Engine async via tokio; `busy` locks most controls |
| Trash | Sets `confirm` modal → `perform(ConfirmKind::…)` |
| Show / Finder | `open -R` via `reveal()` |
| Privacy | `open` on `x-apple.systempreferences:…` URLs |
| File Finder scan | Pause/resume/cancel via `ExecutionControl` |
| Space Lens | Scan/drill changes `space_root` and rescans; checkboxes build selection |
| Threat Ignore | Local ignore list; no modal |
| Notice Dismiss | Clears `app.notice` |
| Escape | Closes confirm modal |

**Progress:** Sidebar **status_pill**; screen-local spinners and progress cards; File Finder shows last path.

### 6.3 Copy tone

Plain, reassuring, Mac-specific; explains Trash undo and read-only scans. Examples:

1. **"For your Mac"** (sidebar tagline)
2. **"Read-only — nothing on disk is changed"**
3. **"Moves to Trash — you can undo this"**
4. **"A quick health check of your Mac. This page only looks. It never changes or deletes anything."**
5. **"Review and clean up"**
6. **"Full Disk Access is required to scan this drive. Open System Settings → Privacy & Security → Full Disk Access, enable Sable, then press Scan again."**
7. **"Done"** / **"That did not work"** (notice headings)
8. **"Moved \"{name}\" to the Trash. Open the Trash to put it back."**
9. **"Circle size = disk space. Items under 2% roll into “Other”."**
10. **"Everything looks fine."** (System Monitor verdict)

---

## 7. KNOWN UI PROBLEMS (FACTUAL ONLY)

- **Dual theme layers:** `theme::polish_style` sets egui visuals while most widgets hard-code `main.rs` colors — changing one file does not restyle everything.
- **No light mode** or user theme toggle.
- **Responsive rules are width-only** (no height handling); Space Lens panes use fixed heights **380/480** — may clip on short windows (min height 520).
- **Compact sidebar** hides nav text — relies on hover tooltips for labels/accessibility.
- **No explicit focus ring styling** beyond egui defaults; keyboard nav not customized per control.
- **Checkbox labels empty** in Space Lens list (`checkbox` with `""`) — screen-reader label gap unless egui accesskit maps widget id.
- **Search field** on Apps has no visible label (placeholder only).
- **Confirm modal** uses internal window id `"confirm"` — not user-facing title.
- **Font assets** referenced at `crates/ui/assets/SableUI-*.ttf`; if missing at runtime, build may fail (project expects files present when built).
- **Custom Utility** uses Segoe UI (Windows-named stack) on macOS — may fall back to system sans.
- **Duplicated layout code:** Performance `tool_card` grid repeated in three match arms; Reindex confirm duplicated.
- **File Finder** screen name **Protection** in enum vs UI label **File Finder** — naming mismatch in code only.

---

## 8. VISUAL SNAPSHOT

Screenshots were **not** captured: Sable is a native egui window (`cargo run -p ui`); this audit environment did not run an interactive GUI capture session. Folder **`/Users/hema/Desktop/Experiments/crackmed/ui-audit/`** was created empty for future screenshots.

### 8.1 Sable — visual description by screen

**Global:** Full-window dark gray (**#121212**) content; left strip slightly darker translucent panel with faint white border; ten stacked nav rows; current row has vertical **#1DB954** bar and brighter text; bottom-left pill **Ready** with gray dot (or pulsing green dot when working).

**Overview:** Top wide rounded glass card with green outline icon, **SABLE** in small green caps, large **Overview**, paragraph subtitle; green-outlined pill with eye icon; large green-tinted hero card showing reclaim size in bright green; three equal cards in a row for apps count, startup count, disk access; below, three tappable rows with green **Open** chevron text.

**Free Up Space:** Orange-tinted trash safety pill; green hero with total bytes; stacked dark glass cards each with horizontal lime progress sliver and **Move to Trash** gray buttons on the right.

**Apps:** Search bar; scroll list of rows with colored initial squares; lower section either empty state with info icon or file list.

**Tune-Up:** Three maintenance cards in a grid; list of startup items with **Show** buttons.

**File Finder / Threat Scan:** Step numbers in green pills; row of pill chips for folders/types; green **Start searching** / **Scan**; results as dense lists with monospace paths.

**Space Lens:** Two-pane (or three with selection column): left checklist list with green bars; right bubble chart of circles sized by data; optional right column **SELECTION** with green byte total.

**System Monitor:** Two large percentage numbers colored green/yellow/red by load; horizontal meters; short verdict sentence in colored bold.

**Confirm modal:** Dimmed entire window; centered card with warning icon, bold question, **Move to Trash** green button and **Cancel** gray button.

### 8.2 Custom Utility

Single centered column on **#101214** background: title, paragraph, one text field, blue **Start Scan** and three gray control buttons, three dark metric tiles in a row, status line, scrolling monospace path list.

---

## 9. CHANGE CONSTRAINTS

### 9.1 Must not change (for redesign handoff)

- **Screen enum set** and engine operations wired in `ScanApp` (care, junk, scan, threat, space, trash flows).
- **Confirmation before Trash** and `ConfirmKind` actions.
- **Async message protocol** `UiMsg` and `drain` handling.
- **Auto-load rules** per screen unless product intentionally changes behavior.
- **Engine crate API** in `RustUtilityApp/crates/engine` — UI should remain a shell.
- **macOS integrations:** `reveal`, Privacy URLs, Trash via Finder/engine helpers.

### 9.2 Safe to restyle freely

- All colors, radii, shadows in `theme.rs` and `main.rs` constants.
- `glass_*` frame definitions, typography sizes, spacing in `polish_style`.
- Icon drawing in `icons.rs` (geometry/stroke) — keep semantic icon mapping for nav.
- Button/card function internals (`primary`, `ghost`, `row`, `bubble`, etc.).
- Layout breakpoints in `UiLayout` (sidebar widths, padding, grid_cols).
- Copy strings in screen functions (tone/branding) without altering engine calls.
- Custom Utility **styles.css** entirely separate from Sable.

### 9.3 How to approach a token-driven redesign

1. **Centralize colors:** Today split between `main.rs` (`BG`, `GREEN`, …) and `theme.rs` (`polish_style`, glass fills). A redesign should merge into one module and pass into widget fns — otherwise egui defaults and hard-coded `RichText::color()` will drift.
2. **Widget layer:** Most UI goes through helpers at bottom of `main.rs` (~2721–3094). Restyling **primary/ghost/chip/card/row/page_header** updates most screens at once.
3. **Screen-specific layout** (Space Lens grid sizes, scroll max-heights) stays in screen functions unless extracted.
4. **Fonts:** Swap TTF bytes and `install_fonts` families only — no other files required for font change.
5. **Custom Utility:** Independent CSS variables in `:root` — no impact on Sable.

---

*Generated from static analysis of `RustUtilityApp/crates/ui` and `CustomUtilityApp/frontend`. Window title: **Sable** (`eframe::run_native("Sable", …)`).*
