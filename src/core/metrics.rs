//! Backend-agnostic metric traits.
//!
//! These traits define the interface for metrics that any backend
//! (Prometheus, OpenTelemetry, StatsD, etc.) can implement.

/// A monotonically increasing counter.
///
/// Counters are used for values that only go up, such as:
/// - Total HTTP requests
/// - Errors encountered
/// - Tasks completed
///
/// # Example
/// ```ignore
/// counter.inc();           // Increment by 1
/// counter.inc_by(5);       // Increment by 5
/// let value = counter.get(); // Get current value
/// ```
pub trait CounterTrait: Clone + Send + Sync + 'static {
    /// Increment the counter by 1.
    fn inc(&self);

    /// Increment the counter by a specific value.
    fn inc_by(&self, value: u64);

    /// Get the current counter value.
    fn get(&self) -> u64;
}

/// A gauge that can go up or down.
///
/// Gauges are used for values that fluctuate, such as:
/// - Current memory usage
/// - Active connections
/// - Temperature readings
/// - Queue depth
///
/// # Example
/// ```ignore
/// gauge.set(42);           // Set to specific value
/// gauge.inc();             // Increment by 1
/// gauge.dec();             // Decrement by 1
/// let value = gauge.get(); // Get current value
/// ```
pub trait GaugeTrait: Clone + Send + Sync + 'static {
    /// Set the gauge to a specific value.
    fn set(&self, value: i64);

    /// Increment the gauge by 1.
    fn inc(&self);

    /// Increment the gauge by a specific value.
    fn inc_by(&self, value: i64);

    /// Decrement the gauge by 1.
    fn dec(&self);

    /// Decrement the gauge by a specific value.
    fn dec_by(&self, value: i64);

    /// Get the current gauge value.
    fn get(&self) -> i64;
}

/// A histogram for recording distributions of values.
///
/// Histograms are used for measuring distributions, such as:
/// - Request latencies
/// - Response sizes
/// - Processing times
///
/// Values are placed into configured buckets for aggregation.
///
/// # Example
/// ```ignore
/// histogram.observe(0.042);  // Record a latency of 42ms
/// histogram.observe(0.156);  // Record a latency of 156ms
/// ```
pub trait HistogramTrait: Clone + Send + Sync + 'static {
    /// Record an observation in the histogram.
    fn observe(&self, value: f64);

    /// Get the current histogram sum and count.
    ///
    /// Returns `(sum, count)` where:
    /// - `sum` is the total of all observed values
    /// - `count` is the number of observations
    ///
    /// **Note:** This is primarily useful for testing with the mock backend.
    /// Prometheus histograms don't expose this externally - values are scraped
    /// via the `/metrics` endpoint instead.
    ///
    /// Default implementation returns `(0.0, 0)` for backends that don't support reading.
    fn get_histogram(&self) -> (f64, u64) {
        (0.0, 0)
    }
}

/// A metric with metadata (name and description).
///
/// This is a generic wrapper that works with any metric type
/// implementing the appropriate trait.
#[derive(Debug)]
pub struct Metric<T> {
    inner: T,
    name: String,
    description: String,
}

impl<T> Metric<T> {
    /// Create a new metric with the given name, description, and inner metric.
    pub fn new(name: impl Into<String>, description: impl Into<String>, inner: T) -> Self {
        Self {
            inner,
            name: name.into(),
            description: description.into(),
        }
    }

    /// Get the metric name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the metric description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Access the underlying metric.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Counter operations - delegated to inner type
// ═══════════════════════════════════════════════════════════════════════════

impl<T: CounterTrait> Metric<T> {
    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.inner.inc();
    }

    /// Increment the counter by a specific value.
    pub fn inc_by(&self, value: u64) {
        self.inner.inc_by(value);
    }

    /// Get the current counter value.
    pub fn get_counter(&self) -> u64 {
        self.inner.get()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Gauge operations - delegated to inner type
// ═══════════════════════════════════════════════════════════════════════════

impl<T: GaugeTrait> Metric<T> {
    /// Set the gauge to a specific value.
    pub fn set(&self, value: i64) {
        self.inner.set(value);
    }

    /// Increment the gauge by 1.
    pub fn gauge_inc(&self) {
        self.inner.inc();
    }

    /// Increment the gauge by a specific value.
    pub fn gauge_inc_by(&self, value: i64) {
        self.inner.inc_by(value);
    }

    /// Decrement the gauge by 1.
    pub fn dec(&self) {
        self.inner.dec();
    }

    /// Decrement the gauge by a specific value.
    pub fn dec_by(&self, value: i64) {
        self.inner.dec_by(value);
    }

    /// Get the current gauge value.
    pub fn get_gauge(&self) -> i64 {
        self.inner.get()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Histogram operations - delegated to inner type
// ═══════════════════════════════════════════════════════════════════════════

impl<T: HistogramTrait> Metric<T> {
    /// Record an observation in the histogram.
    pub fn observe(&self, value: f64) {
        self.inner.observe(value);
    }

    /// Get the current histogram value.
    pub fn get_histogram(&self) -> (f64, u64) {
        self.inner.get_histogram()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test with a simple mock implementation
    #[derive(Clone, Default)]
    struct TestCounter(std::sync::Arc<std::sync::atomic::AtomicU64>);

    impl CounterTrait for TestCounter {
        fn inc(&self) {
            self.0
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        fn inc_by(&self, value: u64) {
            self.0
                .fetch_add(value, std::sync::atomic::Ordering::Relaxed);
        }

        fn get(&self) -> u64 {
            self.0.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    #[test]
    fn test_metric_wrapper_with_counter() {
        let counter = Metric::new(
            "test_requests_total",
            "Total number of test requests",
            TestCounter::default(),
        );

        assert_eq!(counter.name(), "test_requests_total");
        assert_eq!(counter.description(), "Total number of test requests");
        assert_eq!(counter.get_counter(), 0);

        counter.inc();
        assert_eq!(counter.get_counter(), 1);

        counter.inc_by(10);
        assert_eq!(counter.get_counter(), 11);
    }
}

