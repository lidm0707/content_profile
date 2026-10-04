#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let cfg = content_backend::config::Config::from_env();
    let state = content_backend::app::AppState::from_config(&cfg)
        .await
        .unwrap_or_else(|e| {
            eprintln!("startup error: {e}");
            std::process::exit(1);
        });
    if let Err(e) = content_backend::infra::http::serve(state, cfg.http_addr).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    unimplemented!("http server is native only");
}
