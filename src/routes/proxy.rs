use std::time::Duration;

use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::config::Config;

#[derive(Deserialize)]
pub struct ProxyParams {
    url: String,
}

pub async fn proxy(params: Query<ProxyParams>) -> Response {
    let url = &params.url;
    if url.is_empty() {
        tracing::warn!("proxy request with empty url");
        return (StatusCode::BAD_REQUEST, "invalid data").into_response();
    }
    let config = Config::get();
    let parsed_url = match reqwest::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(error = %e, "failed to parse url");
            return (StatusCode::BAD_REQUEST, "invalid data").into_response();
        }
    };
    let partial_url = format!(
        "{}://{}{}",
        parsed_url.scheme(),
        parsed_url.authority(),
        parsed_url.path()
    );
    if !config.proxy.allowed_urls.contains(&partial_url) {
        tracing::warn!("proxy request with disallowed url: {}", url);
        return (StatusCode::FORBIDDEN, "invalid data").into_response();
    }
    let timeout = Duration::from_secs(config.proxy.timeout_secs);
    let client = match reqwest::Client::builder().timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to build reqwest client");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
        }
    };
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "proxy request failed for {}", url);
            return (StatusCode::BAD_GATEWAY, "bad gateway").into_response();
        }
    };
    let status = response.status();
    let incoming_headers = response.headers().clone();
    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);
    let mut builder = Response::builder().status(status);
    let headers = builder.headers_mut().unwrap();
    copy_headers(
        &incoming_headers,
        headers,
        &[
            "content-length".to_string(),
            "cache-control".to_string(),
            "expires".to_string(),
        ],
    );
    builder.body(body).unwrap().into_response()
}

fn copy_headers(
    from: &reqwest::header::HeaderMap,
    to: &mut axum::http::HeaderMap,
    headers_to_copy: &[String],
) {
    for header_name in headers_to_copy {
        if let Ok(name) = header_name.parse::<axum::http::header::HeaderName>() {
            if let Some(value) = from.get(&name) {
                to.insert(name, value.clone());
            }
        }
    }
}
