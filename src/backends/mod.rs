//! Metric backend implementations.
//!
//! Each backend is feature-gated and implements the core traits
//! from `crate::core::metrics`.

#[cfg(feature = "prometheus")]
pub mod prometheus;

#[cfg(feature = "mock")]
pub mod mock;

// Re-exports for convenience
#[cfg(feature = "prometheus")]
pub use self::prometheus::*;

#[cfg(feature = "mock")]
pub use self::mock::*;

