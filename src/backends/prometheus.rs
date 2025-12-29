//! Prometheus backend implementation.
//!
//! This module implements the core metric traits for the `prometheus-client` crate.

use crate::core::metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};

// ═══════════════════════════════════════════════════════════════════════════
// CounterTrait implementation for prometheus-client Counter
// ═══════════════════════════════════════════════════════════════════════════

impl CounterTrait for Counter<u64> {
    fn inc(&self) {
        Counter::inc(self);
    }

    fn inc_by(&self, value: u64) {
        Counter::inc_by(self, value);
    }

    fn get(&self) -> u64 {
        Counter::get(self)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GaugeTrait implementation for prometheus-client Gauge
// ═══════════════════════════════════════════════════════════════════════════

impl GaugeTrait for Gauge<i64> {
    fn set(&self, value: i64) {
        Gauge::set(self, value);
    }

    fn inc(&self) {
        Gauge::inc(self);
    }

    fn inc_by(&self, value: i64) {
        Gauge::inc_by(self, value);
    }

    fn dec(&self) {
        Gauge::dec(self);
    }

    fn dec_by(&self, value: i64) {
        Gauge::dec_by(self, value);
    }

    fn get(&self) -> i64 {
        Gauge::get(self)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HistogramTrait implementation for prometheus-client Histogram
// ═══════════════════════════════════════════════════════════════════════════

impl HistogramTrait for Histogram {
    fn observe(&self, value: f64) {
        Histogram::observe(self, value);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Type aliases for convenience
// ═══════════════════════════════════════════════════════════════════════════

/// A Prometheus counter metric with metadata.
pub type PrometheusCounter = Metric<Counter<u64>>;

/// A Prometheus gauge metric with metadata.
pub type PrometheusGauge = Metric<Gauge<i64>>;

/// A Prometheus histogram metric with metadata.
pub type PrometheusHistogram = Metric<Histogram>;

// ═══════════════════════════════════════════════════════════════════════════
// Helper functions for creating metrics
// ═══════════════════════════════════════════════════════════════════════════

/// Create a new Prometheus counter.
pub fn counter(name: impl Into<String>, description: impl Into<String>) -> PrometheusCounter {
    Metric::new(name, description, Counter::default())
}

/// Create a new Prometheus gauge.
pub fn gauge(name: impl Into<String>, description: impl Into<String>) -> PrometheusGauge {
    Metric::new(name, description, Gauge::default())
}

/// Default general-purpose histogram buckets.
/// Exponential buckets covering a wide range: `[0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0]`
pub const DEFAULT_BUCKETS: [f64; 7] = [0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0];

/// Default histogram buckets for latency measurements (in seconds).
/// These are suitable for most HTTP request latency tracking.
pub const DEFAULT_LATENCY_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Default histogram buckets for size measurements (in bytes).
pub const DEFAULT_SIZE_BUCKETS: [f64; 10] = [
    100.0, 1_000.0, 10_000.0, 100_000.0, 1_000_000.0, 
    10_000_000.0, 100_000_000.0, 1_000_000_000.0, 10_000_000_000.0, 100_000_000_000.0,
];

/// Create a new Prometheus histogram with default general-purpose buckets.
///
/// Uses exponential buckets: `[0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0]`
///
/// For more specific use cases, prefer:
/// - [`histogram_for_latency`] for request durations
/// - [`histogram_for_bytes`] for byte sizes
/// - [`histogram_with_buckets`] for custom buckets
///
/// # Example
/// ```ignore
/// let metric = histogram("my_metric", "A general histogram");
/// metric.observe(0.5);
/// ```
pub fn histogram(name: impl Into<String>, description: impl Into<String>) -> PrometheusHistogram {
    histogram_with_buckets(name, description, DEFAULT_BUCKETS.into_iter())
}

/// Create a new Prometheus histogram optimized for latency measurements.
///
/// Uses exponential buckets suitable for request latencies (in seconds):
/// `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`
///
/// # Example
/// ```ignore
/// let latency = histogram_for_latency("request_duration_seconds", "Request latency in seconds");
/// latency.observe(0.042); // 42ms request
/// ```
pub fn histogram_for_latency(name: impl Into<String>, description: impl Into<String>) -> PrometheusHistogram {
    histogram_with_buckets(name, description, DEFAULT_LATENCY_BUCKETS.into_iter())
}

/// Create a new Prometheus histogram with custom buckets.
///
/// # Example
/// ```ignore
/// let latency = histogram_with_buckets(
///     "request_duration_seconds",
///     "Request latency",
///     [0.01, 0.05, 0.1, 0.5, 1.0, 5.0].into_iter(),
/// );
/// ```
pub fn histogram_with_buckets(
    name: impl Into<String>,
    description: impl Into<String>,
    buckets: impl Iterator<Item = f64>,
) -> PrometheusHistogram {
    Metric::new(name, description, Histogram::new(buckets))
}

/// Create a histogram suitable for measuring byte sizes.
///
/// Uses buckets: `[100, 1K, 10K, 100K, 1M, 10M, 100M, 1G, 10G, 100G]`
pub fn histogram_for_bytes(
    name: impl Into<String>,
    description: impl Into<String>,
) -> PrometheusHistogram {
    histogram_with_buckets(name, description, DEFAULT_SIZE_BUCKETS.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_counter() {
        let counter = counter("http_requests_total", "Total HTTP requests");

        assert_eq!(counter.get_counter(), 0);
        counter.inc();
        assert_eq!(counter.get_counter(), 1);
        counter.inc_by(5);
        assert_eq!(counter.get_counter(), 6);
    }

    #[test]
    fn test_prometheus_gauge() {
        let gauge = gauge("active_connections", "Number of active connections");

        assert_eq!(gauge.get_gauge(), 0);
        gauge.set(10);
        assert_eq!(gauge.get_gauge(), 10);
        gauge.gauge_inc();
        assert_eq!(gauge.get_gauge(), 11);
        gauge.dec();
        assert_eq!(gauge.get_gauge(), 10);
        gauge.dec_by(5);
        assert_eq!(gauge.get_gauge(), 5);
    }

    #[test]
    fn test_prometheus_histogram_default() {
        let metric = histogram("general_metric", "A general purpose histogram");

        // Observe values across the default bucket range
        metric.observe(0.005);
        metric.observe(0.5);
        metric.observe(50.0);
        metric.observe(500.0);

        assert_eq!(metric.name(), "general_metric");
        assert_eq!(metric.description(), "A general purpose histogram");
    }

    #[test]
    fn test_prometheus_histogram_for_latency() {
        let latency = histogram_for_latency("request_duration_seconds", "Request latency in seconds");

        // Observe some values
        latency.observe(0.001); // 1ms
        latency.observe(0.042); // 42ms
        latency.observe(0.5);   // 500ms
        latency.observe(2.0);   // 2s

        // Verify metadata
        assert_eq!(latency.name(), "request_duration_seconds");
        assert_eq!(latency.description(), "Request latency in seconds");
    }

    #[test]
    fn test_prometheus_histogram_custom_buckets() {
        let custom_buckets = [0.1, 0.5, 1.0, 5.0, 10.0];
        let latency = histogram_with_buckets(
            "custom_duration_seconds",
            "Custom latency histogram",
            custom_buckets.into_iter(),
        );

        latency.observe(0.25);
        latency.observe(0.75);
        latency.observe(3.0);

        assert_eq!(latency.name(), "custom_duration_seconds");
    }

    #[test]
    fn test_prometheus_histogram_for_bytes() {
        let response_size = histogram_for_bytes(
            "http_response_size_bytes",
            "HTTP response size in bytes",
        );

        response_size.observe(512.0);      // 512 bytes
        response_size.observe(15_000.0);   // 15 KB
        response_size.observe(5_000_000.0); // 5 MB

        assert_eq!(response_size.name(), "http_response_size_bytes");
    }
}

