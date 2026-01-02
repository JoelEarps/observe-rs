//! Mock backend for testing.
//!
//! This module provides lightweight metric implementations using atomics,
//! perfect for unit testing without needing a real metrics backend.

use crate::core::metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════
// MockCounter
// ═══════════════════════════════════════════════════════════════════════════

/// A mock counter for testing purposes.
///
/// Uses atomic operations for thread-safe counting without
/// requiring a full metrics backend.
#[derive(Clone, Default, Debug)]
pub struct MockCounter(Arc<AtomicU64>);

impl MockCounter {
    /// Create a new mock counter starting at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mock counter with an initial value.
    pub fn with_value(value: u64) -> Self {
        Self(Arc::new(AtomicU64::new(value)))
    }

    /// Reset the counter to 0.
    pub fn reset(&self) {
        self.0.store(0, Ordering::Relaxed);
    }
}

impl CounterTrait for MockCounter {
    fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_by(&self, value: u64) {
        self.0.fetch_add(value, Ordering::Relaxed);
    }

    fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MockGauge
// ═══════════════════════════════════════════════════════════════════════════

/// A mock gauge for testing purposes.
///
/// Uses atomic operations for thread-safe value updates without
/// requiring a full metrics backend.
#[derive(Clone, Default, Debug)]
pub struct MockGauge(Arc<AtomicI64>);

impl MockGauge {
    /// Create a new mock gauge starting at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mock gauge with an initial value.
    pub fn with_value(value: i64) -> Self {
        Self(Arc::new(AtomicI64::new(value)))
    }

    /// Reset the gauge to 0.
    pub fn reset(&self) {
        self.0.store(0, Ordering::Relaxed);
    }
}

impl GaugeTrait for MockGauge {
    fn set(&self, value: i64) {
        self.0.store(value, Ordering::Relaxed);
    }

    fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_by(&self, value: i64) {
        self.0.fetch_add(value, Ordering::Relaxed);
    }

    fn dec(&self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }

    fn dec_by(&self, value: i64) {
        self.0.fetch_sub(value, Ordering::Relaxed);
    }

    fn get(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MockHistogram
// ═══════════════════════════════════════════════════════════════════════════

/// A mock histogram for testing purposes.
///
/// Records observations in a simple list for later inspection.
#[derive(Clone, Default, Debug)]
pub struct MockHistogram {
    observations: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl MockHistogram {
    /// Create a new mock histogram.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all recorded observations.
    pub fn observations(&self) -> Vec<f64> {
        self.observations.lock().unwrap().clone()
    }

    /// Get the count of observations.
    pub fn count(&self) -> usize {
        self.observations.lock().unwrap().len()
    }

    /// Get the sum of all observations.
    pub fn sum(&self) -> f64 {
        self.observations.lock().unwrap().iter().sum()
    }

    /// Clear all observations.
    pub fn reset(&self) {
        self.observations.lock().unwrap().clear();
    }
}

impl HistogramTrait for MockHistogram {
    fn observe(&self, value: f64) {
        self.observations.lock().unwrap().push(value);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Type aliases for convenience
// ═══════════════════════════════════════════════════════════════════════════

/// A mock counter metric with metadata.
pub type TestCounter = Metric<MockCounter>;

/// A mock gauge metric with metadata.
pub type TestGauge = Metric<MockGauge>;

/// A mock histogram metric with metadata.
pub type TestHistogram = Metric<MockHistogram>;

// ═══════════════════════════════════════════════════════════════════════════
// Helper functions for creating test metrics
// ═══════════════════════════════════════════════════════════════════════════

/// Create a new mock counter for testing.
pub fn test_counter(name: impl Into<String>, description: impl Into<String>) -> TestCounter {
    Metric::new(name, description, MockCounter::new())
}

/// Create a new mock gauge for testing.
pub fn test_gauge(name: impl Into<String>, description: impl Into<String>) -> TestGauge {
    Metric::new(name, description, MockGauge::new())
}

/// Create a new mock histogram for testing.
pub fn test_histogram(name: impl Into<String>, description: impl Into<String>) -> TestHistogram {
    Metric::new(name, description, MockHistogram::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_counter() {
        let counter = test_counter("test_counter", "A test counter");

        assert_eq!(counter.get_counter(), 0);
        counter.inc();
        assert_eq!(counter.get_counter(), 1);
        counter.inc_by(10);
        assert_eq!(counter.get_counter(), 11);
    }

    #[test]
    fn test_mock_gauge() {
        let gauge = test_gauge("test_gauge", "A test gauge");

        assert_eq!(gauge.get_gauge(), 0);
        gauge.set(100);
        assert_eq!(gauge.get_gauge(), 100);
        gauge.dec_by(30);
        assert_eq!(gauge.get_gauge(), 70);
    }

    #[test]
    fn test_mock_histogram() {
        let histogram = test_histogram("test_histogram", "A test histogram");

        histogram.observe(0.1);
        histogram.observe(0.2);
        histogram.observe(0.3);

        assert_eq!(histogram.inner().count(), 3);
        assert!((histogram.inner().sum() - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_mock_counter_is_clone() {
        let counter = MockCounter::new();
        let cloned = counter.clone();

        counter.inc();
        assert_eq!(cloned.get(), 1); // Both see the same value
    }
}
