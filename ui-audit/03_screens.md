# 03 — Screen-by-screen spec

Shared shell for every Sable screen (`ScanApp::update`, `main.rs:882-974`).

Window title string `"Sable"` (`main.rs:220`). Default inner size `[1180.0, 800.0]` (`main.rs:215`). Minimum `[720.0, 520.0]` (`main.rs:216`).

At width `>= 1100` (`main.rs:126-129`):

```
+----------------------------------------------+
| macOS title bar "Sable" (not drawn by egui)  |
+----------------+-----------------------------+
| SidePanel      | CentralPanel fill #121212  |
| exact_width    | inner margin (32, 22)      |
| 268            | ScrollArea vertical        |
| glass_sidebar  | max width content_max      |
| margin x16 y18 |   = min(980, w-sidebar-64) |
+----------------+-----------------------------+
```

At the first capture, window bounds were 1180×828 points (inner 800 + title chrome). PNG `03_overview_1180x800.png` is 2704×1696 pixels.

Later captures: window bounds 1352×848 points. Those PNGs are also 2704×1696 pixels. `content_max` at 1352 = 980 (`main.rs:130`).

Compact (`w < 880`): sidebar **76**, pad **14×12**, nav text hidden (`main.rs:1020-1024`). Not captured. 390 px width: impossible; min width is 720 (`main.rs:216`).

There is no URL. Trigger for each screen is `sidebar_nav_item` click → `self.go(item)` (`main.rs:913-917`).

---

### Screen: Overview

- File(s): `care_screen` `main.rs:1058-1227`; shell `main.rs:902-970`
- Route/trigger: initial `screen: Screen::Care` (`main.rs:285`). Sidebar label `"Overview"` (`main.rs:75`).
- Purpose: Shows reclaimable cache bytes, app count, startup count, and disk-access state, and links to other screens.
- Screenshot files: `03_overview_1180x800.png` (2704×1696 px). Captured with data: "4.1 GB", "62" apps, "10" startup items, "Full" disk access, status "Ready".
- Wireframe (width ≥ 1100, sidebar 268, content pad 32):
  ```
  +--268--+---------------- content_max (<=980) --+
  | brand | page_header glass radius 16, icon 40 |
  | 32px  | safety pill                            |
  | TOOLS | hero glass radius 18                   |
  | nav   | 3 summary cards (grid_cols 3)          |
  | x10   | section WHERE TO GO NEXT + 3 rows      |
  | pill  | ghost "Run the check again"            |
  +-------+----------------------------------------+
  ```
- Components in reading order:
  1. `page_header` (`main.rs:2721`): icon 40×40 green `#1DB954`, label `"SABLE"` 11 px (`main.rs:2728`), title `"Overview"` via `title_size()` 34 px at this width (`main.rs:157`), subtitle at `main.rs:1063` size 14.5 (`main.rs:161`).
  2. `safety(ReadOnly)` (`main.rs:1065`): text `main.rs:2711`, color `LIME` `#1DB954`.
  3. `banner` only if `app.notice` is `Some` (`main.rs:2773`). Absent in the PNG.
  4. `hero` (`main.rs:1079`): label `"YOU CAN FREE UP"` (`main.rs:1096`), bytes at 44 px (`main.rs:1081`) color `GREEN`, sentence `main.rs:1099`, `primary("Review and clean up")` (`main.rs:1104`) → `goto = Screen::Cleanup`.
  5. Three columns (`main.rs:1164`): `summary_card` APPS (`main.rs:1165`), STARTUP ITEMS (`main.rs:1174`), `disk_access_card` (`main.rs:1183`). Value size 28 px (`main.rs:2838`). Disk access value 28 px (`main.rs:2684`).
  6. Section `"WHERE TO GO NEXT"` (`main.rs:1191`). Rows height from `row` padding 12×8 (`main.rs:2907`) plus two text lines. Green `"Open"` (`main.rs:1207`).
  7. `ghost("Run the check again")` (`main.rs:1217`).
