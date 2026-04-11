use dioxus::prelude::*;

// Import page components for use in routes
use crate::pages::ContentEdit;
use crate::pages::Dashboard;
use crate::pages::Home;
use crate::pages::Login;

/// All application routes with navigation structure
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
#[layout(crate::components::MainLayout)]
pub enum Route {
    // Home page - landing page
    #[route("/")]
    Home {},

    // Login page - authentication
    #[route("/login")]
    Login {},

    // Dashboard page - content management interface
    #[route("/dashboard")]
    Dashboard {},

    // Content edit page - handles both creating (id=0) and editing (id > 0) content
    #[route("/content/edit/:id")]
    ContentEdit { id: i32 },
}
