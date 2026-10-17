use dioxus::prelude::*;

use crate::actions::run_checked_care;
use crate::components::NoticeBanner;
use crate::state::{AppState, Screen};
use crate::util::{format_bytes, scan_age};

#[component]
pub fn CareScreen(mut state: Signal<AppState>) -> Element {
    let reclaim = {
        let s = state.read();
        if s.junk.is_empty() {
            s.care.as_ref().map(|r| r.cleanup_bytes)
        } else {
            Some(s.junk.iter().map(|r| r.bytes).sum())
        }
    };
    let threats = state.read().threat.as_ref().map(|r| r.hits.len());
    let agents = state
        .read()
        .care
        .as_ref()
        .map(|r| r.agent_count)
        .unwrap_or(state.read().agents.len());
    let apps = state
        .read()
        .care
        .as_ref()
        .map(|r| r.app_count)
        .unwrap_or(state.read().apps.len());
    let leftovers = if state.read().leftovers_done {
        Some(state.read().leftovers.len())
    } else {
        None
    };
    let downloads = if state.read().loaded.contains(&Screen::Clutter) {
        Some(state.read().clutter.len())
    } else {
        None
    };
    let threats_clear = matches!(threats, Some(0));
    let busy = state.read().busy;
    let picks = state.read().care_picks.clone();
    let selected = [
        Screen::Cleanup,
        Screen::Performance,
        Screen::Clutter,
        Screen::Malware,
    ]
    .iter()
    .filter(|s| {
        picks.contains(s) && !(**s == Screen::Malware && threats_clear)
    })
    .count();
    let age = scan_age(state.read().last_refresh);
    let size = reclaim.map(format_bytes).unwrap_or_else(|| "—".into());
    let loading = state.read().care.is_none() && busy && state.read().care_followups.is_empty();

    let cleanup_metric = reclaim.map(format_bytes).unwrap_or_else(|| "—".into());
    let malware_metric = match threats {
        Some(0) => "No threats found".to_string(),
        Some(count) => format!("{count} threats"),
        None => "Not scanned".to_string(),
    };
    let malware_detail: &'static str = match threats {
        Some(0) => "You're clear",
        Some(_) => "to remove",
        None => "Threat scan has not run",
    };
    let apps_metric = match leftovers {
        Some(count) if count > 0 => format!("{count} leftover files"),
        _ => format!("{apps} apps"),
    };
    let clutter_metric = match downloads {
        Some(count) => format!("{count} large downloads"),
        None => "Not scanned".to_string(),
    };
    let clutter_detail: &'static str = if downloads.is_some() {
        "to remove"
    } else {
        "Large files in Downloads"
    };

    rsx! {
        NoticeBanner { state }
        div { class: "page-kicker", "Home" }
        h1 { class: "page-title", "Your tasks are ready" }
        p { class: "page-sub", "Pick what to run, then clean in one pass. Everything that deletes asks first." }

        div { class: "summary-bar",
            span { "Last scan: " strong { "{age}" } }
            span { strong { "{size}" } " reclaimable" }
            span { strong { "{selected}" } " tasks selected" }
        }

        if loading {
            div { class: "card loading",
                div { class: "spinner" }
                "Checking your Mac..."
            }
        } else {
            div { class: "grid grid-3",
                CareTask {
                    state,
                    screen: Screen::Cleanup,
                    title: "Cleanup",
                    metric: cleanup_metric,
                    detail: "of junk to clean",
                    selectable: true,
                    success: false,
                }
                CareTask {
                    state,
                    screen: Screen::Malware,
                    title: "Protection",
                    metric: malware_metric,
                    detail: malware_detail,
                    selectable: !threats_clear,
                    success: threats_clear,
                }
                CareTask {
                    state,
                    screen: Screen::Performance,
                    title: "Performance",
                    metric: format!("{agents} items"),
                    detail: "to review",
                    selectable: true,
                    success: false,
                }
                CareTask {
                    state,
                    screen: Screen::Apps,
                    title: "Applications",
                    metric: apps_metric,
                    detail: "installed",
                    selectable: false,
                    success: false,
                }
                CareTask {
                    state,
                    screen: Screen::Clutter,
                    title: "My Clutter",
                    metric: clutter_metric,
                    detail: clutter_detail,
                    selectable: true,
                    success: false,
                }
            }
        }

        div { class: "care-footer",
            button {
                class: "btn btn-primary btn-lg",
                disabled: busy || selected == 0,
                onclick: move |_| run_checked_care(state),
                if busy { "Running…" } else { "Run selected" }
            }
        }
    }
}

#[component]
fn CareTask(
    mut state: Signal<AppState>,
    screen: Screen,
    title: &'static str,
    metric: String,
    detail: &'static str,
    selectable: bool,
    success: bool,
) -> Element {
    let picked = state.read().care_picks.contains(&screen);
    let class = if selectable && picked {
        "card interactive task-card selectable selected"
    } else if selectable {
        "card interactive task-card selectable"
    } else {
        "card task-card"
    };
    let metric_style = if success {
        "color: var(--success)"
    } else {
        ""
    };
    let action = if screen == Screen::Apps {
        "Browse"
    } else {
        "Review"
    };

    rsx! {
        div {
            class: "{class}",
            onclick: move |_| {
                if !selectable {
                    return;
                }
                let mut s = state.write();
                if s.care_picks.contains(&screen) {
                    s.care_picks.remove(&screen);
                } else {
                    s.care_picks.insert(screen);
                }
            },
            div { class: "task-top",
                div { class: "task-icon", "{screen.icon()}" }
                if selectable {
                    div { class: if picked { "task-check on" } else { "task-check" },
                        if picked { "✓" }
                    }
                }
            }
            div { class: "task-title", "{title}" }
            div { class: "task-metric", style: "{metric_style}", "{metric}" }
            div { class: "task-detail", "{detail}" }
            div { class: "task-actions",
                button {
                    class: "btn btn-ghost",
                    onclick: move |e| {
                        e.stop_propagation();
                        state.write().go(screen);
                    },
                    "{action}"
                }
            }
        }
    }
}