- Typography on this screen:
  - Eyebrow `"SABLE"`: semibold 11 px, `#1DB954` (`main.rs:2728`)
  - Title: semibold 34 px, `#FFFFFF` (`main.rs:157`, `main.rs:2730`)
  - Subtitle: proportional 14.5 px, `#B3B3B3` (`main.rs:161`, `main.rs:2736`)
  - Section labels: 11 px strong `#B3B3B3` (`main.rs:2752`)
  - Hero number: 44 px strong `#1DB954` (`main.rs:1081`, `main.rs:1097`)
  - Card values: 28 px strong, default text `#FFFFFF` (`main.rs:2838`; disk access uses `#1DB954` or `#F0B429` `main.rs:2686`)
  - Buttons: `TextStyle::Button` 14 px semibold (`theme.rs:61`)
- Spacing between major blocks: `page_header` adds 12 (`main.rs:2743`); hero then `add_space(14)` (`main.rs:1113`); after grid `add_space(8)` (`main.rs:1187`) then `add_space(6)` (`main.rs:1190`); before rerun `add_space(10)` (`main.rs:1216`). Card frames add another 4 (`main.rs:2824`).
- Interactive elements:
  - Nav items: `go` `main.rs:915-916`
  - "Review and clean up": `main.rs:1104-1106` sets `Screen::Cleanup`
  - "Browse apps" / "See startup items": `summary_card` ghost `main.rs:2840`, targets `main.rs:1171` and `main.rs:1180`
  - "Open Privacy settings" only when access is not granted (`main.rs:2697-2698`)
  - Next rows: `main.rs:1211-1212`
  - "Run the check again": clears loaded Care and Cleanup (`main.rs:1221-1223`)
- States present:
  - loading: present — `loading_line("Checking your Mac...")` `main.rs:1111` when reclaim is `None`. Not in the PNG (data already shown).
  - empty: absent on this screen (no `empty()` call in `care_screen`)
  - error: partial — only via shared `banner` `main.rs:1066`
  - success: partial — filled hero and counts in the PNG; no separate success widget
- Responsive behavior: `< 880` stacks hero text and uses 30 px number (`main.rs:1081-1092`), `grid_cols` returns 1 (`main.rs:142`). `< 1100` uses 2 columns then a full-width disk card (`main.rs:1140-1161`). Title 22 / 28 / 34 (`main.rs:151-158`).

### Screen: Free Up Space

- File(s): `cleanup_screen` `main.rs:1230-1307`
- Route/trigger: sidebar `"Free Up Space"` (`main.rs:76`) or Overview primary (`main.rs:1104`)
- Purpose: Lists cache/log groups and asks before moving one group to the Trash.
- Screenshot files: `03_cleanup_1180.png` (2704×1696 px, window 1352×848 at capture). Shows "4.1 GB" total, User caches 3.8 GB, User logs 282.6 MB, Diagnostic reports 124.1 KB, Spotify cache "Not on this Mac".
- Wireframe:
  ```
  +--268--+----------------------------------+
  | nav   | header card                       |
  |       | trash safety pill                 |
  |       | hero: TOTAL YOU CAN CLEAN 38px    |
  |       | section WHAT TAKES THE SPACE      |
  |       | card per junk root (meter h 6)    |
  +-------+-----------------------------------+
  ```
- Components: `page_header` subtitle `main.rs:1235`; `safety(Trash)` `main.rs:1237` text `main.rs:2712` color `AMBER`; hero 38 px `main.rs:1252`; `ghost("Measure again")` `main.rs:1255`; per root title 15.5 px `main.rs:1270`, caption 13 px `main.rs:1271`, bytes 16 px `LIME` `main.rs:1289`, buttons "Move to Trash" and "Show" `main.rs:1275-1287`, or `"Not on this Mac"` `main.rs:1291`; meter `main.rs:1297`.
- Typography: title 34 px `#FFFFFF`; section 11 px `#B3B3B3`; hero number 38 px `#1DB954`; row title 15.5 px; caption 13 px `#B3B3B3`; bytes 16 px `#1DB954`.
- Spacing: section after hero `add_space(14)` `main.rs:1261`; meter after `add_space(4)` `main.rs:1296`.
- Interactive elements: "Measure again" removes Cleanup from `loaded` `main.rs:1256`. "Move to Trash" sets `Confirm` `main.rs:1276-1284` (modal not in the PNG). "Show" calls `reveal` `main.rs:1301-1302` (`open -R`, `main.rs:3121`).
- States: loading present `loading_card` `"Measuring caches and logs..."` `main.rs:1242`. Empty present: idle string `"Nothing to show yet."` which `loading_card` shows under heading `"Nothing here"` `main.rs:2899-2900`. Error partial via `banner` `main.rs:1238`. Success partial: notice text is built in `trash` `main.rs:425`, not a dedicated view.
- Responsive behavior: hero does not branch on `compact` (38 px stays). Grid is a single column of cards at every width. Sidebar width still follows `UiLayout`.

