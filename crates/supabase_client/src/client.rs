use crate::config::ClientConfig;
use crate::transport;
use serde::Serialize;
use serde::de::DeserializeOwned;

const API_KEY_HEADER: &str = "apikey";
const AUTHORIZATION_HEADER: &str = "Authorization";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const PREFER_HEADER: &str = "Prefer";
const RETURN_REPRESENTATION: &str = "return=representation";
const COUNT_EXACT: &str = "count=exact";
const BEARER_PREFIX: &str = "Bearer ";
const APPLICATION_JSON: &str = "application/json";
/// PostgREST query param + default sort: newest first. Without it, offset/
/// limit pages come back in unspecified row order.
const ORDER_PARAM: &str = "order";
const DEFAULT_ORDER: &str = "created_at.desc";
const HEADER_CONTENT_RANGE: &str = "content-range";

fn build_url(
    config: &ClientConfig,
    table: &str,
    params: &[(&str, &str)],
) -> Result<String, String> {
    let base_url = config.rest_url();

    // Build URL manually without using Url::set_query
    let mut url_string = format!("{}/{}", base_url, table);

    // Build query string manually with eq. prefix only for filter parameters
    // (not for ordering, limit, offset, select, etc.)
    if !params.is_empty() {
        let query_string: String = params
            .iter()
            .filter(|(k, v)| !k.is_empty() && !v.is_empty())
            .map(|(k, v)| {
                // Don't add eq. prefix to ordering, limit, offset, select parameters
                let no_eq_prefix = matches!(*k, "order" | "limit" | "offset" | "select");
                if no_eq_prefix {
                    format!("{}={}", encode(k), encode(v))
                } else {
                    format!("{}=eq.{}", encode(k), encode(v))
                }
            })
            .collect::<Vec<_>>()
            .join("&");

        if !query_string.is_empty() {
            url_string = format!("{}?{}", url_string, query_string);
        }
    }
    Ok(url_string)
}

fn encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

pub fn build_headers(
    config: &ClientConfig,
    prefer_return: bool,
    jwt_token: Option<&str>,
    count: bool,
) -> Result<Vec<(String, String)>, String> {
    let credential = get_credential(config, jwt_token);

    let mut headers = vec![
        (API_KEY_HEADER.to_string(), config.anon_key.clone()),
        (
            AUTHORIZATION_HEADER.to_string(),
            format!("{}{}", BEARER_PREFIX, credential),
        ),
        (
            CONTENT_TYPE_HEADER.to_string(),
            APPLICATION_JSON.to_string(),
        ),
    ];

    let prefer = match (prefer_return, count) {
        (true, true) => Some(format!("{},{}", RETURN_REPRESENTATION, COUNT_EXACT)),
        (true, false) => Some(RETURN_REPRESENTATION.to_string()),
        (false, true) => Some(COUNT_EXACT.to_string()),
        (false, false) => None,
    };
    if let Some(p) = prefer {
        headers.push((PREFER_HEADER.to_string(), p));
    }

    Ok(headers)
}

fn get_credential(config: &ClientConfig, jwt_token_param: Option<&str>) -> String {
    if let Some(ref service_role_key) = config.service_role_key {
        return service_role_key.clone();
    }
    if let Some(token) = jwt_token_param {
        return token.to_string();
    }
    if let Some(ref jwt_token) = config.jwt_token {
        return jwt_token.clone();
    }

    config.anon_key.clone()
}

async fn send(
    method: &'static str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<String>,
) -> Result<transport::Response, String> {
    tracing::debug!("{method} {url}");
    let response = transport::send(method, url, headers, body).await?;
    if response.status >= 400 {
        return Err(format!(
            "HTTP {} from Supabase: {}",
            response.status,
            response.text()
        ));
    }
    Ok(response)
}

pub async fn get<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    params: &[(&str, &str)],
) -> Result<Vec<T>, String> {
    let url = build_url(config, table, params)?;
    let headers = build_headers(config, false, None, false)?;

    let response_text = send("GET", &url, headers, None).await?.text().to_owned();

    serde_json::from_str::<Vec<T>>(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn get_by_id<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    id: i32,
) -> Result<Option<T>, String> {
    let items = get(config, table, &[("id", &id.to_string())]).await?;
    Ok(items.into_iter().next())
}

pub async fn get_by<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    column: &str,
    value: &str,
) -> Result<Vec<T>, String> {
    get(config, table, &[(column, value)]).await
}

