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

    // Load saved session
    let session_signal = use_signal(|| {
        if let Ok(Some(session)) = UserContext::load_saved_session() {
            let now = chrono::Utc::now().timestamp();
            if now < session.expires_at {
                tracing::info!("Loaded valid session from storage");
                Some(session)
            } else {
                tracing::warn!("Saved session expired, clearing");
                let _ = UserContext::clear_saved_session();
                None
            }
        } else {
            None
        }
    });

    // Derive JWT token from session signal - this will update when session changes
    let jwt_token = use_memo(move || {
        session_signal
            .read()
            .as_ref()
            .map(|s| s.access_token.clone())
    });

    // Create reactive config that updates when JWT token changes
    let config_signal = use_memo(move || {
        let mode = env!("APP_MODE");
        let supabase_url = env!("SUPABASE_URL");
        let supabase_anon_key = env!("SUPABASE_ANON_KEY");
        let token = jwt_token.read().clone();

        tracing::debug!("Creating config - JWT token present: {}", token.is_some());
        Config::new(mode, supabase_url, supabase_anon_key, token)
    });

    // Provide signals as contexts so components can access them
    use_context_provider(move || session_signal);
    use_context_provider(move || jwt_token);
    use_context_provider(move || config_signal);

    // Create UserContext - this doesn't need to update with config since it uses AuthService directly
    let user_context = UserContext::new(Some(config_signal().clone()));
    use_context_provider(move || user_context);

    // Create ContentContext with initial config
    let content_context = ContentContext::new(Some(config_signal().clone()));
    let mut content_context = use_signal(|| content_context);

    // Watch for JWT token changes and update ContentContext
    {
        let mut content_context = content_context.clone();
        use_effect(move || {
            let token = jwt_token.read().clone();
            content_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || content_context.read().clone());

    // Create TagContext with initial config
    let tag_context = TagContext::new(Some(config_signal().clone()));
    let mut tag_context = use_signal(|| tag_context);

    // Watch for JWT token changes and update TagContext
    {
        let mut tag_context = tag_context.clone();
        use_effect(move || {
            let token = jwt_token.read().clone();
            tag_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || tag_context.read().clone());

    // Create ContentTagsContext with initial config
    let content_tags_context = ContentTagsContext::new(Some(config_signal().clone()));
    let mut content_tags_context = use_signal(|| content_tags_context);

    // Watch for JWT token changes and update ContentTagsContext
    {
        let mut content_tags_context = content_tags_context.clone();
        use_effect(move || {
            let token = jwt_token.read().clone();
            content_tags_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || content_tags_context.read().clone());

    rsx! {
        // Document head elements
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: TAILWIND_CSS }

        Router::<crate::routes::Route> {}
    }
}
