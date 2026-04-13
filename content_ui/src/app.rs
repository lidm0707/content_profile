use content_sdk::utils::config::Config;
use dioxus::prelude::*;

use crate::contexts::{ContentContext, TagContext, UserContext};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Root application component that sets up the router and global providers
#[component]
pub fn App() -> Element {
    let content_refresh_count = use_signal(|| 0u64);
    let mode = env!("APP_MODE");
    let supabase_url = env!("SUPABASE_URL");
    let supabase_anon_key = env!("SUPABASE_ANON_KEY");
    let config = Config::new(mode, supabase_url, supabase_anon_key);

    use_context_provider(move || content_refresh_count);

    // Create UserContext
    let user_context = UserContext::new(Some(config.clone()));
    use_context_provider(move || user_context.clone());

    // Create ContentContext
    let content_context = ContentContext::new(Some(config.clone()));
    use_context_provider(move || content_context.clone());

    // Create TagContext
    let tag_context = TagContext::new(Some(config.clone()));
    use_context_provider(move || tag_context.clone());

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
        document::Stylesheet { href: TAILWIND_CSS }


            Router::<crate::routes::Route> {}

    }
}
