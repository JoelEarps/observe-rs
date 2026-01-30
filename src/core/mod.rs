//! Core traits and types for observe-rs.
//!
//! This module contains backend-agnostic abstractions that any metric
//! system can implement.

pub mod deserialise;
pub mod metrics;
pub mod registry;
pub mod renderer;

pub use metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
pub use registry::{MetricBackend, ObservabilityRegistry, DEFAULT_LATENCY_BUCKETS};
pub use renderer::{MetricsRenderer, RenderedMetrics};
