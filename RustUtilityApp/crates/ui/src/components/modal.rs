use dioxus::prelude::*;

use crate::actions::perform_confirm;
use crate::state::AppState;

#[component]
pub fn ConfirmModal(mut state: Signal<AppState>) -> Element {
    let Some(confirm) = state.read().confirm.clone() else {
        return rsx! {};
    };
    let heading = confirm.heading.clone();
    let body = confirm.body.clone();
    let action = confirm.action;
    let kind = confirm.kind.clone();

    rsx! {
        div {
            class: "scrim",
            onclick: move |_| state.write().confirm = None,
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 { "{heading}" }
                p { "{body}" }
                div { class: "modal-actions",
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            state.write().confirm = None;
                            perform_confirm(state, kind.clone());
                        },
                        "{action}"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| state.write().confirm = None,
                        "Not now"
                    }
                }
            }
        }
    }
}
