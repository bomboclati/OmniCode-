pub mod http;
pub mod routes;
pub mod sync;
pub mod ws;

use crate::config::Config;
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;

pub async fn start(config: Config, port: u16) -> Result<()> {
    let app = routes::create_router(&config);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("OmniCode Web Server starting on http://{}", addr);
    println!("Web UI available at http://localhost:{}/", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