### Screen: Apps

- File(s): `apps_screen` `main.rs:1309-1418`
- Route/trigger: sidebar `"Apps"` (`main.rs:77`)
- Purpose: Lists installed apps and, after a click, files that app keeps under Library.
- Screenshot files: `03_apps_1180.png`. Nav indicator y-band 512–600 px in that PNG matches the third tool (Apps). Filled list plus empty detail are implemented at `main.rs:1338` and `main.rs:1378`.
- Wireframe:
  ```
  +--268--+----------------------------------+
  |       | header + read-only pill           |
  |       | TextEdit w 320 + "{n} apps"       |
  |       | card + ScrollArea max_h 290       |
  |       | empty OR leftover list            |
  +-------+-----------------------------------+
  ```
- Components: search hint `"Search by app name"` margin (10, 8) `main.rs:1328-1330`; count label `main.rs:1332`; rows with `avatar` 34 px radius 9 `main.rs:2936-2939`, name strong, bundle id 12 px `main.rs:1356`; empty `"Pick an app above"` `main.rs:1378`; leftovers section `main.rs:1382`; `"Show the app in Finder"` `main.rs:1414`.
- Typography: title 34 / subtitle 14.5; row name default body 14.5 strong; bundle id 12 px `#B3B3B3`; empty heading 16 px semibold `main.rs:2872`.
- Spacing: `add_space(6)` after search `main.rs:1334`; `add_space(12)` before detail `main.rs:1376`; `add_space(6)` before Finder button `main.rs:1413`.
- Interactive elements: row click `main.rs:1360-1362` calls `survey_selected` `main.rs:1373`. "Show" `main.rs:1402`. Finder `main.rs:1414-1415`.
- States: loading present `"Looking for installed apps..."` `main.rs:1321`. Empty present: no apps string `main.rs:1321`; no search hit `main.rs:1365`; no leftovers `main.rs:1386`. Error: partial (`banner` only). Success: partial (list renders when `apps` is non-empty).
- Responsive behavior: search `desired_width(320)` does not shrink with `compact` (`main.rs:1329`). List max height stays 290 (`main.rs:1339`).

### Screen: Tune-Up

- File(s): `performance_screen` `main.rs:1421-1594`
- Route/trigger: sidebar `"Tune-Up"` (`main.rs:78`)
- Purpose: Runs DNS flush, Spotlight status, and Spotlight reindex, and lists login agents.
- Screenshot files: `03_tuneup_1180.png`. Nav bar y 628–716. Screenshot description: three tool cards and a startup list with "Show" buttons.
- Wireframe at `grid_cols(3)` (`main.rs:1511`):
  ```
  +--268--+------------------------------------------+
  |       | header (no page-level safety pill)       |
  |       | FIXES AND MAINTENANCE                     |
  |       | [tool][tool][tool]  three columns         |
  |       | APPS THAT START AUTOMATICALLY             |
  |       | card of rows: avatar 34, Show button      |
  +-------+-------------------------------------------+
  ```
- Components: `tool_card` title 15 px `main.rs:2856`, body 13 px, embedded `safety`, ghost action. Copy at `main.rs:1438-1459` (repeated for 2-col and 1-col at `main.rs:1475` and `main.rs:1513`). Agent rows `main.rs:1571-1588`.
- Typography: title 34; tool title 15 strong; body 13 `#B3B3B3`; agent label strong; scope line 12 px `#B3B3B3` `main.rs:1579`.
- Spacing: `add_space(14)` before startup section `main.rs:1557`; `add_space(6)` after the intro `main.rs:1564`.
- Interactive elements: "Flush DNS" → `run_tool(Dns)` `main.rs:1550-1551`. "Check status" → `SpotlightStatus`. "Rebuild index" → confirm `main.rs:1539-1544`, action label `"Rebuild"` `main.rs:1542`. "Show" → `reveal` `main.rs:1584`.
- States: loading present `"Reading startup items..."` `main.rs:1566`. Empty present `"No startup items found for your account."` `main.rs:1566`. Error/success: partial, `banner` `main.rs:1428` plus `Notice` from `outcome` `main.rs:3110`.
- Responsive behavior: 1 column if compact, 2 if narrow, 3 otherwise (`main.rs:1471-1548`). Same three cards, duplicated source, not different copy.

