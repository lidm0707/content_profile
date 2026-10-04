use content_sdk::utils::config::AppMode;
use dioxus::prelude::*;

use ui_core::app::{AppSettings, Ui, UiApp};

use crate::routes::AppRoute;

const ENV_BACKEND_API_URL: &str = env!("BACKEND_API_URL");
const ENV_GOOGLE_OAUTH_CLIENT_ID: &str = env!("GOOGLE_OAUTH_CLIENT_ID");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const THEME_CSS: Asset = asset!("/assets/theme.css");
const MARKDOWN_CSS: Asset = asset!("/assets/markdown.css");
const FONTS_URL: &str = "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=Noto+Sans+JP:wght@400;500;700&family=Noto+Serif+JP:wght@600;700;800&display=swap";

/// Web document head for the wasm shell: favicon, fonts and stylesheets.
#[component]
pub fn DocumentHead() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com",
        }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous",
        }
        document::Link {
            rel: "stylesheet",
            href: FONTS_URL,
        }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: THEME_CSS }
        document::Stylesheet { href: MARKDOWN_CSS }
    }
}

/// Backend-connected UI: all data via content_backend's HTTP API,
/// with the extra /settings route on top of ui_core's pages.
#[derive(Clone, PartialEq)]
pub struct BackendUi;

impl Ui for BackendUi {
    fn settings(&self) -> AppSettings {
        AppSettings {
            mode: AppMode::Backend,
            backend_api_url: ENV_BACKEND_API_URL.to_string(),
            google_oauth_client_id: ENV_GOOGLE_OAUTH_CLIENT_ID.to_string(),
            ..Default::default()
        }
    }

    fn router(&self) -> Element {
        rsx! {
            Router::<AppRoute> {}
        }
    }
}

#[component]
pub fn App() -> Element {
    rsx! {
        DocumentHead {}
        UiApp { ui: BackendUi }
    }
}
