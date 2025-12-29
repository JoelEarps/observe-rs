//! Standalone HTTP server for metrics exposure.
//!
//! This module provides a self-contained HTTP server that exposes
//! `/metrics`, `/health`, and `/ready` endpoints.
//!
//! # Example
//!
//! ```ignore
//! use observability_kit::http::standalone::StandaloneServer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = StandaloneServer::builder()
//!         .port(9090)
//!         .build();
//!
//!     server.run().await.unwrap();
//! }
//! ```

use axum::{routing::get, Router};
use tokio::net::TcpListener;

use super::health::{default_health_check, default_readiness_check};

/// Configuration for the standalone server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The port to bind to (default: 9090)
    pub port: u16,
    /// The host to bind to (default: "0.0.0.0")
    pub host: String,
    /// Path for the metrics endpoint (default: "/metrics")
    pub metrics_path: String,
    /// Path for the health endpoint (default: "/health")
    pub health_path: String,
    /// Path for the readiness endpoint (default: "/ready")
    pub ready_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 9090,
            host: "0.0.0.0".to_string(),
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
            ready_path: "/ready".to_string(),
        }
    }
}

/// Builder for creating a standalone server.
#[derive(Default)]
pub struct StandaloneServerBuilder {
    config: ServerConfig,
}

impl StandaloneServerBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the port to bind to.
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set the host to bind to.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Set the metrics endpoint path.
    pub fn metrics_path(mut self, path: impl Into<String>) -> Self {
        self.config.metrics_path = path.into();
        self
    }

    /// Set the health endpoint path.
    pub fn health_path(mut self, path: impl Into<String>) -> Self {
        self.config.health_path = path.into();
        self
    }

    /// Set the readiness endpoint path.
    pub fn ready_path(mut self, path: impl Into<String>) -> Self {
        self.config.ready_path = path.into();
        self
    }

    /// Build the standalone server.
    pub fn build(self) -> StandaloneServer {
        StandaloneServer {
            config: self.config,
        }
    }
}

/// A standalone HTTP server for exposing metrics.
pub struct StandaloneServer {
    config: ServerConfig,
}

impl StandaloneServer {
    /// Create a new builder for the standalone server.
    pub fn builder() -> StandaloneServerBuilder {
        StandaloneServerBuilder::new()
    }

    /// Get the server configuration.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Run the server (blocking).
    pub async fn run(&self) -> Result<(), ServerError> {
        let app = self.create_router();
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        println!(
            "Observability server listening on http://{}",
            listener.local_addr().unwrap()
        );

        axum::serve(listener, app)
            .await
            .map_err(|e| ServerError::ServeError(e.to_string()))?;

        Ok(())
    }

    /// Create the router with all endpoints.
    fn create_router(&self) -> Router {
        Router::new()
            .route(&self.config.metrics_path, get(metrics_handler))
            .route(&self.config.health_path, get(health_handler))
            .route(&self.config.ready_path, get(ready_handler))
    }
}

/// Server error types.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    BindError(String),
    #[error("Server error: {0}")]
    ServeError(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// HTTP Handlers
// ═══════════════════════════════════════════════════════════════════════════

async fn metrics_handler() -> &'static str {
    // TODO: Wire up to actual registry encoding
    "# No metrics registered yet\n"
}

async fn health_handler() -> (axum::http::StatusCode, &'static str) {
    let status = default_health_check();
    let code = axum::http::StatusCode::from_u16(status.status_code())
        .unwrap_or(axum::http::StatusCode::OK);
    (code, "OK")
}

async fn ready_handler() -> (axum::http::StatusCode, &'static str) {
    let status = default_readiness_check();
    let code = axum::http::StatusCode::from_u16(status.status_code())
        .unwrap_or(axum::http::StatusCode::OK);
    (code, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.metrics_path, "/metrics");
        assert_eq!(config.health_path, "/health");
        assert_eq!(config.ready_path, "/ready");
    }

    #[test]
    fn test_builder() {
        let server = StandaloneServer::builder()
            .port(3000)
            .host("127.0.0.1")
            .metrics_path("/prometheus")
            .build();

        assert_eq!(server.config().port, 3000);
        assert_eq!(server.config().host, "127.0.0.1");
        assert_eq!(server.config().metrics_path, "/prometheus");
    }
}

