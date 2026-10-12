# 06 — Content dump

Strings below are user-visible. Format placeholders stay as `{}`. Duplicate copies of the same sentence in the Tune-Up breakpoint branches are cited once, with the other line numbers.

## Shell / sidebar

| string | file:line |
|---|---|
| Sable | `main.rs:994` (also window title `main.rs:220`) |
| For your Mac | `main.rs:995` |
| Sable — Mac cleaner and tune-up | `main.rs:984` |
| TOOLS / MENU | `main.rs:911` |
| Overview | `main.rs:75` |
| Free Up Space | `main.rs:76` |
| Apps | `main.rs:77` |
| Tune-Up | `main.rs:78` |
| File Finder | `main.rs:79` |
| Threat Scan | `main.rs:80` |
| Big Downloads | `main.rs:81` |
| Space Lens | `main.rs:82` |
| Cloud Storage | `main.rs:83` |
| System Monitor | `main.rs:84` |
| Health check of your Mac | `main.rs:90` |
| Caches and logs | `main.rs:91` |
| Installed apps and leftovers | `main.rs:92` |
| Startup items and fixes | `main.rs:93` |
| Find installers and archives by type | `main.rs:94` |
| Compare files to known malware hashes | `main.rs:95` |
| Large files sitting in Downloads | `main.rs:96` |
| Visual map of what uses your disk | `main.rs:97` |
| Local iCloud and cloud folder sizes | `main.rs:98` |
| Live CPU and memory readings | `main.rs:99` |
| Full disk access | `main.rs:934` |
| Limited disk access | `main.rs:936` |
| Ready | `main.rs:3021` |
| Working... | `main.rs:3020` |
| Paused | `main.rs:3017` |
| Stopping... | `main.rs:3018` |
| Moving to Trash... | `main.rs:3019` |
| SABLE | `main.rs:2728` |
| Read-only — nothing on disk is changed | `main.rs:2711` |
| Moves to Trash — you can undo this | `main.rs:2712` |
| May ask for your Mac password | `main.rs:2713` |
| Done | `main.rs:2793` |
| That did not work | `main.rs:2793` |
| Dismiss | `main.rs:2800` |
| Cancel | `main.rs:862` |
| Nothing here | `main.rs:2900` |

## Overview

| string | file:line |
|---|---|
| A quick health check of your Mac. This page only looks. It never changes or deletes anything. | `main.rs:1063` |
| YOU CAN FREE UP | `main.rs:1083`, `main.rs:1096` |
| of temporary files (caches and logs). Apps rebuild what they need. | `main.rs:1086`, `main.rs:1099` |
| Review and clean up | `main.rs:1090`, `main.rs:1104` |
| Checking your Mac... | `main.rs:1111` |
| APPS | `main.rs:1122` |
| apps installed | `main.rs:1124` |
| Browse apps | `main.rs:1125` |
| STARTUP ITEMS | `main.rs:1131` |
| background items start when you log in | `main.rs:1133` |
| See startup items | `main.rs:1134` |
| DISK ACCESS | `main.rs:2681` |
| Full | `main.rs:2683` |
| Limited | `main.rs:2683` |
| Sable can read every folder it needs. | `main.rs:2690` |
| Some folders are off limits, so sizes may look smaller than they are. | `main.rs:2692` |
| Open Privacy settings | `main.rs:2697` |
| WHERE TO GO NEXT | `main.rs:1191` |
| Find large files in Downloads you may no longer need. | `main.rs:1193` |
| See which folders are using the most disk space. | `main.rs:1194` |
| Check what an app leaves behind in your Library. | `main.rs:1195` |
| Open | `main.rs:1207` |
| Run the check again | `main.rs:1217` |

## Free Up Space

