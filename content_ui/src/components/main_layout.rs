use crate::{components::Navbar, routes::Route};
use dioxus::prelude::*;

/// Main layout component that wraps pages with navigation
#[component]
pub fn MainLayout(children: Element) -> Element {
    rsx! {
        Navbar {}
        div {
            class: "min-h-screen bg-gray-50",
            div {
                class: "pt-16",
                Outlet::<Route> {}
            }
        }
    }
}
