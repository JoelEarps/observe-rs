//! Test assertion helpers for metrics.
//!
//! This module provides convenient macros and functions for testing
//! metric values in your application tests.

#[cfg(feature = "mock")]
pub mod helpers {
    use observability_kit::backends::mock::MockHistogram;
    use observability_kit::core::metrics::{CounterTrait, GaugeTrait, Metric};

    /// Assert that a counter has the expected value.
    ///
    /// # Example
    /// ```ignore
    /// let counter = test_counter("my_counter", "description");
    /// counter.inc();
    /// assert_counter_value(&counter, 1);
    /// ```
    pub fn assert_counter_value<T: CounterTrait>(metric: &Metric<T>, expected: u64) {
        let actual = metric.get_counter();
        assert_eq!(
            actual,
            expected,
            "Counter '{}' expected {} but was {}",
            metric.name(),
            expected,
            actual
        );
    }

    /// Assert that a gauge has the expected value.
    ///
    /// # Example
    /// ```ignore
    /// let gauge = test_gauge("my_gauge", "description");
    /// gauge.set(42);
    /// assert_gauge_value(&gauge, 42);
    /// ```
    pub fn assert_gauge_value<T: GaugeTrait>(metric: &Metric<T>, expected: i64) {
        let actual = metric.get_gauge();
        assert_eq!(
            actual,
            expected,
            "Gauge '{}' expected {} but was {}",
            metric.name(),
            expected,
            actual
        );
    }

    /// Assert that a histogram has recorded the expected number of observations.
    pub fn assert_histogram_count(metric: &Metric<MockHistogram>, expected_count: usize) {
        let actual = metric.inner().count();
        assert_eq!(
            actual,
            expected_count,
            "Histogram '{}' expected {} observations but had {}",
            metric.name(),
            expected_count,
            actual
        );
    }

    /// Assert that a histogram sum is approximately equal to expected.
    pub fn assert_histogram_sum_approx(
        metric: &Metric<MockHistogram>,
        expected: f64,
        epsilon: f64,
    ) {
        let actual = metric.inner().sum();
        assert!(
            (actual - expected).abs() < epsilon,
            "Histogram '{}' sum expected ~{} (±{}) but was {}",
            metric.name(),
            expected,
            epsilon,
            actual
        );
    }

    /// Assert that a counter was incremented by a specific amount.
    ///
    /// Returns the new value for chaining.
    pub fn assert_counter_incremented<T: CounterTrait>(
        metric: &Metric<T>,
        previous: u64,
        increment: u64,
    ) -> u64 {
        let current = metric.get_counter();
        let expected = previous + increment;
        assert_eq!(
            current,
            expected,
            "Counter '{}' expected to increment from {} by {} to {}, but was {}",
            metric.name(),
            previous,
            increment,
            expected,
            current
        );
        current
    }
}

#[cfg(feature = "mock")]
mod tests {
    use super::helpers::*;
    use observability_kit::backends::mock::{test_counter, test_gauge, test_histogram};

    #[test]
    fn test_assert_counter_value() {
        let counter = test_counter("test", "test");
        counter.inc_by(5);
        assert_counter_value(&counter, 5);
    }

    #[test]
    fn test_assert_gauge_value() {
        let gauge = test_gauge("test", "test");
        gauge.set(42);
        assert_gauge_value(&gauge, 42);
    }

    #[test]
    fn test_assert_histogram_count() {
        let histogram = test_histogram("test", "test");
        histogram.observe(1.0);
        histogram.observe(2.0);
        assert_histogram_count(&histogram, 2);
    }

    #[test]
    fn test_assert_histogram_sum() {
        let histogram = test_histogram("test", "test");
        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);
        assert_histogram_sum_approx(&histogram, 6.0, 0.001);
    }

    #[test]
    fn test_assert_counter_incremented() {
        let counter = test_counter("test", "test");
        let v1 = counter.get_counter();
        counter.inc_by(10);
        let v2 = assert_counter_incremented(&counter, v1, 10);
        assert_eq!(v2, 10);
    }
}
