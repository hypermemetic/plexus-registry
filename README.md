# Registry

Backend discovery and registration service for Plexus hubs.

## Overview

Registry is a standalone service that enables dynamic discovery of Plexus backend instances. It stores backend connection information and provides a simple API for registration, discovery, and health checks.

**Key features:**
- SQLite storage with TOML configuration
- Auto-registration of local Plexus instance
- Manual registration via RPC
- Health check timestamps
- Config file hot-reloading

## Quick Start

### Running the Registry

```bash
cd registry
cargo run
```

By default, the registry starts on port 4445 (4444 is reserved for the main Plexus instance).

### Configuration

The registry looks for configuration at `~/.config/plexus/backends.toml`:

```toml
[[backend]]
name = "plexus"
host = "127.0.0.1"
port = 4444
protocol = "ws"
description = "Local Plexus instance"

[[backend]]
name = "prod"
host = "plexus.example.com"
port = 4444
protocol = "wss"
description = "Production Plexus hub"
```

### Environment Variables

- `REGISTRY_PORT` - WebSocket server port (default: 4445)
- `RUST_LOG` - Logging level (default: info)

## API Methods

All methods are in the `registry` namespace:

### `register`

Register a new backend.

**Parameters:**
- `name` (string, required) - Unique backend name
- `host` (string, required) - Host address
- `port` (number, required) - Port number
- `protocol` (string, optional) - Protocol ("ws" or "wss", default: "ws")
- `description` (string, optional) - Human-readable description
- `namespace` (string, optional) - Routing namespace

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.register",
  "params": {
    "name": "dev",
    "host": "192.168.1.100",
    "port": 4444,
    "protocol": "ws",
    "description": "Development server"
  }
}
```

### `list`

List all registered backends.

**Parameters:**
- `active_only` (boolean, optional) - Filter to active backends only (default: true)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.list",
  "params": {
    "active_only": true
  }
}
```

### `get`

Get a specific backend by name.

**Parameters:**
- `name` (string, required) - Backend name

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.get",
  "params": {
    "name": "plexus"
  }
}
```

### `update`

Update an existing backend.

**Parameters:**
- `name` (string, required) - Backend name to update
- `host` (string, optional) - New host address
- `port` (number, optional) - New port number
- `protocol` (string, optional) - New protocol
- `description` (string, optional) - New description
- `namespace` (string, optional) - New namespace

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.update",
  "params": {
    "name": "dev",
    "host": "192.168.1.101"
  }
}
```

### `delete`

Remove a backend from the registry.

**Parameters:**
- `name` (string, required) - Backend name to delete

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.delete",
  "params": {
    "name": "old-server"
  }
}
```

### `ping`

Update the health check timestamp for a backend.

**Parameters:**
- `name` (string, required) - Backend name

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.ping",
  "params": {
    "name": "plexus"
  }
}
```

### `reload`

Reload backends from the configuration file.

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "registry.reload",
  "params": {}
}
```

## Testing with websocat

```bash
# Start the registry
cargo run &

# Connect with websocat
websocat ws://localhost:4445

# List backends
{"jsonrpc":"2.0","id":1,"method":"registry.list","params":{"active_only":true}}

# Register a new backend
{"jsonrpc":"2.0","id":2,"method":"registry.register","params":{"name":"test","host":"127.0.0.1","port":8888}}
```

## Integration with Synapse

Synapse can discover backends from the registry:

```bash
# Synapse will query the registry for available backends
synapse hub2 cone list

# This internally:
# 1. Connects to registry at localhost:4445
# 2. Calls registry.get("hub2")
# 3. Connects to the discovered backend
# 4. Executes cone.list
```

## Architecture

### Data Flow

```
┌──────────┐         ┌──────────┐         ┌──────────┐
│ Synapse  │────────>│ Registry │────────>│  SQLite  │
│  Client  │ discover│  Service │  store  │ Database │
└──────────┘         └──────────┘         └──────────┘
                          │
                          │ load
                          ▼
                     ┌──────────┐
                     │   TOML   │
                     │  Config  │
                     └──────────┘
```

### Storage Priority

1. **Runtime registrations** (via `register` RPC)
2. **Database** (persisted from previous runs)
3. **Config file** (`~/.config/plexus/backends.toml`)
4. **Auto-registration** (self at localhost:4444)

### Source Types

- `Auto` - Automatically registered at startup (e.g., plexus @ localhost:4444)
- `File` - Loaded from TOML configuration file
- `Manual` - Registered via RPC call
- `Env` - Loaded from environment variables (future)

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### As a Library

```rust
use registry::{Registry, RegistryStorageConfig};
use hub_transport::TransportServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create registry with custom config
    let config = RegistryStorageConfig {
        db_path: "custom.db".into(),
        config_path: Some("backends.toml".into()),
    };

    let registry = Registry::new(config).await?;
    let registry_arc = Arc::new(registry);

    // Host via hub-transport
    let rpc_converter = |arc: Arc<Registry>| Ok(arc.into_rpc_methods());

    TransportServer::builder(registry_arc, rpc_converter)
        .with_websocket(4445)
        .build().await?
        .serve().await?;

    Ok(())
}
```

## License

AGPL-3.0-only
