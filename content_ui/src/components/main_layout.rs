use dioxus::prelude::*;
use crate::components::Navbar;
use crate::routes::Route;

/// Main layout component that provides a fixed navbar with content padding
/// This ensures all page content is visible below the fixed navigation bar
#[component]
pub fn MainLayout() -> Element {
    rsx! {
        // Fixed navbar at the top of the page
        Navbar {}

        // Main content area with top padding to prevent overlap with navbar
        // The navbar has h-16 (64px) height, so we add pt-16 (4rem = 64px)
        div {
            class: "min-h-screen bg-gray-50",

            // Content container with padding
            div {
                class: "pt-16",
                Outlet::<Route> {}
            }
        }
    }
}
