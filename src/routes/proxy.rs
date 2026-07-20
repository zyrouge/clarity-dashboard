use std::time::Duration;

use axum::extract::Query;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::config::{Config, ProxyAllowedUrlConfig};

#[derive(Deserialize)]
pub struct ProxyParams {
    url: String,
}

pub async fn proxy_any(
    method: Method,
    params: Query<ProxyParams>,
    body: axum::body::Body,
) -> Response {
    let proxy_url = &params.url;
    if proxy_url.is_empty() {
        tracing::warn!("proxy request with empty url");
        return (StatusCode::BAD_REQUEST, "invalid data").into_response();
    }
    let Some(allowed_url_config) = get_allowed_url_config(&method, &proxy_url) else {
        tracing::warn!("proxy request with disallowed url: {}", proxy_url);
        return (StatusCode::FORBIDDEN, "invalid data").into_response();
    };
    let proxy_timeout = Duration::from_secs(allowed_url_config.timeout_secs);
    let proxy_client_builder = reqwest::Client::builder().timeout(proxy_timeout);
    let proxy_client = match proxy_client_builder.build() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to build reqwest client");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response();
        }
    };
    let proxy_request = proxy_client
        .request(method, proxy_url)
        .body(reqwest::Body::wrap_stream(body.into_data_stream()));
    let proxy_response = match proxy_request.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "proxy request failed for {}", proxy_url);
            return (StatusCode::BAD_GATEWAY, "bad gateway").into_response();
        }
    };
    let proxy_response_status = proxy_response.status();
    let proxy_response_headers = proxy_response.headers().clone();
    let proxy_response_stream = proxy_response.bytes_stream();
    let response_body = axum::body::Body::from_stream(proxy_response_stream);
    let mut response_builder = Response::builder().status(proxy_response_status);
    let response_headers = response_builder.headers_mut().unwrap();
    let response_copyable_headers = [
        "content-type".to_string(),
        "content-length".to_string(),
        "cache-control".to_string(),
        "expires".to_string(),
    ];
    for x in response_copyable_headers {
        if let Ok(name) = x.parse::<axum::http::header::HeaderName>() {
            if let Some(value) = proxy_response_headers.get(&name) {
                response_headers.insert(name, value.clone());
            }
        }
    }
    response_builder
        .body(response_body)
        .unwrap()
        .into_response()
}

fn get_allowed_url_config(method: &Method, url: &str) -> Option<&'static ProxyAllowedUrlConfig> {
    let config = Config::get();
    let parsed_proxy_url = reqwest::Url::parse(url).ok()?;
    let partial_proxy_url = format!(
        "{}://{}{}",
        parsed_proxy_url.scheme(),
        parsed_proxy_url.authority(),
        parsed_proxy_url.path()
    );
    config
        .proxy
        .allowed_urls
        .get(&partial_proxy_url)
        .take_if(|x| x.methods.contains(method))
}
