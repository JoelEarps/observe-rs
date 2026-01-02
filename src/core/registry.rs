//! Generic registry wrapper for metric backends.
//!
//! This module provides a unified interface for creating, registering,
//! and rendering metrics across different backends.

use super::metrics::{CounterTrait, GaugeTrait, HistogramTrait, Metric};
use super::renderer::{MetricsRenderer, RenderedMetrics};

/// Trait that defines what a backend must provide.
///
/// Each backend (Prometheus, OTLP, etc.) implements this trait to specify
/// its concrete types for registry and metrics.
pub trait MetricBackend: Send + Sync + 'static {
    /// The registry type for this backend
    type Registry: MetricsRenderer + Send + Sync;

    /// The counter type for this backend
    type Counter: CounterTrait;

    /// The gauge type for this backend
    type Gauge: GaugeTrait;

    /// The histogram type for this backend
    type Histogram: HistogramTrait;

    /// Error type for registration failures
    type Error: std::error::Error + Send + Sync;

    /// Create a new registry
    fn create_registry() -> Self::Registry;

    /// Create and register a counter
    fn register_counter(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
    ) -> Result<Self::Counter, Self::Error>;

    /// Create and register a gauge
    fn register_gauge(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
    ) -> Result<Self::Gauge, Self::Error>;

    /// Create and register a histogram with custom buckets
    fn register_histogram(
        registry: &mut Self::Registry,
        name: &str,
        help: &str,
        buckets: Vec<f64>,
    ) -> Result<Self::Histogram, Self::Error>;
}

/// A wrapper around a metric backend's registry.
///
/// Provides a unified API for creating metrics that are automatically
/// registered to the underlying registry.
///
/// # Example
/// ```ignore
/// use observability_kit::core::registry::ObservabilityRegistry;
/// use observability_kit::backends::prometheus::PrometheusBackend;
///
/// let mut registry = ObservabilityRegistry::<PrometheusBackend>::new();
///
/// let requests = registry.counter("http_requests_total", "Total HTTP requests")?;
/// let connections = registry.gauge("active_connections", "Active connections")?;
///
/// requests.inc();
/// connections.set(42);
///
/// let output = registry.render()?;
/// println!("{}", output.as_str()?);
/// ```
pub struct ObservabilityRegistry<B: MetricBackend> {
    inner: B::Registry,
}

impl<B: MetricBackend> ObservabilityRegistry<B> {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            inner: B::create_registry(),
        }
    }

    /// Create and register a counter.
    pub fn counter(
        &mut self,
        name: impl Into<String>,
        help: impl Into<String>,
    ) -> Result<Metric<B::Counter>, B::Error> {
        let name = name.into();
        let help = help.into();
        let counter = B::register_counter(&mut self.inner, &name, &help)?;
        Ok(Metric::new(name, help, counter))
    }

    /// Create and register a gauge.
    pub fn gauge(
        &mut self,
        name: impl Into<String>,
        help: impl Into<String>,
    ) -> Result<Metric<B::Gauge>, B::Error> {
        let name = name.into();
        let help = help.into();
        let gauge = B::register_gauge(&mut self.inner, &name, &help)?;
        Ok(Metric::new(name, help, gauge))
    }

    /// Create and register a histogram with default latency buckets.
    pub fn histogram(
        &mut self,
        name: impl Into<String>,
        help: impl Into<String>,
    ) -> Result<Metric<B::Histogram>, B::Error> {
        self.histogram_with_buckets(
            name,
            help,
            vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
    }

    /// Create and register a histogram with custom buckets.
    pub fn histogram_with_buckets(
        &mut self,
        name: impl Into<String>,
        help: impl Into<String>,
        buckets: Vec<f64>,
    ) -> Result<Metric<B::Histogram>, B::Error> {
        let name = name.into();
        let help = help.into();
        let histogram = B::register_histogram(&mut self.inner, &name, &help, buckets)?;
        Ok(Metric::new(name, help, histogram))
    }

    /// Render the metrics in the backend's format.
    pub fn render(&self) -> Result<RenderedMetrics, <B::Registry as MetricsRenderer>::Error> {
        self.inner.render()
    }

    /// Get a reference to the underlying registry.
    pub fn inner(&self) -> &B::Registry {
        &self.inner
    }

    /// Get a mutable reference to the underlying registry.
    pub fn inner_mut(&mut self) -> &mut B::Registry {
        &mut self.inner
    }
}

impl<B: MetricBackend> Default for ObservabilityRegistry<B> {
    fn default() -> Self {
        Self::new()
    }
}