| string | file:line |
|---|---|
| Apps keep temporary files (caches and logs) to run faster. They are safe to remove, and apps recreate what they need. Review each group, then move it to the Trash. | `main.rs:1235` |
| Measuring caches and logs... | `main.rs:1242` |
| Nothing to show yet. | `main.rs:1242` |
| TOTAL YOU CAN CLEAN | `main.rs:1251` |
| Measure again | `main.rs:1255` |
| WHAT TAKES THE SPACE | `main.rs:1262` |
| Move to Trash | `main.rs:1275`, `main.rs:1282` |
| Move {} to the Trash? | `main.rs:1277` |
| {} will be moved to the Trash. Apps recreate these files when needed, and you can put them back from the Trash. | `main.rs:1278-1280` |
| Show | `main.rs:1286` |
| Not on this Mac | `main.rs:1291` |
| Moved "{}" to the Trash. Open the Trash to put it back. | `main.rs:425` |
| User caches / User logs / Diagnostic reports / Spotify cache / Adobe media cache / Xcode derived data | `main.rs:3100-3105` (matched to engine labels) |
| Temporary files apps keep so they open faster. Rebuilt automatically. | `main.rs:3100` |
| Activity notes apps write for troubleshooting. Not needed day to day. | `main.rs:3101` |
| Crash reports macOS saves after an app quits unexpectedly. | `main.rs:3102` |
| Songs Spotify stored for faster playback. It downloads them again if needed. | `main.rs:3103` |
| Preview files from Adobe video and audio apps. Rebuilt on demand. | `main.rs:3104` |
| Build files from Xcode. The next build recreates them. | `main.rs:3105` |
| Temporary files that apps can recreate. | `main.rs:3106` |

## Apps

| string | file:line |
|---|---|
| Everything installed on this Mac. Pick an app to see the files it keeps in your Library, which stay behind even after you drag the app to the Trash. | `main.rs:1314` |
| Looking for installed apps... | `main.rs:1321` |
| No apps found in /Applications or ~/Applications. | `main.rs:1321` |
| Search by app name | `main.rs:1328` |
| {} apps | `main.rs:1332` |
| No app matches your search. | `main.rs:1365` |
| Pick an app above | `main.rs:1378` |
| Its leftover files will appear here. Nothing is deleted from this screen. | `main.rs:1378` |
| FILES {} KEEPS IN YOUR LIBRARY | `main.rs:1382` |
| Searching your Library... | `main.rs:1384` |
| No leftover files found | `main.rs:1386` |
| This app does not keep anything in the usual Library folders. | `main.rs:1386` |
| Show the app in Finder | `main.rs:1414` |

## Tune-Up

| string | file:line |
|---|---|
| See which apps start automatically in the background, and run a few common macOS fixes. | `main.rs:1426` |
| FIXES AND MAINTENANCE | `main.rs:1431` |
| Flush DNS cache | `main.rs:1438` (also `1477`, `1515`) |
| Clears the saved list of website addresses. Try it when sites will not load after switching Wi-Fi or a VPN. | `main.rs:1439` |
| Flush DNS | `main.rs:1441` |
| Check Spotlight | `main.rs:1446` |
| Shows whether Spotlight search is indexing your disk. Only reads the status. | `main.rs:1447` |
| Check status | `main.rs:1450` |
| Rebuild Spotlight | `main.rs:1456` |
| Starts a fresh index when search results are wrong or missing. Can take a while and may ask for your password. | `main.rs:1457` |
| Rebuild index | `main.rs:1459` |
| Rebuild the Spotlight index? | `main.rs:1463` |
| Search may be slow or incomplete for a while as macOS reindexes your disk. Your files are not touched. | `main.rs:1464` |
| Rebuild | `main.rs:1465` |
| APPS THAT START AUTOMATICALLY | `main.rs:1558` |
| These background items launch when you log in. Sable lists them but never turns them off. | `main.rs:1560` |
| Reading startup items... | `main.rs:1566` |
| No startup items found for your account. | `main.rs:1566` |
| Starts for your account only | `main.rs:1578` |
| Starts for every user on this Mac | `main.rs:1578` |
| DNS cache cleared. | `main.rs:457` |
| Spotlight status unavailable. | `main.rs:458` |
| Spotlight is rebuilding its index. | `main.rs:459` |

## File Finder

| string | file:line |
|---|---|
| Looks through a folder and lists files of the types you choose, like old installers or archives. It does not check for viruses and never changes your files. | `main.rs:1601` |
| Sable has limited disk access | `main.rs:1609` |
| Some folders cannot be read and will be skipped. Grant Full Disk Access to include them. | `main.rs:1611` |
| Open Privacy settings | `main.rs:1615` |
| Where should it look? | `main.rs:1623` |
| Downloads, Documents, Desktop, Home folder | `main.rs:1626` |
| Which kinds of files? | `main.rs:1636` |
| Installers, Archives, Disk images, Videos, Documents | `main.rs:1639-1643` |
| Extensions: | `main.rs:1651` |
| dmg, pkg, zip | `main.rs:1656` |
| Start searching | `main.rs:1664` |
| Pause / Resume | `main.rs:1668` |
| Stop | `main.rs:1679` |
| Paused after checking / Checked | `main.rs:1696` |
| Files checked | `main.rs:1711` |
| Matches | `main.rs:1712` |
| Could not read | `main.rs:1713` |
| Nothing searched yet | `main.rs:1735` |
| Choose a folder and file types, then press Start searching. | `main.rs:1735` |
| No matching files | `main.rs:1737` |
| Nothing in that folder uses the extensions you picked. | `main.rs:1737` |
| {} MATCHING FILES | `main.rs:1740` |
| and {} more | `main.rs:1764` |

