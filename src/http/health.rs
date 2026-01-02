//! Health and readiness endpoint logic.
//!
//! These endpoints follow Kubernetes conventions for container probes.

/// Health check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// The service is healthy.
    Healthy,
    /// The service is unhealthy with an optional reason.
    Unhealthy(Option<String>),
}

impl HealthStatus {
    /// Returns true if the status is healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Returns the HTTP status code for this health status.
    pub fn status_code(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Unhealthy(_) => 503,
        }
    }
}

/// Readiness check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadinessStatus {
    /// The service is ready to receive traffic.
    Ready,
    /// The service is not ready with an optional reason.
    NotReady(Option<String>),
}

impl ReadinessStatus {
    /// Returns true if the status is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, ReadinessStatus::Ready)
    }

    /// Returns the HTTP status code for this readiness status.
    pub fn status_code(&self) -> u16 {
        match self {
            ReadinessStatus::Ready => 200,
            ReadinessStatus::NotReady(_) => 503,
        }
    }
}

/// A health check function that can be provided by the user.
pub type HealthCheckFn = Box<dyn Fn() -> HealthStatus + Send + Sync>;

/// A readiness check function that can be provided by the user.
pub type ReadinessCheckFn = Box<dyn Fn() -> ReadinessStatus + Send + Sync>;

/// Default health check - always returns healthy.
pub fn default_health_check() -> HealthStatus {
    HealthStatus::Healthy
}

/// Default readiness check - always returns ready.
pub fn default_readiness_check() -> ReadinessStatus {
    ReadinessStatus::Ready
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert_eq!(healthy.status_code(), 200);

        let unhealthy = HealthStatus::Unhealthy(Some("Database down".to_string()));
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.status_code(), 503);
    }

    #[test]
    fn test_readiness_status() {
        let ready = ReadinessStatus::Ready;
        assert!(ready.is_ready());
        assert_eq!(ready.status_code(), 200);

        let not_ready = ReadinessStatus::NotReady(Some("Warming up".to_string()));
        assert!(!not_ready.is_ready());
        assert_eq!(not_ready.status_code(), 503);
    }
}
