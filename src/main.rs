mod config;
mod routes;

use tokio::net::TcpListener;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let config = Config::get();
    let router = routes::router();
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address).await?;
    tracing::info!("server running on http://{}", address);
    axum::serve(listener, router).await?;
    Ok(())
}