### Screen: File Finder

- File(s): `protection_screen` `main.rs:1596-1771`. Enum name is `Screen::Protection` (`main.rs:51`); visible label is `"File Finder"` (`main.rs:79`).
- Route/trigger: sidebar. Does not auto-load (`main.rs:105`).
- Purpose: Walks a folder and lists files whose extensions match the chosen set. Does not delete.
- Screenshot files: `03_filefinder_1180.png`. Empty state visible: "Nothing searched yet". Chips: Downloads and Installers filled green. Path `/Users/hema/Downloads`. Extensions `dmg,pkg`.
- Wireframe:
  ```
  +--268--+----------------------------------+
  |       | header + read-only pill           |
  |       | optional limited-access card      |
  |       | wizard card: steps 1 and 2        |
  |       | primary Start searching h>=40     |
  |       | empty card                        |
  +-------+-----------------------------------+
  ```
- Components: limited-access card only if access != Granted `main.rs:1607`. Step badges `step_label` `main.rs:1623`, `main.rs:1636`. Chips `main.rs:1626-1648`. `field` `main.rs:1633`. Extensions `TextEdit` width 300 monospace `main.rs:1652-1658`. Buttons `main.rs:1664-1684`. Empty `main.rs:1735`.
- Typography: step title `heading_font(15)` `main.rs:2758`; chip text 13 px `main.rs:3046`; empty heading 16 px.
- Spacing: `add_space(8)` after access card `main.rs:1619`; `add_space(10)` between steps `main.rs:1635`, `main.rs:1662`; `add_space(12)` before results `main.rs:1687`.
- Interactive elements: chip sets `app.root` / `app.watch` `main.rs:1628`, `main.rs:1645`. "Start searching" → `start_scan` `main.rs:1665`. Pause/Resume toggles `ExecutionControl` `main.rs:1668-1676`. Stop → `cancel` `main.rs:1679-1682`. "Show" on a hit → `reveal` `main.rs:1755`.
- States: empty present `main.rs:1735` and `main.rs:1737` (in the PNG). Loading present: spinner card while `busy && scan_ran` `main.rs:1689` (not in PNG). Error partial: `banner` plus per-row error count `stat` `main.rs:1713`. Success partial: stat row after scan `main.rs:1708`.
- Responsive behavior: stat row uses `grid_cols(3)` `main.rs:1709-1728`. Chip row is `horizontal_wrapped` `main.rs:1625` (wraps, no breakpoint).

### Screen: Threat Scan

- File(s): `malware_screen` `main.rs:1773-1911`. Label `"Threat Scan"` `main.rs:80`.
- Route/trigger: sidebar. No auto-load (`main.rs:105`).
- Purpose: Hashes files in a chosen folder against a public list and confirms before Trash.
- Screenshot files: `03_threat_1180.png` (nav bar y 886–1000). Empty copy is `"Nothing scanned yet"` `main.rs:1838` when `threat` is `None`.
- Wireframe:
  ```
  +--268--+----------------------------------+
  |       | header + trash safety pill        |
  |       | disclaimer card                    |
  |       | step 1 chips + path field + Scan   |
  |       | empty / spinner / hit list         |
  +-------+------------------------------------+
  ```
- Components: disclaimer `main.rs:1783-1792`. Chips Downloads, Desktop, Documents, Launch Agents `main.rs:1801-1806`. `primary("Scan")` `main.rs:1821`. Busy card `main.rs:1827-1834`. `stat("Files hashed")` `main.rs:1842`. Hit row buttons Trash / Ignore / Show `main.rs:1882-1888`. Badge `"hash match"` color `ROSE` `main.rs:1891`.
- Typography: disclaimer body 13 px `#B3B3B3` `main.rs:1786-1788`. Stat value 28 px `main.rs:3011`.
- Spacing: `add_space(8)` `main.rs:1794`, `main.rs:1814`, `main.rs:1820`; `add_space(12)` `main.rs:1825`.
- Interactive elements: "Scan" → `start_threat` `main.rs:1822`. "Trash" opens confirm `main.rs:1902-1909`. "Ignore" → `ignore_threat` `main.rs:1899` (no confirm). "Show" → `reveal`. "Open Privacy settings" `main.rs:1790` URL `main.rs:1791`.
- States: empty present `main.rs:1838` and no-hits `main.rs:1861`. Loading present `main.rs:1827` (returns early, hides results). Error present as `Notice` `main.rs:596-598` and `main.rs:610` (`"Choose a folder to scan."`, `"the malware scan stopped before it finished"`). Success partial via `banner`.
- Responsive behavior: no column split in this function. Sidebar breakpoint still applies.

