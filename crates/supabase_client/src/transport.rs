// Send-safe HTTP transport: gloo-net on wasm, reqwest on native.
// gloo-net internals use Rc, which makes its futures !Send — unusable
// inside axum handlers on native, hence the split.

const ERR_SEND: &str = "request failed";
const ERR_TEXT: &str = "failed to read response body";

type HeaderFn = Box<dyn Fn(&str) -> Option<String> + Send>;

pub struct Response {
    pub status: u16,
    pub text: String,
    header_fn: HeaderFn,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<String> {
        (self.header_fn)(name)
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn send(
    method: &'static str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<String>,
) -> Result<Response, String> {
    use gloo_net::http::Request;
    use std::future::Future;
    use std::pin::Pin;

    type GlooSend =
        Pin<Box<dyn Future<Output = Result<gloo_net::http::Response, gloo_net::Error>>>>;

    let builder = match method {
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        "PATCH" => Request::patch(url),
        _ => Request::get(url),
    };
    let mut builder = builder;
    for (k, v) in &headers {
        builder = builder.header(k, v);
    }
    let send: GlooSend = match body {
        Some(b) => {
            let request = builder.body(b).map_err(|e| e.to_string())?;
            Box::pin(request.send())
        }
        None => Box::pin(builder.send()),
    };
    let resp = send.await.map_err(|_| ERR_SEND.to_string())?;
    let status = resp.status();
    let text = resp.text().await.map_err(|_| ERR_TEXT.to_string())?;
    Ok(Response {
        status,
        text,
        header_fn: Box::new(move |name: &str| resp.headers().get(name)),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn send(
    method: &'static str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<String>,
) -> Result<Response, String> {
    let client = reqwest::Client::new();
    let builder = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };
    let builder = {
        let mut b = builder;
        for (k, v) in &headers {
            b = b.header(k, v);
        }
        b
    };
    let builder = match body {
        Some(b) => builder.body(b),
        None => builder,
    };
    let resp = builder.send().await.map_err(|_| ERR_SEND.to_string())?;
    let status = resp.status().as_u16();
    let header_map = resp.headers().clone();
    let text = resp.text().await.map_err(|_| ERR_TEXT.to_string())?;
    Ok(Response {
        status,
        text,
        header_fn: Box::new(move |name: &str| {
            header_map
                .get(name)
                .map(|v| v.to_str().unwrap_or_default().to_owned())
        }),
    })
}
