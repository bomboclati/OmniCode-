use crate::config::Config;
use crate::server;
use anyhow::Result;
use std::sync::Arc;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

pub async fn run(config: Arc<Config>, port: u16) -> Result<()> {
    let config_clone = (*config).clone();
    tokio::spawn(async move {
        if let Err(e) = server::start(config_clone, port).await {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let event_loop = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("OmniCode")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new()
        .with_url(&format!("http://127.0.0.1:{}", port))
        .build(&window)?;

    event_loop.run(move |_event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;
    });
    Ok(())
}
