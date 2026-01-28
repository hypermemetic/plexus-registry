use anyhow::Result;
use hub_core::plexus::DynamicHub;
use hub_transport::TransportServer;
use registry::Registry;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Registry service...");

    // Create registry activation and wrap in DynamicHub
    let registry = Registry::with_defaults()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create registry: {}", e))?;

    tracing::info!("Registry initialized successfully");

    // Wrap registry in DynamicHub for proper routing
    // Use "registry-hub" as hub namespace, "registry" is the child activation
    let hub = Arc::new(
        DynamicHub::new("registry-hub")
            .register(registry)
    );

    // Provide RPC converter using DynamicHub's method
    let rpc_converter = |arc: Arc<DynamicHub>| {
        DynamicHub::arc_into_rpc_module(arc)
            .map_err(|e| anyhow::anyhow!("Failed to create RPC module: {}", e))
    };

    // Configure and start transports
    // Default: WebSocket on 4445 (4444 is for main substrate)
    let port = std::env::var("REGISTRY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4445);

    tracing::info!("Starting WebSocket server on port {}", port);

    TransportServer::builder(hub, rpc_converter)
        .with_websocket(port)
        .build()
        .await?
        .serve()
        .await?;

    Ok(())
}
