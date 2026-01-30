//! Prometheus backend implementation.
//!
//! This module implements the core metric traits for the `prometheus-client` crate.
//!
//! # Labeled Metrics
//!
//! For metrics with labels, use the `Family` type with a custom label struct:
//!
//! ```ignore
//! use observe_rs::backends::prometheus::{Family, EncodeLabelSet, Histogram};
//!
//! #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
//! struct HttpLabels {
//!     method: String,
//!     status: u16,
//! }
//!
//! let latency: Family<HttpLabels, Histogram> = Family::new_with_constructor(|| {
//!     Histogram::new([0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0].into_iter())
//! });
//!
//! // Observe a value with specific labels
//! latency.get_or_create(&HttpLabels {
//!     method: "GET".into(),
//!     status: 200,
//! }).observe(0.042);
//! ```

use crate::core::metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
use crate::core::registry::{MetricBackend, ObservabilityRegistry};
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use prometheus_client::registry::Registry;

// Re-export key types for labeled metrics
pub use prometheus_client::encoding::EncodeLabelSet;
pub use prometheus_client::metrics::family::Family;

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
// MetricBackend implementation for Prometheus
// ═══════════════════════════════════════════════════════════════════════════

/// Error type for Prometheus registration operations.
#[derive(Debug, thiserror::Error)]
pub enum PrometheusError {
    #[error("Failed to register metric: {0}")]
    RegistrationError(String),

    /// Metric name does not match Prometheus rules: `[a-zA-Z_:][a-zA-Z0-9_:]*`
    #[error("Invalid metric name (Prometheus): {0}")]
    InvalidNamingConvention(String),

    /// Histogram buckets invalid (e.g. not finite, negative, or unsorted).
    #[error("Invalid histogram buckets: {0}")]
    InvalidHistogramBuckets(String),
}

/// First character of a Prometheus metric name: letter or underscore only.
fn is_valid_first_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Subsequent characters: letter, digit, or underscore.
fn is_valid_subsequent_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Prometheus metric names must match `[a-zA-Z_][a-zA-Z0-9_]*` (non-empty).
fn validate_prometheus_metric_name(name: &str) -> Result<(), PrometheusError> {
    if name.is_empty() {
        return Err(PrometheusError::InvalidNamingConvention(
            "metric name cannot be empty".to_string(),
        ));
    }
    // Chars is a wrapper around the iterator of a string slice therefore we can use any iterator methods on it
    let mut chars = name.chars();
    let first = chars.next().ok_or_else(|| {
        PrometheusError::InvalidNamingConvention("metric name cannot be empty".to_string())
    })?;
    if !is_valid_first_char(first) {
        return Err(PrometheusError::InvalidNamingConvention(format!(
            "metric name must start with [a-zA-Z_], got {:?}",
            first
        )));
    }
    for c in chars {
        if !is_valid_subsequent_char(c) {
            return Err(PrometheusError::InvalidNamingConvention(format!(
                "metric name may only contain [a-zA-Z0-9_], got invalid char {:?} in {:?}",
                c, name
            )));
        }
    }
    Ok(())
}

/// Histogram buckets must be finite, non-negative, and strictly increasing.
fn validate_histogram_buckets(buckets: &[f64]) -> Result<(), PrometheusError> {
    for (i, &b) in buckets.iter().enumerate() {
        if !b.is_finite() {
            return Err(PrometheusError::InvalidHistogramBuckets(format!(
                "bucket at index {} is not finite (NaN or Infinity): {}",
                i, b
            )));
        }
        if b < 0.0 {
            return Err(PrometheusError::InvalidHistogramBuckets(format!(
                "bucket at index {} is negative: {}",
                i, b
            )));
        }
        if i > 0 && b <= buckets[i - 1] {
            return Err(PrometheusError::InvalidHistogramBuckets(format!(
                "buckets must be strictly increasing; index {} ({}) <= previous ({})",
                i,
                b,
                buckets[i - 1]
            )));
        }
    }
    Ok(())
}

/// Prometheus backend marker type.
///
/// Use this with `ObservabilityRegistry<PrometheusBackend>` to create
/// a registry backed by prometheus-client.
pub struct PrometheusBackend;

