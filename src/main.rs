mod content;
mod github;
mod render;
mod routes;
mod state;

use std::io;

use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = state::AppState::from_env()?;
    let address = std::env::var("PORTFOLIO_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(&address).await?;
    info!(address = %listener.local_addr()?, "portfolio server listening");
    axum::serve(listener, app).await
}
