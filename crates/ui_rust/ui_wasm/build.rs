fn main() {
    dotenvy::dotenv().ok();

    println!("cargo:rerun-if-changed=.env");

    // This app always talks to the content_backend HTTP API; the API base
    // comes from BACKEND_API_URL (empty = same-origin, i.e. behind nginx).
    println!(
        "cargo:rustc-env=BACKEND_API_URL={}",
        std::env::var("BACKEND_API_URL").unwrap_or_default()
    );

    // Google OAuth client ID is env-only (build-time); no per-user setting.
    println!(
        "cargo:rustc-env=GOOGLE_OAUTH_CLIENT_ID={}",
        std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or_default()
    );
}
