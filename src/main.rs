// Module declarations
mod app;
mod components;
mod contexts;
mod models;
mod pages;
mod routes;
mod services;
mod utils;

fn init_tracing() {
    use log::Level;
    console_log::init_with_level(Level::Debug).expect("Failed to initialize logger");
    tracing_wasm::set_as_global_default();
}

// Main entry point
fn main() {
    init_tracing();
    tracing::info!("Starting application...");
    dioxus::launch(app::App);
}
