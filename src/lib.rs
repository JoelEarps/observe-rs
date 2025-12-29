//! # Observability Kit
//!
//! A flexible, multi-backend observability library for Rust.
//!
//! ## Features
//!
//! - **Multi-backend support**: Prometheus, OpenTelemetry, StatsD, and more
//! - **Feature-gated**: Only compile what you need
//! - **Flexible deployment**: Standalone server or middleware integration
//! - **Testing utilities**: Mock backends for easy unit testing
//!
//! ## Quick Start
//!
//! ### Standalone Server (Sidecar/Embedded)
//!
//! ```ignore
//! use observability_kit::http::StandaloneServer;
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
//!
//! ### Creating Metrics
//!
//! ```ignore
//! use observability_kit::backends::prometheus::{counter, gauge};
//!
//! let requests = counter("http_requests_total", "Total HTTP requests");
//! let connections = gauge("active_connections", "Number of active connections");
//!
//! requests.inc();
//! connections.set(42);
//! ```
//!
//! ### Testing with Mock Backend
//!
//! ```ignore
//! use observability_kit::backends::mock::{test_counter, test_gauge};
//!
//! #[test]
//! fn test_my_function() {
//!     let counter = test_counter("test_metric", "For testing");
//!     
//!     my_function_that_increments(&counter);
//!     
//!     assert_eq!(counter.get_counter(), 1);
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `prometheus` | Prometheus metrics backend | ✓ |
//! | `otlp` | OpenTelemetry/OTLP backend | |
//! | `standalone` | Standalone HTTP server | ✓ |
//! | `axum-integration` | Axum middleware integration | |
//! | `mock` | Mock backend for testing | |
//! | `json-config` | JSON configuration support | |
//! | `yaml-config` | YAML configuration support | |

// Core module - always available
pub mod core;

// Feature-gated modules
pub mod backends;

#[cfg(feature = "standalone")]
pub mod http;

// Prelude for convenient imports
pub mod prelude {
    pub use crate::core::metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};

    #[cfg(feature = "prometheus")]
    pub use crate::backends::prometheus::{counter, gauge, PrometheusCounter, PrometheusGauge};

    #[cfg(feature = "mock")]
    pub use crate::backends::mock::{
        test_counter, test_gauge, test_histogram, MockCounter, MockGauge, MockHistogram,
        TestCounter, TestGauge, TestHistogram,
    };

    #[cfg(feature = "standalone")]
    pub use crate::http::standalone::{ServerConfig, StandaloneServer, StandaloneServerBuilder};
}

