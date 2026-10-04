mod app;
mod layout;
mod routes;

fn main() {
    ui_core::app::init_tracing();
    tracing::info!("Starting ui_wasm (all data via content_backend API)...");
    dioxus::launch(app::App);
}