### Screen: Big Downloads

- File(s): `clutter_screen` `main.rs:1913-2008`. Label `"Big Downloads"` `main.rs:81`. Threshold `50 * 1024 * 1024` in `start_clutter` `main.rs:473`.
- Route/trigger: sidebar, or Overview row `main.rs:1193`.
- Purpose: Lists Downloads files of at least 50 MB and confirms each Trash move.
- Screenshot files: `03_downloads_1180.png` (nav bar y 1028–1116).
- Wireframe: header, trash pill, either empty card or hero (38 px) plus one card of rows with meter width `(available * 0.55).clamp(160, 420)` `main.rs:1961`.
- Components: empty heading `"Your Downloads folder is tidy"` `main.rs:1928`. Hero `"LARGE FILES FOUND"` `main.rs:1940`, size 38 `main.rs:1941`. Row name shortened to 55 chars `main.rs:1965`. Buttons "Show" `main.rs:1979`, "Move to Trash" `main.rs:1982`.
- Typography: hero 38 px `#1DB954`; file name strong; path 12 px `#B3B3B3` `main.rs:1971`; size strong `#1DB954` `main.rs:1977`.
- Spacing: `add_space(14)` after hero `main.rs:1951`; `add_space(8)` between size and buttons `main.rs:1978`.
- Interactive elements: "Look again" `main.rs:1929` and `main.rs:1945` clears loaded Clutter. Trash sets confirm `main.rs:1983-1994`.
- States: loading present `main.rs:1926`. Empty present `main.rs:1928` (also the success-looking tidy message). Error partial (`banner`). Success partial (notice `main.rs:491`).
- Responsive behavior: row layout is one horizontal group; meter width scales with available width (`main.rs:1961`). No `grid_cols` branch.

### Screen: Space Lens

- File(s): `space_screen` `main.rs:2184-2355`, panes `main.rs:2357-2531`, registry `main.rs:2089-2181`
- Route/trigger: sidebar `"Space Lens"` `main.rs:82`. No auto-load.
- Purpose: Measures a folder and shows a list plus a bubble map; Trash only after confirm.
- Screenshot files: `03_spacelens_1180.png`. Empty state "No map yet" is visible. Chips include "Macintosh HD" filled. Target text `/System/Volumes/Data`. Two "Scan" buttons (one with icon in the card `main.rs:2241`, one below the empty card `main.rs:2334`).
- Wireframe before a report (this PNG):
  ```
  +--268--+----------------------------------+
  |       | header (no safety pill)            |
  |       | setup card: chips + Scan           |
  |       | empty "No map yet" + primary Scan  |
  +-------+------------------------------------+
  ```
  After a report (`main.rs:2263`): path row, total at 22 px `main.rs:2282`, then list pane + bubble pane + registry. List width `(pair * 0.46).clamp(200, 420)` `main.rs:2376`. Registry 248 if width > 780 and not compact `main.rs:2371-2373`. Pane height 480, or 380 if compact `main.rs:2378`.
