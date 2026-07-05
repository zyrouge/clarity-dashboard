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
    if !config.proxy.allowed_urls.contains(url) {
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
    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);
    Response::builder().status(status).body(body).unwrap()
}
