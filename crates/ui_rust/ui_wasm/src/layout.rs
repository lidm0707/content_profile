use ui_core::components::Navbar;
use dioxus::prelude::*;

use crate::routes::AppRoute;

/// Layout for the backend app: ui_core navbar plus a Settings link,
/// wrapping pages through the outlet.
#[component]
pub fn AppBar() -> Element {
    rsx! {
        Navbar {}
        div {
            class: "min-h-screen bg-gray-50",
            div {
                class: "pt-16",
                Outlet::<AppRoute> {}
            }
        }
    }
}
