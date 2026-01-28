//! Registry - Backend discovery and registration service for Plexus hubs
//!
//! This crate provides a standalone registry service that enables dynamic
//! discovery of Plexus backend instances. Multiple Synapse clients can
//! discover and connect to different Plexus hubs without manual configuration.
//!
//! # Architecture
//!
//! The registry stores backend connection information in SQLite and can be
//! configured via TOML files. It implements the Activation trait from hub-core
//! and can be hosted via hub-transport.
//!
//! # Example
//!
//! ```rust,no_run
//! use registry::{Registry, RegistryStorageConfig};
//! use hub_core::plexus::Plexus;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let registry = Registry::with_defaults().await.unwrap();
//! let plexus = Arc::new(Plexus::new().register(registry));
//! # }
//! ```

pub mod activation;
pub mod storage;
pub mod types;

// Re-export plexus module for hub_macro compatibility
// The hub_methods macro expects crate::plexus::* to be available
pub mod plexus {
    pub use hub_core::plexus::*;
    pub use hub_core::types::Handle;
}

// Re-export serde helpers for macro-generated code
// This allows the hub_methods macro to reference serde helpers via crate::serde_helpers
pub use hub_core::serde_helpers;

pub use activation::Registry;
pub use types::{BackendInfo, BackendSource, RegistryEvent, RegistryStorageConfig};
