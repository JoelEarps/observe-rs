//! Standalone HTTP server for metrics exposure.
//!
//! This module provides a self-contained HTTP server that exposes
//! `/metrics`, `/health`, and `/ready` endpoints.
//!
//! # Example
//!
//! ```ignore
//! use observe_rs::http::standalone::StandaloneServer;
//! use observe_rs::backends::prometheus::PrometheusBackend;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = StandaloneServer::<PrometheusBackend>::builder()
//!         .port(9090)
//!         .build();
//!
//!     // Create metrics
//!     let requests = server.registry().counter("http_requests_total", "Total requests")?;
//!     requests.inc();
//!
//!     // Run the server
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::core::registry::{MetricBackend, ObservabilityRegistry};
use crate::core::renderer::MetricsRenderer;

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
pub struct StandaloneServerBuilder<B: MetricBackend> {
    config: ServerConfig,
    _marker: std::marker::PhantomData<B>,
}

impl<B: MetricBackend> Default for StandaloneServerBuilder<B> {
    fn default() -> Self {
        Self {
            config: ServerConfig::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B: MetricBackend> StandaloneServerBuilder<B> {
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
    pub fn build(self) -> StandaloneServer<B> {
        StandaloneServer {
            config: self.config,
            registry: Arc::new(RwLock::new(ObservabilityRegistry::<B>::new())),
        }
    }
}

/// Shared state for the HTTP handlers.
struct AppState<B: MetricBackend> {
    registry: Arc<RwLock<ObservabilityRegistry<B>>>,
}

impl<B: MetricBackend> Clone for AppState<B> {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

/// A standalone HTTP server for exposing metrics.
///
/// The server is generic over the metric backend, allowing you to use
/// Prometheus, OpenTelemetry, or any other supported backend.
///
/// # Example
/// ```ignore
/// use observe_rs::http::standalone::StandaloneServer;
/// use observe_rs::backends::prometheus::PrometheusBackend;
///
/// let server = StandaloneServer::<PrometheusBackend>::builder()
///     .port(9090)
///     .build();
///
/// // Get a handle to create metrics
/// let registry = server.registry();
/// let counter = registry.write().await.counter("my_counter", "A counter")?;
/// counter.inc();
///
/// // Run the server
/// server.run().await?;
/// ```
pub struct StandaloneServer<B: MetricBackend> {
    config: ServerConfig,
    registry: Arc<RwLock<ObservabilityRegistry<B>>>,
}

impl<B: MetricBackend> StandaloneServer<B> {
    /// Create a new builder for the standalone server.
    pub fn builder() -> StandaloneServerBuilder<B> {
        StandaloneServerBuilder::new()
    }

    /// Get the server configuration.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get a handle to the metrics registry.
    ///
    /// Use this to create metrics that will be exposed on the `/metrics` endpoint.
    ///
    /// # Example
    /// ```ignore
    /// let registry = server.registry();
    /// let counter = registry.write().await.counter("requests_total", "Total requests")?;
    /// counter.inc();
    /// ```
    pub fn registry(&self) -> Arc<RwLock<ObservabilityRegistry<B>>> {
        Arc::clone(&self.registry)
    }

    /// Run the server (blocking).
    pub async fn run(&self) -> Result<(), ServerError>
    where
        B::Registry: MetricsRenderer<Error = std::fmt::Error>,
    {
        let state = AppState {
            registry: Arc::clone(&self.registry),
        };

        let app = self.create_router(state);
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
    fn create_router(&self, state: AppState<B>) -> Router
    where
        B::Registry: MetricsRenderer<Error = std::fmt::Error>,
    {
        Router::new()
            .route(&self.config.metrics_path, get(metrics_handler::<B>))
            .route(&self.config.health_path, get(health_handler))
            .route(&self.config.ready_path, get(ready_handler))
            .with_state(state)
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

async fn metrics_handler<B: MetricBackend>(State(state): State<AppState<B>>) -> impl IntoResponse
where
    B::Registry: MetricsRenderer<Error = std::fmt::Error>,
{
    let registry = state.registry.read().await;

    match registry.render() {
        Ok(rendered) => {
            let content_type = rendered.content_type.clone();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                rendered.into_bytes(),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render metrics: {}", e),
        )
            .into_response(),
    }
}

async fn health_handler() -> (StatusCode, &'static str) {
    let status = default_health_check();
    let code = StatusCode::from_u16(status.status_code()).unwrap_or(StatusCode::OK);
    (code, "OK")
}

async fn ready_handler() -> (StatusCode, &'static str) {
    let status = default_readiness_check();
    let code = StatusCode::from_u16(status.status_code()).unwrap_or(StatusCode::OK);
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

    #[cfg(feature = "prometheus")]
    #[test]
    fn test_builder() {
        use crate::backends::prometheus::prometheus_backend::PrometheusBackend;

        let server = StandaloneServer::<PrometheusBackend>::builder()
            .port(3000)
            .host("127.0.0.1")
            .metrics_path("/prometheus")
            .build();

        assert_eq!(server.config().port, 3000);
        assert_eq!(server.config().host, "127.0.0.1");
        assert_eq!(server.config().metrics_path, "/prometheus");
    }
}