## Threat Scan

| string | file:line |
|---|---|
| Checks a folder against a public list of malware file hashes from abuse.ch MalwareBazaar. A match is moved to the Trash only after you confirm. | `main.rs:1778` |
| What this scan does not do | `main.rs:1784` |
| It does not use CleanMyMac's signature database. It does not read browser cookies, history, passwords, or messages. It does not change camera, microphone, or other app permissions. It does not watch the network in the background. | `main.rs:1786` |
| Where should it look? | `main.rs:1798` |
| Downloads, Desktop, Documents, Launch Agents | `main.rs:1802-1805` |
| The first scan downloads the public hash list if the saved copy is missing or older than a day. Later scans reuse that copy. | `main.rs:1816` |
| Scan | `main.rs:1821` |
| Updating signatures if needed, then hashing files. | `main.rs:1831` |
| Nothing scanned yet | `main.rs:1838` |
| Choose a folder and press Scan. | `main.rs:1838` |
| Files hashed | `main.rs:1842` |
| {} files were larger than 50 MB and were skipped. {} items could not be read. | `main.rs:1848-1851` |
| No known malware hashes | `main.rs:1861` |
| Nothing in that folder matched the public list. | `main.rs:1861` |
| {} MATCHES | `main.rs:1865` |
| Trash | `main.rs:1882` |
| Ignore | `main.rs:1885` |
| hash match | `main.rs:1891` |
| Move {name} to the Trash? | `main.rs:1905` |
| This file matched a public malware hash. The Trash can restore it. Related helpers and system files are left alone. | `main.rs:1906` |
| Choose a folder to scan. | `main.rs:597` |
| the malware scan stopped before it finished | `main.rs:610` |
| That file will be skipped on later scans. | `main.rs:646` |

## Big Downloads

| string | file:line |
|---|---|
| Large files (50 MB or more) sitting in your Downloads folder. Old installers and videos pile up here. Review them one at a time. | `main.rs:1918` |
| Looking through Downloads... | `main.rs:1926` |
| Your Downloads folder is tidy | `main.rs:1928` |
| No file there is 50 MB or larger. | `main.rs:1928` |
| Look again | `main.rs:1929` |
| LARGE FILES FOUND | `main.rs:1940` |
| across {} files | `main.rs:1942` |
| Move "{}" to the Trash? | `main.rs:1985` |
| This frees {}. You can put it back from the Trash if you change your mind. | `main.rs:1988-1990` |

## Space Lens

| string | file:line |
|---|---|
| Pick a drive or folder, press Scan, then explore the map. Double-click a bubble or folder to zoom in. | `main.rs:2189` |
| Full Disk Access needed | `main.rs:2052` |
| To scan the whole Mac, allow Full Disk Access for Sable in System Settings. Without it, scanning stops here. | `main.rs:2055` |
| Select storage | `main.rs:2204` |
| Macintosh HD | `main.rs:2207` |
| Home, Documents, Downloads, Desktop | `main.rs:2211-2214` |
| Custom folder | `main.rs:2228` |
| Scan | `main.rs:2232`, `main.rs:2241`, `main.rs:2334` |
| Target: {} | `main.rs:2234` |
| Scan again | `main.rs:2244` |
| Indexing folders and measuring sizes in the background… | `main.rs:2253` |
| Building your storage map… | `main.rs:2262` |
| Up one level | `main.rs:2267` |
| {} in this folder | `main.rs:2281` |
| Nothing to show | `main.rs:2305` |
| This folder is empty or cannot be read. | `main.rs:2305` |
| No map yet | `main.rs:2330` |
| Macintosh HD is selected by default. Press Scan to build your storage map. | `main.rs:2331` |
| LIST — LARGEST FIRST | `main.rs:2415` |
| LENS — PROPORTIONAL MAP | `main.rs:2495` |
| Circle size = disk space. Items under 2% roll into “Other”. | `main.rs:2497` |
| Other | `main.rs:2526` |
| SELECTION | `main.rs:2108` |
| {} item(s) marked | `main.rs:2111` |
| Check files in the list to add them here. | `main.rs:2117` |
| × | `main.rs:2143` |
| protected | `main.rs:2136` |
| Protected system paths and folders are blocked. Sable moves files to the Trash (undoable), not permanent delete. | `main.rs:2157` |
| Remove | `main.rs:2164` |
| Clear selection | `main.rs:2176` |
| Move selected files to the Trash? | `main.rs:2167` |
| This frees about {}. System files stay protected. You can restore items from the Trash. | `main.rs:2168-2170` |
| Folder / File | `main.rs:2012`, `main.rs:2018` |
| Modified date unknown / Modified today / Modified yesterday / Modified {n} days ago / months ago / years ago | `main.rs:2024-2040` |
| On this Mac | `main.rs:2952` |
| Cloud, stored locally | `main.rs:2953` |
| Cloud, not on this Mac | `main.rs:2954` |
| Type: {} / Size: {} | `main.rs:2078-2079` |
| Double-click to zoom into this folder | `main.rs:2083` |
| Choose a location first — try “This Mac” or Home. | `main.rs:659` |
| That folder does not exist: {} | `main.rs:668` |
| Full Disk Access is required to scan this drive. Open System Settings → Privacy & Security → Full Disk Access, enable Sable, then press Scan again. | `main.rs:699` |
| Moved {count} files to the Trash. Open the Trash to put them back. | `main.rs:580` |

