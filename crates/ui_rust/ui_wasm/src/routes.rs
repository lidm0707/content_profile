use dioxus::prelude::*;
use ui_core::pages::{ContentEdit, ContentList, Dashboard, Home, Login, TagsEdit, TagsList};

use crate::layout::AppBar;
use ui_core::pages::Settings;

/// Routes for the backend-connected app. Mirrors the ui_core routes
/// (same paths, so reused pages can link between them) plus `/settings`.
#[derive(Debug, Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[layout(AppBar)]
    #[route("/")]
    Home {},

    #[route("/login")]
    Login {},

    #[route("/settings")]
    Settings {},

    #[route("/dashboard")]
    Dashboard {},

    #[route("/content/edit/:id")]
    ContentEdit { id: i32 },

    #[route("/tags/edit/:id")]
    TagsEdit { id: i32 },

    #[route("/tags")]
    TagsList {},

    #[route("/content/list/:tag")]
    ContentList { tag: String },
}