impl MetricBackend for PrometheusBackend {
    type Registry = Registry;
    type Counter = Counter<u64>;
    type Gauge = Gauge<i64>;
    type Histogram = Histogram;
    type Error = PrometheusError;

    fn create_registry() -> Self::Registry {
        Registry::default()
    }

    fn register_counter(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
    ) -> Result<Self::Counter, Self::Error> {
        validate_prometheus_metric_name(name)?;
        let counter = Counter::default();
        registry.register(name, help, counter.clone());
        Ok(counter)
    }

    fn register_gauge(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
    ) -> Result<Self::Gauge, Self::Error> {
        validate_prometheus_metric_name(name)?;
        let gauge = Gauge::default();
        registry.register(name, help, gauge.clone());
        Ok(gauge)
    }

    fn register_histogram(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
        buckets: Vec<f64>,
    ) -> Result<Self::Histogram, Self::Error> {
        validate_prometheus_metric_name(name)?;
        validate_histogram_buckets(&buckets)?;
        let histogram = Histogram::new(buckets);
        registry.register(name, help, histogram.clone());
        Ok(histogram)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Type aliases for convenience
// ═══════════════════════════════════════════════════════════════════════════

/// A complete Prometheus metrics registry.
///
/// This is the recommended way to use Prometheus metrics with this library.
///
/// # Example
/// ```ignore
/// use observe_rs::backends::prometheus::PrometheusRegistry;
///
/// let mut registry = PrometheusRegistry::new();
///
/// let requests = registry.counter("http_requests_total", "Total requests")?;
/// let connections = registry.gauge("active_connections", "Active connections")?;
///
/// requests.inc();
/// connections.set(42);
///
/// // Render for /metrics endpoint
/// let output = registry.render()?;
/// println!("{}", output.as_str()?);
/// ```
pub type PrometheusRegistry = ObservabilityRegistry<PrometheusBackend>;

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
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
    100_000_000_000.0,
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
pub fn histogram_for_latency(
    name: impl Into<String>,
    description: impl Into<String>,
) -> PrometheusHistogram {
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

// ═══════════════════════════════════════════════════════════════════════════
// Labeled Metric Families
// ═══════════════════════════════════════════════════════════════════════════

/// A labeled histogram family type alias.
///
/// Use this with a custom label struct that derives `EncodeLabelSet`.
///
/// # Example
/// ```ignore
/// use observe_rs::backends::prometheus::{LabeledHistogram, EncodeLabelSet};
///
/// #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
/// struct HttpLabels {
///     method: String,
///     status: u16,
/// }
///
/// let latency = labeled_histogram_for_latency::<HttpLabels>();
/// latency.get_or_create(&HttpLabels { method: "GET".into(), status: 200 }).observe(0.042);
/// ```
pub type LabeledHistogram<L> = Family<L, Histogram>;

/// A labeled counter family type alias.
pub type LabeledCounter<L> = Family<L, Counter<u64>>;

/// A labeled gauge family type alias.
pub type LabeledGauge<L> = Family<L, Gauge<i64>>;

/// Create a labeled histogram family with default latency buckets.
///
/// Uses the same buckets as [`histogram_for_latency`]:
/// `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`
///
/// # Example
/// ```ignore
/// use observe_rs::backends::prometheus::{labeled_histogram_for_latency, EncodeLabelSet};
///
/// #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
/// struct HttpLabels {
///     method: String,
///     endpoint: String,
/// }
///
/// let latency = labeled_histogram_for_latency::<HttpLabels>();
///
/// // Record latency for GET /api/users
/// latency.get_or_create(&HttpLabels {
///     method: "GET".into(),
///     endpoint: "/api/users".into(),
/// }).observe(0.042);
/// ```
pub fn labeled_histogram_for_latency<L>() -> LabeledHistogram<L>
where
    L: EncodeLabelSet + Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    Family::new_with_constructor(create_latency_histogram)
}

fn create_latency_histogram() -> Histogram {
    Histogram::new(DEFAULT_LATENCY_BUCKETS)
}

/// Create a labeled histogram family with default byte size buckets.
///
/// Uses the same buckets as [`histogram_for_bytes`]:
/// `[100, 1K, 10K, 100K, 1M, 10M, 100M, 1G, 10G, 100G]`
pub fn labeled_histogram_for_bytes<L>() -> LabeledHistogram<L>
where
    L: EncodeLabelSet + Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    Family::new_with_constructor(create_bytes_histogram)
}

fn create_bytes_histogram() -> Histogram {
    Histogram::new(DEFAULT_SIZE_BUCKETS)
}

/// Create a labeled histogram family with default general-purpose buckets.
///
/// Uses the same buckets as [`histogram`]:
/// `[0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0]`
pub fn labeled_histogram<L>() -> LabeledHistogram<L>
where
    L: EncodeLabelSet + Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    Family::new_with_constructor(create_default_histogram)
}

