//! Metrics rendering for different output formats.

/// Trait for registries that can render their metrics.
pub trait MetricsRenderer {
    /// Error type for rendering failures.
    type Error;

    /// Render metrics in the appropriate format (Prometheus text, JSON, etc.)
    fn render(&self) -> Result<RenderedMetrics, Self::Error>;
}

/// Wrapper for rendered metrics with content type.
///
/// This struct holds the serialized metrics output along with
/// the appropriate HTTP Content-Type header.
pub struct RenderedMetrics {
    /// The MIME content type (e.g., "text/plain; version=0.0.4")
    pub content_type: String,
    /// The raw bytes of the rendered output
    pub body: Vec<u8>,
}

impl RenderedMetrics {
    /// Create a new rendered metrics response.
    pub fn new(content_type: impl Into<String>, body: Vec<u8>) -> Self {
        Self {
            content_type: content_type.into(),
            body,
        }
    }

    /// Try to interpret the body as a UTF-8 string.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    /// Get the body as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Consume and return the body.
    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Prometheus MetricsRenderer implementation
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "prometheus")]
impl MetricsRenderer for prometheus_client::registry::Registry {
    type Error = std::fmt::Error;

    fn render(&self) -> Result<RenderedMetrics, Self::Error> {
        let mut buffer = String::new();
        prometheus_client::encoding::text::encode(&mut buffer, self)?;

        Ok(RenderedMetrics::new(
            "text/plain; version=0.0.4; charset=utf-8",
            buffer.into_bytes(),
        ))
    }
}