## Cloud Storage

| string | file:line |
|---|---|
| How much space iCloud Drive and other cloud services use on this Mac. Nothing is signed in, uploaded, or deleted. | `main.rs:2539` |
| Measuring cloud folders... | `main.rs:2546` |
| No cloud folders found. | `main.rs:2546` |
| This service is not set up on this Mac. | `main.rs:2560` |
| Measure again | `main.rs:2585` |

## System Monitor

| string | file:line |
|---|---|
| How hard your Mac is working right now. Press Refresh for a new reading. | `main.rs:2595` |
| Taking a reading... | `main.rs:2602` |
| No reading yet. | `main.rs:2602` |
| PROCESSOR | `main.rs:2623` |
| How busy the CPU is. | `main.rs:2626` |
| MEMORY | `main.rs:2630` |
| {} of {} in use | `main.rs:2633-2636` |
| Memory is nearly full. Quit apps you are not using. | `main.rs:2663` |
| The processor is very busy. Something heavy is running. | `main.rs:2665` |
| Everything looks fine. | `main.rs:2667` |
| Refresh | `main.rs:2673` |

## Custom Utility (`index.html`, `renderer.ts`)

| string | file:line |
|---|---|
| Custom Utility | `index.html:12` (title `index.html:6`) |
| Chunked filesystem walk. Detection is a placeholder. Nothing is deleted. | `index.html:13` |
| Root folder | `index.html:17` |
| /Users/you/Documents | `index.html:18` placeholder |
| Start Scan | `index.html:22` |
| Pause | `index.html:23` |
| Resume | `index.html:24` |
| Cancel | `index.html:25` |
| Files scanned | `index.html:30` |
| Files / second | `index.html:34` |
| Threats | `index.html:38` |
| Waiting for the engine. | `index.html:43` |
| Enter a folder path. | `renderer.ts:73` |
| Engine online. Choose a folder and start. | `renderer.ts:50` |
| Threat flagged (queued, not deleted). | `renderer.ts:55` |
| Engine error | `renderer.ts:57` |
| Scan canceled. | `renderer.ts:59` |
| Scan finished. | `renderer.ts:59` |
| Paused. | `renderer.ts:61` |
| Scanning. | `renderer.ts:63`, `renderer.ts:66` |

## Voice

- Sentences are full and spoken to "you" / "your Mac", not telegraphic UI chrome. Example: "This page only looks. It never changes or deletes anything." (`main.rs:1063`). Example: "You can put it back from the Trash if you change your mind." (`main.rs:1989`).
- Technical nouns are kept, then explained in the same sentence. Example: "Clears the saved list of website addresses. Try it when sites will not load after switching Wi-Fi or a VPN." (`main.rs:1439`). Example: "Compares" is avoided; the screen says "public list of malware file hashes from abuse.ch MalwareBazaar" (`main.rs:1778`).
- Section labels are short and uppercase ("YOU CAN FREE UP" `main.rs:1096`, "WHAT TAKES THE SPACE" `main.rs:1262`), while body lines often run past 80 characters (cleanup subtitle is one string at `main.rs:1235`; threat disclaimer is one string at `main.rs:1786`).
