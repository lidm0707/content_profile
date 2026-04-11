use dioxus::prelude::*;

use crate::contexts::UserContext;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

/// Root application component that sets up the router and global providers
#[component]
pub fn App() -> Element {
    let content_refresh_count = use_signal(|| 0u64);

    use_context_provider(move || content_refresh_count);

    // Create UserContext
    let user_context = UserContext::new();
    use_context_provider(move || user_context.clone());

    // Load saved session and create session signal
    let session_signal = use_signal(|| {
        if let Ok(Some(session)) = UserContext::load_saved_session() {
            let now = chrono::Utc::now().timestamp();
            if now < session.expires_at {
                Some(session)
            } else {
                let _ = UserContext::clear_saved_session();
                None
            }
        } else {
            None
        }
    });

    use_context_provider(move || session_signal);

    rsx! {
        // Document head elements
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        // Main application router
        Router::<crate::routes::Route> {}
    }
}
