use content_sdk::utils::config::Config;
use dioxus::prelude::*;

use content_sdk::contexts::{ContentContext, ContentTagsContext, TagContext, UserContext};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Root application component that sets up the router and global providers
#[component]
pub fn App() -> Element {
    let content_refresh_count = use_signal(|| 0u64);
    use_context_provider(move || content_refresh_count);

    // Load saved session and extract JWT token
    let jwt_token = use_signal(|| {
        if let Ok(Some(session)) = UserContext::load_saved_session() {
            let now = chrono::Utc::now().timestamp();
            if now < session.expires_at {
                Some(session.access_token)
            } else {
                let _ = UserContext::clear_saved_session();
                None
            }
        } else {
            None
        }
    });

    let mode = env!("APP_MODE");
    let supabase_url = env!("SUPABASE_URL");
    let supabase_anon_key = env!("SUPABASE_ANON_KEY");
    let config = Config::new(
        mode,
        supabase_url,
        supabase_anon_key,
        jwt_token.read().clone(),
    );

    use_context_provider(move || jwt_token);

    // Create UserContext
    let user_context = UserContext::new(Some(config.clone()));
    use_context_provider(move || user_context.clone());

    // Create ContentContext
    let content_context = ContentContext::new(Some(config.clone()));
    use_context_provider(move || content_context.clone());

    // Create TagContext
    let tag_context = TagContext::new(Some(config.clone()));
    use_context_provider(move || tag_context.clone());

    // Create ContentTagsContext
    let content_tags_context = ContentTagsContext::new(Some(config.clone()));
    use_context_provider(move || content_tags_context.clone());

    rsx! {
        // Document head elements
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: TAILWIND_CSS }


            Router::<crate::routes::Route> {}

    }
}
