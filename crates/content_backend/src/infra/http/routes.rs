use axum::routing::{delete, get, post};

use super::api;

pub fn routes() -> axum::Router<super::server::AppState> {
    axum::Router::new()
        .route("/api/auth/signup", post(api::signup))
        .route("/api/auth/login", post(api::login))
        .route("/api/auth/supabase", post(api::supabase_exchange))
        .route("/api/auth/logout", post(api::logout))
        .route("/api/auth/session", get(api::get_session))
        .route(
            "/api/settings",
            get(api::get_settings).put(api::put_settings),
        )
        .route(
            "/api/content",
            get(api::list_content).post(api::create_content),
        )
        .route("/api/content/page", get(api::content_page))
        .route("/api/content/by-ids", get(api::content_by_ids))
        .route("/api/content/{id}/tags", get(api::tags_for_content))
        .route(
            "/api/content/{id}/content-tags",
            get(api::content_tags_for_content),
        )
        .route("/api/content/slug/{slug}", get(api::get_content_by_slug))
        .route(
            "/api/content/{id}",
            get(api::get_content)
                .put(api::update_content)
                .delete(api::delete_content),
        )
        .route("/api/tags", get(api::list_tags).post(api::create_tag))
        .route("/api/tags/by-ids", get(api::tags_by_ids))
        .route(
            "/api/tags/{id}/content-tags",
            get(api::content_tags_for_tag),
        )
        .route(
            "/api/tags/{id}",
            get(api::get_tag)
                .put(api::update_tag)
                .delete(api::delete_tag),
        )
        .route("/api/content-tags", post(api::add_content_tag))
        .route(
            "/api/content-tags/content/{content_id}/tag/{tag_id}",
            delete(api::remove_content_tag_pair),
        )
        .route("/api/content-tags/{id}", delete(api::remove_content_tag))
        .route("/api/sync/flush", post(api::flush_sync))
        .route("/api/sync/pull", post(api::sync_pull))
        .route("/api/oauth/google/url", get(api::google_auth_url))
        .route("/api/oauth/google/callback", get(api::google_callback))
        .route("/api/oauth/google/token", get(api::google_access_token))
        .route("/api/oauth/google/status", get(api::google_status))
}
