//! Core traits and types for the observability kit.
//!
//! This module contains backend-agnostic abstractions that any metric
//! system can implement.

pub mod metrics;

pub use metrics::{CounterTrait, GaugeTrait, HistogramTrait};