- Components: permission card `main.rs:2044` (not shown if access is granted; PNG shows "Full disk access"). `chip`s `main.rs:2207-2225`. Collapsing `"Custom folder"` `main.rs:2228`. Bubble diameter `(36 + 70 * sqrt(share)) * scale` `main.rs:2960`, scale 0.72 if compact `main.rs:2959`. Hover card max width 360 `main.rs:2074`, offset (14, 14) `main.rs:2071`.
- Typography: total 22 px strong `#1DB954` `main.rs:2282`. Bubble label 11 px, or 10 px if compact `main.rs:2973`. List meta 11.5 px `main.rs:2451`. Selection bytes 26 px `main.rs:2109`.
- Spacing: `add_space(10)` after setup `main.rs:2259`; `add_space(8)` after total `main.rs:2286`; pane gap 12 `main.rs:2374`.
- Interactive elements: "Scan" → `start_space` `main.rs:2242`, `main.rs:2335`. "Up one level" `main.rs:2265`. Double-click directory → drill `main.rs:2471`. Secondary click → reveal `main.rs:2474`. Checkbox toggles `space_selected` `main.rs:2429`. "Remove" opens confirm `main.rs:2164`. "Clear selection" `main.rs:2176`. Registry "×" is `ghost` `main.rs:2143`.
- States: loading present `"Building your storage map…"` `main.rs:2262` and inline spinner `main.rs:2248`. Empty present `"No map yet"` `main.rs:2328` (in PNG) and `"Nothing to show"` `main.rs:2305`. Error present as `Notice` strings `main.rs:659`, `main.rs:668`, `main.rs:699`. Success partial via `banner` after trash `main.rs:580`.
- Responsive behavior: compact drops the side registry under the panes (`main.rs:2397`). Bubble scale 0.72 (`main.rs:2959`). Pane height 380 vs 480 (`main.rs:2378`).

### Screen: Cloud Storage

- File(s): `cloud_screen` `main.rs:2534-2588`. Label `"Cloud Storage"` `main.rs:83`.
- Route/trigger: sidebar. Auto-loads (`main.rs:105` is false only for Protection, Malware, Space).
- Purpose: Shows on-disk size of iCloud and other cloud folders. Does not upload or delete.
- Screenshot files: `03_cloud_1180.png` (nav bar y 1286–1400).
- Wireframe: header, read-only pill, one `card` per folder (`main.rs:2552`), meter color `SKY` `#78BEF0` `main.rs:2577`, then `ghost("Measure again")` `main.rs:2585`.
- Components: label 16 px strong `main.rs:2555`; path 12.5 px or `"This service is not set up on this Mac."` `main.rs:2557-2560`; bytes 18 px `LIME` `main.rs:2571`; "Show" `main.rs:2568`.
- Typography: title 34; folder name 16; path 12.5 `#B3B3B3`; bytes 18 `#1DB954`.
- Spacing: meter `add_space(4)` `main.rs:2576`; before measure `add_space(6)` `main.rs:2584`.
- Interactive elements: "Show" → `reveal` `main.rs:2581`. "Measure again" clears loaded Cloud `main.rs:2586`.
- States: loading present `"Measuring cloud folders..."` `main.rs:2546`. Empty present `"No cloud folders found."` `main.rs:2546`. Error partial (`banner`). Success: absent as its own widget.
- Responsive behavior: no `grid_cols` in this function. Cards are full content width.

### Screen: System Monitor

- File(s): `assistant_screen` `main.rs:2590-2676`. Label `"System Monitor"` `main.rs:84`.
- Route/trigger: sidebar. Auto-loads via `sample_machine` `main.rs:346`.
- Purpose: Shows one CPU sample and one memory sample and a one-line verdict.
- Screenshot files: `03_monitor_1180.png`. Visible values: PROCESSOR 11% (green), MEMORY 65%, "15.6 GB of 24.0 GB in use", verdict "Everything looks fine.", button "Refresh".
- Wireframe (not compact, `main.rs:2640`):
  ```
  +--268--+----------------------------------+
  |       | header + read-only pill            |
  |       | [PROCESSOR card] [MEMORY card]     |
  |       | verdict card                       |
  |       | ghost Refresh                      |
  +-------+------------------------------------+
  ```
- Components: value 36 px colored by `gauge` `main.rs:2615`. Thresholds: `> 0.85` `ROSE`, `> 0.65` `AMBER`, else `LIME` `main.rs:2998-3005`. Meter height 6. Verdict strings `main.rs:2662-2667`.
- Typography: 36 px strong, color from `gauge`; caption 13 px `#B3B3B3` `main.rs:2617`; verdict strong, `AMBER` or `LIME`.
- Spacing: `add_space(8)` before verdict `main.rs:2661`; `add_space(6)` before Refresh `main.rs:2672`.
- Interactive elements: "Refresh" → `sample_machine` `main.rs:2673-2674`.
- States: loading present `"Taking a reading..."` `main.rs:2602`. Empty present `"No reading yet."` `main.rs:2602` (only if `sample` is `None` and not busy). Error: absent in this function (sample fallback is zeros `main.rs:522-526`, still renders). Success: the filled cards in the PNG.
- Responsive behavior: compact stacks the two cards (`main.rs:2620`). Otherwise `columns(2)` `main.rs:2640`.

