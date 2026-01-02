//! Core traits and types for the observability kit.
//!
//! This module contains backend-agnostic abstractions that any metric
//! system can implement.

pub mod metrics;
pub mod registry;
pub mod renderer;

pub use metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
pub use registry::{MetricBackend, ObservabilityRegistry};
pub use renderer::{MetricsRenderer, RenderedMetrics};
