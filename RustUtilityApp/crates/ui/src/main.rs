mod actions;
mod components;
mod screens;
mod state;
mod util;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

use actions::ensure_auto_load;
use components::{ConfirmModal, Sidebar};
use screens::*;
use state::{AppState, Screen};

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("Clean Your Mac")
                    .with_inner_size(LogicalSize::new(1180.0, 800.0))
                    .with_min_inner_size(LogicalSize::new(720.0, 520.0)),
            ),
        )
        .launch(App);
}

const CSS: Asset = asset!("/assets/styles.css");
// Keep font files in the asset graph so @font-face URLs resolve in the desktop bundle.
const _FONT_REGULAR: Asset = asset!("/assets/CleanYourMac-Regular.ttf");
const _FONT_SEMIBOLD: Asset = asset!("/assets/CleanYourMac-SemiBold.ttf");

#[component]
fn App() -> Element {
    let state = use_signal(AppState::new);

    use_effect(move || {
        ensure_auto_load(state);
    });

    let screen = state.read().screen;
    let busy = state.read().busy;

    rsx! {
        document::Stylesheet { href: CSS }
        div { class: "app",
            Sidebar { state }
            main { class: "main",
                div { class: if busy { "busy-strip on" } else { "busy-strip" } }
                div { class: "main-scroll",
                    match screen {
                        Screen::Care => rsx! { CareScreen { state } },
                        Screen::Cleanup => rsx! { CleanupScreen { state } },
                        Screen::Apps => rsx! { AppsScreen { state } },
                        Screen::Performance => rsx! { PerformanceScreen { state } },
                        Screen::Protection => rsx! { ProtectionScreen { state } },
                        Screen::Malware => rsx! { MalwareScreen { state } },
                        Screen::Clutter => rsx! { ClutterScreen { state } },
                        Screen::Space => rsx! { SpaceScreen { state } },
                        Screen::Cloud => rsx! { CloudScreen { state } },
                        Screen::Assistant => rsx! { AssistantScreen { state } },
                    }
                }
            }
            ConfirmModal { state }
        }
    }
}
