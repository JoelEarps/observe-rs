//! Prometheus backend implementation.
//!
//! This module implements the core metric traits for the `prometheus-client` crate.

use crate::core::metrics::{CounterTrait, GaugeTrait, Metric};
use prometheus_client::metrics::{counter::Counter, gauge::Gauge};

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
// Type aliases for convenience
// ═══════════════════════════════════════════════════════════════════════════

/// A Prometheus counter metric with metadata.
pub type PrometheusCounter = Metric<Counter<u64>>;

/// A Prometheus gauge metric with metadata.
pub type PrometheusGauge = Metric<Gauge<i64>>;

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
}