### Screen: Confirm modal

- File(s): `confirm_modal` `main.rs:825-878`
- Route/trigger: `app.confirm = Some(...)` from Trash, threat trash, space remove, or Spotlight rebuild. Not opened by navigation.
- Purpose: Asks before Trash or Spotlight reindex.
- Screenshot files: NOT CAPTURED. Click automation was not aimed at the confirm button, to avoid sending the confirm action.
- Wireframe:
  ```
  +---------- full screen scrim black alpha 170 ----------+
  |         window w 420 (320 if compact)                 |
  |         glass_card margin 26, no title bar            |
  |         alert icon 16 + heading                        |
  |         body #B3B3B3                                   |
  |         primary (min 120x40) + ghost Cancel (70x32)    |
  +--------------------------------------------------------+
  ```
  Width `main.rs:832`. Scrim `main.rs:839`. Frame `main.rs:850`.
- Components: `icon_with_label` Alert `main.rs:853`. `add_space(4)` then `add_space(6)` `main.rs:854-855`. `add_space(18)` before buttons `main.rs:857`.
- Typography: heading uses `icon_with_label` label size 12.5 (`icons.rs:22`) color `TEXT`. Body `MUTED` `#B3B3B3` `main.rs:856`. Button text 14 px semibold (`theme.rs:61`).
- Spacing: 4 + 6 between title and body; 18 before buttons (`main.rs:854-857`).
- Interactive elements: primary runs `perform` `main.rs:871-873`. "Cancel" and Escape set `confirm = None` `main.rs:862-867`, `main.rs:876`.
- States: this modal is the confirm state. No loading state inside it. Error/success happen after it closes, via `banner`.
- Responsive behavior: width 320 when `layout.compact` (`main.rs:832`). No other change.

### Screen: Custom Utility (Electron)

- File(s): `CustomUtilityApp/frontend/src/renderer/index.html`, `styles.css`, `renderer.ts`
- Route/trigger: `npm start` (`frontend/package.json:9`). Single page, no router.
- Purpose: Starts, pauses, resumes, and cancels a scan and shows three counters plus a path list.
- Screenshot files: NOT CAPTURED. App was not started.
- Wireframe from CSS (px):
  ```
  +-- viewport -------------------------------+
  | main max-width 720, pad 32 20 48           |
  | h1 1.6rem                                  |
  | label margin 24 0 12, input pad 10x12      |
  | .controls gap 8, margin-bottom 20          |
  | .metrics 3 columns, gap 12                 |
  | #status                                    |
  | ul max-height 220                          |
  +--------------------------------------------+
  ```
- Components in order (`index.html:11-44`): `h1` "Custom Utility"; paragraph "Chunked filesystem walk. Detection is a placeholder. Nothing is deleted."; label "Root folder"; `input#root` placeholder `/Users/you/Documents`; buttons `#start` "Start Scan", `#pause` "Pause", `#resume` "Resume", `#cancel` "Cancel"; articles "Files scanned", "Files / second", "Threats" (initial text "0"); `#status` "Waiting for the engine."; `ul#list`.
- Typography: h1 `1.6rem` (`styles.css:20`); body inherits Segoe UI (`styles.css:3`); label `0.85rem` (`styles.css:31`); metric label `0.8rem` `#b0b6be`; metric value `1.4rem` (`styles.css:90`); list `0.8rem` monospace (`styles.css:104-105`).
- Spacing: main padding `32px 20px 48px` (`styles.css:15`); controls gap `8px` (`styles.css:49`); metrics gap `12px` (`styles.css:72`); article padding `14px` (`styles.css:78`).
- Interactive elements: click handlers `renderer.ts:70-88`. Status strings `renderer.ts:49-66`.
- States: waiting text in HTML (`index.html:43`). Scanning / paused / finished / error strings in `renderer.ts:49-66`. No separate empty-list widget (the `ul` starts empty). No loading spinner in HTML/CSS.
- Responsive behavior: none. No `@media` in `styles.css`. Three columns stay `repeat(3, 1fr)` (`styles.css:71`) at every width.
