use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use crate::app::ContentDto;
use crate::app::ContentPage;

pub use crate::app::AppState;

#[derive(serde::Serialize)]
pub struct PageResponse {
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<ContentDto>,
    pub total_items: u32,
    pub total_pages: u32,
}

impl From<ContentPage> for PageResponse {
    fn from(p: ContentPage) -> Self {
        Self {
            page: p.page,
            page_size: p.page_size,
            items: p.items,
            total_items: p.total_items,
            total_pages: p.total_pages,
        }
    }
}

pub fn router(state: AppState) -> Router {
    super::routes::routes().with_state(state)
}

pub async fn serve(state: AppState, addr: SocketAddr) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("http server listening on {addr}");
    axum::serve(listener, router(state).layer(TraceLayer::new_for_http())).await
}