pub async fn create<T: Serialize, R: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    data: &T,
) -> Result<Vec<R>, String> {
    let url = build_url(config, table, &[])?;
    let headers = build_headers(config, true, None, false)?;
    let body =
        serde_json::to_string(data).map_err(|e| format!("Failed to serialize request: {}", e))?;

    let response_text = send("POST", &url, headers, Some(body))
        .await?
        .text()
        .to_owned();

    serde_json::from_str::<Vec<R>>(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn update<T: Serialize, R: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    id: i32,
    data: &T,
) -> Result<Vec<R>, String> {
    let url = build_url(config, table, &[("id", &id.to_string())])?;
    let headers = build_headers(config, true, None, false)?;
    let body =
        serde_json::to_string(data).map_err(|e| format!("Failed to serialize request: {}", e))?;

    let response_text = send("PATCH", &url, headers, Some(body))
        .await?
        .text()
        .to_owned();

    serde_json::from_str::<Vec<R>>(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn delete(config: &ClientConfig, table: &str, id: i32) -> Result<(), String> {
    let url = build_url(config, table, &[("id", &id.to_string())])?;
    let headers = build_headers(config, false, None, false)?;

    send("DELETE", &url, headers, None).await?;

    Ok(())
}

/// Get records where a column value is in a list of values using Supabase's in filter
/// Example usage: get_by_in::<Content>(config, "content", "id", &[1, 2, 3]).await?
pub async fn get_by_in<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    column: &str,
    values: &[i32],
) -> Result<Vec<T>, String> {
    let values_str = values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let url = format!(
        "{}/{}?{}=in.({})",
        config.rest_url(),
        table,
        encode(column),
        values_str
    );
    let headers = build_headers(config, false, None, false)?;

    let response_text = send("GET", &url, headers, None).await?.text().to_owned();

    serde_json::from_str::<Vec<T>>(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get records with pagination support using offset and limit
/// Example usage: get_paginated::<Content>(config, "content", &[("status", "published")], 0, 10).await?
pub async fn get_paginated<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    filters: &[(&str, &str)],
    offset: u32,
    limit: u32,
) -> Result<Vec<T>, String> {
    let mut params: Vec<(&str, String)> =
        filters.iter().map(|(k, v)| (*k, v.to_string())).collect();

    params.push(("offset", offset.to_string()));
    params.push(("limit", limit.to_string()));
    params.push((ORDER_PARAM, DEFAULT_ORDER.to_string()));

    let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let url = build_url(config, table, &params_ref)?;
    let headers = build_headers(config, false, None, false)?;

    let response_text = send("GET", &url, headers, None).await?.text().to_owned();

    serde_json::from_str::<Vec<T>>(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get records with pagination support and total count in a single request
/// Uses Supabase's `Prefer: count=exact` header to get both data and count
/// Returns a tuple of (data, total_count)
/// Example usage: get_paginated_with_count::<Content>(config, "content", &[("status", "published")], 0, 10).await?
pub async fn get_paginated_with_count<T: DeserializeOwned>(
    config: &ClientConfig,
    table: &str,
    filters: &[(&str, &str)],
    offset: u32,
    limit: u32,
) -> Result<(Vec<T>, u32), String> {
    let mut params: Vec<(&str, String)> =
        filters.iter().map(|(k, v)| (*k, v.to_string())).collect();

    params.push(("offset", offset.to_string()));
    params.push(("limit", limit.to_string()));
    params.push((ORDER_PARAM, DEFAULT_ORDER.to_string()));

    let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let url = build_url(config, table, &params_ref)?;
    let headers = build_headers(config, false, None, true)?;

    let response = send("GET", &url, headers, None).await?;

    let data = serde_json::from_str::<Vec<T>>(response.text())
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content_range = response
        .header(HEADER_CONTENT_RANGE)
        .ok_or_else(|| "Content-Range header not found".to_string())?;

    let total_count = parse_content_range(&content_range)?;
    Ok((data, total_count))
}

/// Count the total number of records in a table, optionally with filters
/// Example usage: count(config, "content", &[("status", "published")]).await?
pub async fn count(
    config: &ClientConfig,
    table: &str,
    filters: &[(&str, &str)],
) -> Result<u32, String> {
    let params_ref: Vec<(&str, &str)> = filters.iter().map(|(k, v)| (*k, *v)).collect();

    let url = build_url(config, table, &params_ref)?;
    let headers = build_headers(config, false, None, true)?;

    let response = send("GET", &url, headers, None).await?;

    let content_range = response
        .header(HEADER_CONTENT_RANGE)
        .ok_or_else(|| "Content-Range header not found".to_string())?;

    parse_content_range(&content_range)
}

fn parse_content_range(content_range: &str) -> Result<u32, String> {
    let count_str = content_range
        .split('/')
        .nth(1)
        .ok_or_else(|| "Invalid Content-Range format".to_string())?;

    count_str
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse count: {}", e))
}
