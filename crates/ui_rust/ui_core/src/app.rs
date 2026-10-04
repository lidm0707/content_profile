use content_sdk::utils::config::{AppMode, Config};
use dioxus::prelude::*;

use content_sdk::contexts::{ContentContext, ContentTagsContext, TagContext, UserContext};
use content_sdk::models::Session;

/// Per-user settings key for the Google Drive target folder.
pub const KEY_GOOGLE_FOLDER_ID: &str = "google_drive_folder_id";

/// Everything the shell needs to build its [`Config`]. Supplied by the
/// embedder (the lib bakes no build-time env).
#[derive(Clone, PartialEq, Default)]
pub struct AppSettings {
    pub mode: AppMode,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub google_oauth_client_id: String,
    pub backend_api_url: String,
}

/// Interface for an embeddable UI shell. Implementors choose the data-source
/// settings and the router to render; ui_core provides the shared context
/// setup and document head via [`UiApp`].
pub trait Ui {
    /// Data-source and API settings for the config.
    fn settings(&self) -> AppSettings;
    /// The router element for this app's route enum.
    fn router(&self) -> Element;
}

/// Initializes wasm logging/panic hooks. Called by the binary entry point.
pub fn init_tracing() {
    use log::Level;
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).expect("Failed to initialize logger");
}

/// Public interface for embedders: sets up session, JWT, config and all
/// data contexts (User/Content/Tag/ContentTags) as shared contexts.
/// Must be called from within a component's render (it installs hooks).
pub fn init_app_contexts(settings: AppSettings) {
    let content_refresh_count = use_signal(|| 0u64);
    use_context_provider(move || content_refresh_count);

    // Session starts empty and is restored asynchronously (see the restore
    // effect below): backend mode probes the HttpOnly session cookie —
    // tokens live in pg — other modes fall back to localStorage.
    let mut session_signal = use_signal(|| Option::<Session>::None);
    // True once the restore probe has completed; guards must not redirect
    // to Login before this (the probe is async, so None is "unknown").
    let mut session_checked = use_signal(|| false);

    // Derive JWT token from session signal - this will update when session changes
    let jwt_token = use_memo(move || {
        session_signal
            .read()
            .as_ref()
            .map(|s| s.access_token.clone())
            .filter(|t| !t.is_empty())
    });

    // Create reactive config that updates when JWT token changes
    let config_signal = use_memo(move || {
        let token = jwt_token.read().clone();

        tracing::debug!("Creating config - JWT token present: {}", token.is_some());
        Config::new(
            settings.mode.as_str(),
            &settings.supabase_url,
            &settings.supabase_anon_key,
            token,
            if settings.google_oauth_client_id.is_empty() {
                None
            } else {
                Some(settings.google_oauth_client_id.clone())
            },
        )
        .with_backend_api_url(if settings.backend_api_url.is_empty() {
            None
        } else {
            Some(settings.backend_api_url.clone())
        })
    });

    // Provide signals as contexts so components can access them
    use_context_provider(move || session_signal);
    use_context_provider(move || session_checked);
    use_context_provider(move || jwt_token);
    use_context_provider(move || config_signal);

    // Create UserContext and watch for JWT token changes so backend
    // requests always carry the current session's Bearer token
    let user_context = UserContext::new(Some(config_signal().clone()));
    let user_context_signal = use_signal(|| user_context);
    let mut user_context = user_context_signal;
    use_effect(move || {
        let token = jwt_token.read().clone();
        user_context.write().update_jwt_token(token);
    });
    use_context_provider(move || user_context.read().clone());

    // Restore session after user_context_signal exists: backend mode probes
    // the HttpOnly session cookie (tokens live in pg), other modes fall back
    // to the localStorage session.
    {
        let mode = settings.mode;
        use_effect(move || {
            if session_signal.read().is_some() || *session_checked.read() {
                return;
            }
            spawn(async move {
                let mut restored = None;
                if mode == AppMode::Backend {
                    let ctx = user_context_signal.read().clone();
                    restored = ctx.restore_cookie_session().await;
                }
                // Legacy/localStorage fallback (supabase-direct mode and
                // pre-cookie sessions).
                if restored.is_none()
                    && let Ok(Some(s)) = UserContext::load_saved_session()
                {
                    restored = Some(s);
                }
                match restored {
                    Some(s) => session_signal.set(Some(s)),
                    None => session_checked.set(true),
                }
            });
        });
    }
    use_context_provider(move || user_context.read().clone());

    // Refresh an expired/near-expiry session instead of forcing a logout;
    // only clears it when the refresh itself fails.
    use_effect(move || {
        const EXPIRY_SKEW_SECS: i64 = 30;
        let Some(session) = session_signal() else {
            return;
        };
        let now = chrono::Utc::now().timestamp();
        if now < session.expires_at - EXPIRY_SKEW_SECS {
            return;
        }
        spawn(async move {
            let ctx = user_context.read().clone();
            match ctx.restore_saved_session().await {
                Some(fresh) => session_signal.set(Some(fresh)),
                None => session_signal.set(None),
            }
        });
    });

    // Create ContentContext with initial config
    let content_context = ContentContext::new(Some(config_signal().clone()));
    let content_context = use_signal(|| content_context);

    // Watch for JWT token changes and update ContentContext
    {
        let mut content_context = content_context;
        use_effect(move || {
            let token = jwt_token.read().clone();
            content_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || content_context.read().clone());

    // Create TagContext with initial config
    let tag_context = TagContext::new(Some(config_signal().clone()));
    let tag_context = use_signal(|| tag_context);

    // Watch for JWT token changes and update TagContext
    {
        let mut tag_context = tag_context;
        use_effect(move || {
            let token = jwt_token.read().clone();
            tag_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || tag_context.read().clone());

    // Create ContentTagsContext with initial config
    let content_tags_context = ContentTagsContext::new(Some(config_signal().clone()));
    let content_tags_context = use_signal(|| content_tags_context);

    // Watch for JWT token changes and update ContentTagsContext
    {
        let mut content_tags_context = content_tags_context;
        use_effect(move || {
            let token = jwt_token.read().clone();
            content_tags_context.write().update_jwt_token(token);
        });
    }

    use_context_provider(move || content_tags_context.read().clone());
}

/// Generic root shell: implements the app frame for any [`Ui`] —
/// shared context setup, then the implementor's router. Platform shell
/// (document head, stylesheets) belongs to the embedder.
#[component]
pub fn UiApp<U: Ui + Clone + PartialEq + 'static>(ui: U) -> Element {
    init_app_contexts(ui.settings());

    rsx! {
        {ui.router()}
    }
}