fn create_default_histogram() -> Histogram {
    Histogram::new(DEFAULT_BUCKETS)
}

/// Create a labeled counter family.
pub fn labeled_counter<L>() -> LabeledCounter<L>
where
    L: EncodeLabelSet + Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    Family::default()
}

/// Create a labeled gauge family.
pub fn labeled_gauge<L>() -> LabeledGauge<L>
where
    L: EncodeLabelSet + Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    Family::default()
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
        let latency =
            histogram_for_latency("request_duration_seconds", "Request latency in seconds");

        // Observe some values
        latency.observe(0.001); // 1ms
        latency.observe(0.042); // 42ms
        latency.observe(0.5); // 500ms
        latency.observe(2.0); // 2s

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
        let response_size =
            histogram_for_bytes("http_response_size_bytes", "HTTP response size in bytes");

        response_size.observe(512.0); // 512 bytes
        response_size.observe(15_000.0); // 15 KB
        response_size.observe(5_000_000.0); // 5 MB

        assert_eq!(response_size.name(), "http_response_size_bytes");
    }

    #[test]
    fn test_prometheus_registry_creates_and_registers_metrics() {
        let mut registry = PrometheusRegistry::new();

        // Create metrics via the registry
        let requests = registry
            .counter("http_requests_total", "Total HTTP requests")
            .unwrap();
        let connections = registry
            .gauge("active_connections", "Active connections")
            .unwrap();
        let latency = registry
            .histogram("request_duration_seconds", "Request latency")
            .unwrap();

        // Use the metrics
        requests.inc();
        requests.inc_by(5);
        connections.set(42);
        latency.observe(0.042);
        latency.observe(0.156);

        // Verify values
        assert_eq!(requests.get_counter(), 6);
        assert_eq!(connections.get_gauge(), 42);
    }

    #[test]
    fn test_prometheus_registry_renders_metrics() {
        let mut registry = PrometheusRegistry::new();

        let requests = registry
            .counter("test_requests_total", "Test counter")
            .unwrap();
        requests.inc();
        requests.inc_by(10);

        let gauge = registry.gauge("test_gauge", "Test gauge").unwrap();
        gauge.set(42);

        // Render the metrics
        let output = registry.render().unwrap();
        let text = output.as_str().unwrap();

        // Verify the output contains our metrics
        assert!(text.contains("test_requests_total"));
        assert!(text.contains("11")); // 1 + 10
        assert!(text.contains("test_gauge"));
        assert!(text.contains("42"));

        // Verify content type
        assert!(output.content_type.contains("text/plain"));
    }

    #[test]
    fn test_labeled_histogram_for_latency() {
        use std::hash::Hash;

        #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
        struct HttpLabels {
            method: String,
            status: u16,
        }

        let latency: LabeledHistogram<HttpLabels> = labeled_histogram_for_latency();

        // Record latency for different label combinations
        latency
            .get_or_create(&HttpLabels {
                method: "GET".into(),
                status: 200,
            })
            .observe(0.042);

        latency
            .get_or_create(&HttpLabels {
                method: "POST".into(),
                status: 201,
            })
            .observe(0.156);

        latency
            .get_or_create(&HttpLabels {
                method: "GET".into(),
                status: 404,
            })
            .observe(0.008);

        // Record multiple observations for the same labels
        latency
            .get_or_create(&HttpLabels {
                method: "GET".into(),
                status: 200,
            })
            .observe(0.089);
    }

    #[test]
    fn test_labeled_counter() {
        #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
        struct RequestLabels {
            method: String,
            path: String,
        }

        let requests: LabeledCounter<RequestLabels> = labeled_counter();

        requests
            .get_or_create(&RequestLabels {
                method: "GET".into(),
                path: "/api/users".into(),
            })
            .inc();

        requests
            .get_or_create(&RequestLabels {
                method: "POST".into(),
                path: "/api/users".into(),
            })
            .inc_by(5);

        // Verify counts
        let get_count = requests
            .get_or_create(&RequestLabels {
                method: "GET".into(),
                path: "/api/users".into(),
            })
            .get();
        assert_eq!(get_count, 1);

        let post_count = requests
            .get_or_create(&RequestLabels {
                method: "POST".into(),
                path: "/api/users".into(),
            })
            .get();
        assert_eq!(post_count, 5);
    }

    #[test]
    fn test_labeled_gauge() {
        #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
        struct ConnectionLabels {
            pool: String,
        }

        let connections: LabeledGauge<ConnectionLabels> = labeled_gauge();

        connections
            .get_or_create(&ConnectionLabels {
                pool: "primary".into(),
            })
            .set(10);

        connections
            .get_or_create(&ConnectionLabels {
                pool: "replica".into(),
            })
            .set(5);

        // Verify values
        let primary = connections
            .get_or_create(&ConnectionLabels {
                pool: "primary".into(),
            })
            .get();
        assert_eq!(primary, 10);
    }

    #[test]
    fn validation_empty_metric_name_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.counter("", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention, got {:?}",
            result
        );
    }

    #[test]
    fn validation_metric_name_starting_with_digit_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.counter("123bad", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention, got {:?}",
            result
        );
    }

    #[test]
    fn validation_metric_name_with_hyphen_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.counter("my-metric", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention, got {:?}",
            result
        );
    }

    #[test]
    fn validation_metric_name_with_colon_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.counter("name_with:colon", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention (colons not allowed), got {:?}",
            result
        );
    }

    #[test]
    fn validation_valid_metric_names_accepted() {
        let mut registry = PrometheusRegistry::new();
        assert!(registry.counter("http_requests_total", "help").is_ok());
        assert!(registry.gauge("_private_metric", "help").is_ok());
        assert!(registry.counter("request_count_42", "help").is_ok());
    }

    #[test]
    fn validation_single_char_valid_names_accepted() {
        let mut registry = PrometheusRegistry::new();
        assert!(registry.counter("a", "help").is_ok());
        assert!(registry.gauge("_", "help").is_ok());
        assert!(registry.counter("Z", "help").is_ok());
    }

    #[test]
    fn validation_metric_name_with_dot_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.counter("my.metric", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention (dots not allowed), got {:?}",
            result
        );
    }

    #[test]
    fn validation_gauge_invalid_name_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.gauge("bad-name", "help");
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention for gauge, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_invalid_name_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("bad.name", "help", vec![1.0]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidNamingConvention(_))),
            "expected InvalidNamingConvention for histogram, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_empty_accepted() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![]);
        assert!(
            result.is_ok(),
            "empty buckets should be accepted: {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_negative_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![-1.0, 1.0]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidHistogramBuckets(_))),
            "expected InvalidHistogramBuckets, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_nan_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![f64::NAN, 1.0]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidHistogramBuckets(_))),
            "expected InvalidHistogramBuckets, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_infinity_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![1.0, f64::INFINITY]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidHistogramBuckets(_))),
            "expected InvalidHistogramBuckets, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_unsorted_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![1.0, 0.5, 2.0]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidHistogramBuckets(_))),
            "expected InvalidHistogramBuckets, got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_duplicate_rejected() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("h", "help", vec![0.5, 0.5, 1.0]);
        assert!(
            matches!(result, Err(PrometheusError::InvalidHistogramBuckets(_))),
            "expected InvalidHistogramBuckets (strictly increasing), got {:?}",
            result
        );
    }

    #[test]
    fn validation_histogram_buckets_valid_accepted() {
        let mut registry = PrometheusRegistry::new();
        let result = registry.histogram_with_buckets("latency", "help", vec![0.1, 0.5, 1.0]);
        assert!(
            result.is_ok(),
            "valid buckets should be accepted: {:?}",
            result
        );
    }
}
