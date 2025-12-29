//! Integration tests for metric operations.
//!
//! These tests verify that metrics work correctly across different backends.

use observability_kit::core::metrics::{CounterTrait, GaugeTrait, Metric};

#[cfg(feature = "prometheus")]
mod prometheus_tests {
    use observability_kit::backends::prometheus::{counter, gauge};

    #[test]
    fn test_counter_basic_operations() {
        let requests = counter("http_requests_total", "Total HTTP requests");

        assert_eq!(requests.get_counter(), 0);

        requests.inc();
        assert_eq!(requests.get_counter(), 1);

        requests.inc_by(10);
        assert_eq!(requests.get_counter(), 11);
    }

    #[test]
    fn test_gauge_basic_operations() {
        let connections = gauge("active_connections", "Active connections");

        assert_eq!(connections.get_gauge(), 0);

        connections.set(100);
        assert_eq!(connections.get_gauge(), 100);

        connections.gauge_inc();
        assert_eq!(connections.get_gauge(), 101);

        connections.dec();
        assert_eq!(connections.get_gauge(), 100);

        connections.dec_by(50);
        assert_eq!(connections.get_gauge(), 50);
    }

    #[test]
    fn test_metric_metadata() {
        let requests = counter("my_counter", "My counter description");

        assert_eq!(requests.name(), "my_counter");
        assert_eq!(requests.description(), "My counter description");
    }

    #[test]
    fn test_counter_is_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(counter("thread_safe_counter", "A thread-safe counter"));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..100 {
                        counter.inc();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get_counter(), 1000);
    }
}

#[cfg(feature = "mock")]
mod mock_tests {
    use observability_kit::backends::mock::{test_counter, test_gauge, test_histogram};

    #[test]
    fn test_mock_counter() {
        let counter = test_counter("test_counter", "Test counter");

        assert_eq!(counter.get_counter(), 0);
        counter.inc();
        counter.inc_by(5);
        assert_eq!(counter.get_counter(), 6);
    }

    #[test]
    fn test_mock_gauge() {
        let gauge = test_gauge("test_gauge", "Test gauge");

        gauge.set(42);
        assert_eq!(gauge.get_gauge(), 42);

        gauge.dec_by(12);
        assert_eq!(gauge.get_gauge(), 30);
    }

    #[test]
    fn test_mock_histogram_observations() {
        let histogram = test_histogram("request_duration", "Request duration in seconds");

        histogram.observe(0.1);
        histogram.observe(0.2);
        histogram.observe(0.3);

        let inner = histogram.inner();
        assert_eq!(inner.count(), 3);

        let sum = inner.sum();
        assert!((sum - 0.6).abs() < 0.0001);

        let observations = inner.observations();
        assert_eq!(observations, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_mock_histogram_reset() {
        let histogram = test_histogram("resettable", "A resettable histogram");

        histogram.observe(1.0);
        histogram.observe(2.0);
        assert_eq!(histogram.inner().count(), 2);

        histogram.inner().reset();
        assert_eq!(histogram.inner().count(), 0);
    }

    #[test]
    fn test_mock_counter_clone_shares_state() {
        use observability_kit::backends::mock::MockCounter;

        let counter1 = MockCounter::new();
        let counter2 = counter1.clone();

        counter1.inc();
        assert_eq!(counter2.get(), 1);

        counter2.inc_by(5);
        assert_eq!(counter1.get(), 6);
    }
}

