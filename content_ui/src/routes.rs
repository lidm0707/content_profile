use dioxus::prelude::*;

use crate::components::MainLayout;
// Import page components for use in routes
use crate::pages::ContentEdit;
use crate::pages::ContentList;
use crate::pages::Dashboard;
use crate::pages::Home;
use crate::pages::Login;
use crate::pages::TagsEdit;
use crate::pages::TagsList;

/// All application routes with navigation structure
#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(MainLayout)]
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

    // Tags edit page - handles both creating (id=0) and editing (id > 0) tags
    #[route("/tags/edit/:id")]
    TagsEdit { id: i32 },

    // Tags list page - displays all tags with management options
    #[route("/tags")]
    TagsList {},

    // Content list page - displays content filtered by tag (empty string = all content)
    #[route("/content/list/:tag")]
    ContentList { tag: String },
}
