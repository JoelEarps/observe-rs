//! HTTP server and endpoint implementations.
//!
//! This module contains:
//! - Standalone HTTP server (feature: `standalone`)
//! - Health and readiness endpoints
//! - Metrics endpoint handlers

#[cfg(feature = "standalone")]
pub mod standalone;

pub mod health;

#[cfg(feature = "standalone")]
pub use standalone::*;

