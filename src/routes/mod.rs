use axum::{
    Router,
    routing::{any, get},
};

use crate::routes::pages::pages_router;

mod pages;
mod ping;
mod proxy;

pub fn router() -> Router {
    Router::new()
        .route("/api/ping", get(ping::ping))
        .route("/api/proxy", any(proxy::proxy_any))
        .fallback_service(pages_router())
}
