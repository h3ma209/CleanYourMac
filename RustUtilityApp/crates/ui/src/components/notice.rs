use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn NoticeBanner(mut state: Signal<AppState>) -> Element {
    let Some(notice) = state.read().notice.clone() else {
        return rsx! {};
    };
    let class = if notice.ok { "notice ok" } else { "notice err" };
    rsx! {
        div { class: "{class}",
            span { "{notice.text}" }
            button {
                class: "btn btn-ghost",
                onclick: move |_| state.write().notice = None,
                "Dismiss"
            }
        }
    }
}
