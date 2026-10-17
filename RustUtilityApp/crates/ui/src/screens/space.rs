use std::path::PathBuf;

use dioxus::prelude::*;
use engine::computer_root;

use crate::actions::{space_back, space_forward, space_open, start_space};
use crate::components::{NoticeBanner, PageHeader, Safety};
use crate::state::{AppState, Confirm, ConfirmKind, Screen};
use crate::util::{file_name, format_bytes, pretty, reveal};

#[component]
pub fn SpaceScreen(mut state: Signal<AppState>) -> Element {
    let busy = state.read().busy;
    let home = state.read().home.clone();
    let space_root = state.read().space_root.clone();
    let space = state.read().space.clone();
    let can_back = !state.read().space_history.is_empty();
    let can_forward = !state.read().space_future.is_empty();
    let selected = state.read().space_selected.clone();
    let selected_bytes: u64 = state.read().space_selected_bytes.values().copied().sum();

    rsx! {
        PageHeader {
            screen: Screen::Space,
            subtitle: "One scan builds a map. Then open folders from that map without rescanning.",
            safety: Safety::Trash,
            action_label: Some(if busy { "Scanning…".into() } else { "Scan".into() }),
            action_enabled: !busy,
            on_action: Some(EventHandler::new(move |_| start_space(state, true))),
        }
        NoticeBanner { state }

        div { class: "toolbar",
            button {
                class: "btn btn-secondary",
                disabled: !can_back || busy,
                onclick: move |_| space_back(state),
                "← Back"
            }
            button {
                class: "btn btn-secondary",
                disabled: !can_forward || busy,
                onclick: move |_| space_forward(state),
                "Forward →"
            }
            button {
                class: "chip",
                onclick: move |_| {
                    state.write().space_root = home.display().to_string();
                    state.write().space_history.clear();
                    state.write().space_future.clear();
                    start_space(state, true);
                },
                "Home"
            }
            button {
                class: "chip",
                onclick: move |_| {
                    state.write().space_root = computer_root().display().to_string();
                    state.write().space_history.clear();
                    state.write().space_future.clear();
                    start_space(state, true);
                },
                "This Mac"
            }
            span { class: "row-sub", style: "margin-left: auto;", "{space_root}" }
        }

        if let Some(report) = space {
            {
                let total = report.total.max(1);
                rsx! {
                    div { class: "summary-bar",
                        span { "Folder total: " strong { "{format_bytes(report.total)}" } }
                        span { strong { "{report.children.len()}" } " items" }
                        if !selected.is_empty() {
                            span { strong { "{format_bytes(selected_bytes)}" } " selected" }
                        }
                    }

                    div { class: "card",
                        for child in report.children {
                            {
                                let path = child.path.clone();
                                let path_open = child.path.clone();
                                let path_sel = child.path.clone();
                                let bytes = child.bytes;
                                let is_dir = child.is_directory;
                                let checked = selected.contains(&child.path);
                                let pct = ((child.bytes as f64 / total as f64) * 100.0).clamp(0.0, 100.0);
                                let name = file_name(&child.path);
                                rsx! {
                                    div { class: "row",
                                        input {
                                            r#type: "checkbox",
                                            checked: checked,
                                            disabled: busy || !child.is_purgeable,
                                            onchange: move |_| {
                                                let mut s = state.write();
                                                if s.space_selected.contains(&path_sel) {
                                                    s.space_selected.remove(&path_sel);
                                                    s.space_selected_bytes.remove(&path_sel);
                                                } else {
                                                    s.space_selected.insert(path_sel.clone());
                                                    s.space_selected_bytes.insert(path_sel.clone(), bytes);
                                                }
                                            },
                                        }
                                        div {
                                            class: "row-body",
                                            style: "cursor: pointer;",
                                            onclick: move |_| {
                                                if is_dir {
                                                    space_open(state, path_open.clone());
                                                } else {
                                                    reveal(&path_open);
                                                }
                                            },
                                            div { class: "row-title",
                                                if is_dir { "📁 {name}" } else { "📄 {name}" }
                                            }
                                            div { class: "row-sub", "{pretty(&home, &path)} · {pct:.1}%" }
                                            div { class: "meter",
                                                span { style: "width: {pct}%;" }
                                            }
                                        }
                                        div { class: "row-meta", "{format_bytes(bytes)}" }
                                    }
                                }
                            }
                        }
                    }

                    if !selected.is_empty() {
                        div { class: "care-footer", style: "margin-top: 16px;",
                            button {
                                class: "btn btn-danger btn-lg",
                                disabled: busy,
                                onclick: move |_| {
                                    let paths: Vec<PathBuf> =
                                        state.read().space_selected.iter().cloned().collect();
                                    state.write().confirm = Some(Confirm {
                                        heading: format!("Move {} items to Trash?", paths.len()),
                                        body: format!(
                                            "{} will be moved to the Trash.",
                                            format_bytes(selected_bytes)
                                        ),
                                        action: "Move to Trash",
                                        kind: ConfirmKind::TrashSpace(paths),
                                    });
                                },
                                "Trash selected ({format_bytes(selected_bytes)})"
                            }
                        }
                    }
                }
            }
        } else if busy {
            div { class: "card loading", div { class: "spinner" } "Building disk map..." }
        } else {
            div { class: "card empty",
                h3 { "Scan to map this folder" }
                p { "Press Scan once. Then click folders to drill in without waiting again." }
            }
        }
    }
}
