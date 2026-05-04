use async_trait::async_trait;
use log::info;
use pingora_core::prelude::*;
use pingora_core::server::Server;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_http::RequestHeader;
use pingora_proxy::{ProxyHttp, Session, http_proxy_service};
use std::env;

const PROXY_PORT: &str = "0.0.0.0:6190";
const UI_UPSTREAM: (&str, u16) = ("content_ui", 80);

struct SupabasePeer {
    host: String,
    port: u16,
}

impl SupabasePeer {
    fn from_env() -> Self {
        let url = env::var("SUPABASE_URL").expect("SUPABASE_URL env var required");

        let (host, port) = Self::parse_supabase_url(&url);
        info!("supabase cloud upstream resolved to {host}:{port}");

        Self { host, port }
    }

    fn parse_supabase_url(url: &str) -> (String, u16) {
        let stripped = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        let host = stripped.trim_end_matches('/').to_string();
        let port = if url.starts_with("https://") { 443 } else { 80 };
        (host, port)
    }
}

pub struct ContentProxy {
    supabase: SupabasePeer,
}

impl ContentProxy {
    fn new() -> Self {
        Self {
            supabase: SupabasePeer::from_env(),
        }
    }
}

#[async_trait]
impl ProxyHttp for ContentProxy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        if self.is_api_path(session) {
            info!("routing to supabase cloud");
            let use_tls = self.supabase.port == 443;
            let peer = Box::new(HttpPeer::new(
                (self.supabase.host.as_str(), self.supabase.port),
                use_tls,
                self.supabase.host.clone(),
            ));
            Ok(peer)
        } else {
            info!("routing to content_ui upstream");
            Ok(Box::new(HttpPeer::new(UI_UPSTREAM, false, String::new())))
        }
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        let path = upstream.uri.path();
        if path.starts_with("/rest/") || path.starts_with("/auth/") {
            upstream.insert_header("Host", self.supabase.host.clone())?;
        }
        Ok(())
    }
}

impl ContentProxy {
    fn is_api_path(&self, session: &Session) -> bool {
        let uri = session.req_header().uri.path();
        uri.starts_with("/rest/") || uri.starts_with("/auth/")
    }
}

fn main() {
    env_logger::init();

    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();

    let mut lb = http_proxy_service(&server.configuration, ContentProxy::new());
    lb.add_tcp(PROXY_PORT);

    server.add_service(lb);

    info!("content_proxy listening on {PROXY_PORT}");
    server.run_forever();
}
