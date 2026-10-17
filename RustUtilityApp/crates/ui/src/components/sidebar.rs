use dioxus::prelude::*;

use crate::state::{rail_groups, AppState};

#[component]
pub fn Sidebar(state: Signal<AppState>) -> Element {
    let screen = state.read().screen;
    let busy = state.read().busy;
    let status = state.read().status;
    let granted = state.read().access == engine::AccessStatus::Granted;
    let (dot, label, hint) = if busy {
        ("busy", "Working", status)
    } else if granted {
        ("ok", "Ready", "Full Disk Access on")
    } else {
        ("warn", "Limited access", "Some folders need permission")
    };

    rsx! {
        aside { class: "rail",
            div { class: "brand",
                div { class: "brand-mark", "C" }
                div { class: "brand-text",
                    span { class: "brand-name", "Clean Your Mac" }
                    span { class: "brand-sub", "Storage & care" }
                }
            }
            nav { class: "nav-groups",
                for (group, items) in rail_groups() {
                    div {
                        div { class: "nav-group-label", "{group}" }
                        for item in items.iter().copied() {
                            button {
                                class: if item == screen { "nav-item active" } else { "nav-item" },
                                onclick: move |_| state.write().go(item),
                                span { class: "nav-icon", "{item.icon()}" }
                                span { "{item.label()}" }
                            }
                        }
                    }
                }
            }
            div { class: "rail-foot",
                span { class: "status-dot {dot}" }
                div { class: "status-meta",
                    span { class: "status-label", "{label}" }
                    span { class: "status-hint", "{hint}" }
                }
            }
        }
    }
}
