//! One-shot CLI: pull content + tags from Supabase into local pg,
//! then push queued local writes back. Runs in pg_sync mode.
//!
//! Requires DATABASE_URL, SUPABASE_URL, SUPABASE_ANON_KEY (env or .env).

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let backend = content_backend::Backend::new(
        content_backend::BackendMode::PgSync,
        content_backend::infra::supabase::config_from_env(),
    );
    match backend.pull_remote().await {
        Ok(pulled) => println!("pulled {pulled} row(s) from Supabase"),
        Err(e) => {
            eprintln!("pull failed: {e}");
            std::process::exit(1);
        }
    }
    match backend.flush_pending().await {
        Ok(pushed) => println!("pushed {pushed} queued row(s) to Supabase"),
        Err(e) => eprintln!("push failed: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    unimplemented!("sync cli is native only");
}
