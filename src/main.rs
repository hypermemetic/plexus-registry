use anyhow::Result;
use clap::Parser;
use plexus_core::plexus::DynamicHub;
use plexus_transport::TransportServer;
use registry::{Registry, RegistryStorageConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Backend discovery and registration service for Plexus hubs
#[derive(Parser, Debug)]
#[command(name = "registry")]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on for WebSocket connections
    #[arg(short, long, default_value = "4445", env = "REGISTRY_PORT")]
    port: u16,

    /// Host address to bind to
    #[arg(long, default_value = "0.0.0.0", env = "REGISTRY_HOST")]
    host: String,

    /// Path to SQLite database file
    #[arg(long, env = "REGISTRY_DB_PATH")]
    db_path: Option<PathBuf>,

    /// Path to backends TOML config file
    #[arg(short, long, env = "REGISTRY_CONFIG")]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,

    /// Hub namespace for routing
    #[arg(long, default_value = "registry-hub")]
    hub_namespace: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&args.log_level)),
        )
        .init();

    tracing::info!("Starting Registry service v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("  WebSocket: ws://{}:{}", args.host, args.port);

    // Build storage config
    let storage_config = if args.db_path.is_some() || args.config.is_some() {
        let default = RegistryStorageConfig::default();
        RegistryStorageConfig {
            db_path: args.db_path.unwrap_or(default.db_path),
            config_path: args.config.or(default.config_path),
        }
    } else {
        RegistryStorageConfig::default()
    };

    tracing::info!("  Database: {}", storage_config.db_path.display());
    if let Some(ref config_path) = storage_config.config_path {
        tracing::info!("  Config: {}", config_path.display());
    }

    // Create registry activation and wrap in DynamicHub
    let registry = Registry::new(storage_config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create registry: {}", e))?;

    tracing::info!("Registry initialized successfully");

    // Wrap registry in DynamicHub for proper routing
    let hub = Arc::new(
        DynamicHub::new(&args.hub_namespace)
            .register(registry)
    );

    // Provide RPC converter using DynamicHub's method
    let rpc_converter = |arc: Arc<DynamicHub>| {
        DynamicHub::arc_into_rpc_module(arc)
            .map_err(|e| anyhow::anyhow!("Failed to create RPC module: {}", e))
    };

    tracing::info!("Starting WebSocket server...");

    TransportServer::builder(hub, rpc_converter)
        .with_websocket(args.port)
        .build()
        .await?
        .serve()
        .await?;

    Ok(())
}
