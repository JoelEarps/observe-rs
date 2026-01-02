//! Integration tests for HTTP endpoints.
//!
//! These tests verify the HTTP server endpoints work correctly.

#[cfg(feature = "standalone")]
mod http_tests {
    use observability_kit::http::health::{HealthStatus, ReadinessStatus};
    use observability_kit::http::standalone::ServerConfig;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();

        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.metrics_path, "/metrics");
        assert_eq!(config.health_path, "/health");
        assert_eq!(config.ready_path, "/ready");
    }

    #[cfg(feature = "prometheus")]
    #[test]
    fn test_server_builder_customization() {
        use observability_kit::backends::prometheus::PrometheusBackend;
        use observability_kit::http::standalone::StandaloneServer;

        let server = StandaloneServer::<PrometheusBackend>::builder()
            .port(3000)
            .host("127.0.0.1")
            .metrics_path("/prometheus/metrics")
            .health_path("/healthz")
            .ready_path("/readyz")
            .build();

        let config = server.config();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.metrics_path, "/prometheus/metrics");
        assert_eq!(config.health_path, "/healthz");
        assert_eq!(config.ready_path, "/readyz");
    }

    #[test]
    fn test_health_status_codes() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert_eq!(healthy.status_code(), 200);

        let unhealthy = HealthStatus::Unhealthy(Some("Database connection failed".to_string()));
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.status_code(), 503);

        let unhealthy_no_reason = HealthStatus::Unhealthy(None);
        assert!(!unhealthy_no_reason.is_healthy());
        assert_eq!(unhealthy_no_reason.status_code(), 503);
    }

    #[test]
    fn test_readiness_status_codes() {
        let ready = ReadinessStatus::Ready;
        assert!(ready.is_ready());
        assert_eq!(ready.status_code(), 200);

        let not_ready = ReadinessStatus::NotReady(Some("Still warming up".to_string()));
        assert!(!not_ready.is_ready());
        assert_eq!(not_ready.status_code(), 503);

        let not_ready_no_reason = ReadinessStatus::NotReady(None);
        assert!(!not_ready_no_reason.is_ready());
        assert_eq!(not_ready_no_reason.status_code(), 503);
    }

    // Integration test that actually starts the server and makes HTTP requests
    // Note: This test requires full network access and may not work in sandboxed environments.
    // Run with: cargo test --features "prometheus mock" -- --ignored
    #[cfg(feature = "prometheus")]
    #[tokio::test]
    #[ignore = "Requires network access - run manually with --ignored flag"]
    async fn test_server_endpoints_integration() {
        use observability_kit::backends::prometheus::PrometheusBackend;
        use observability_kit::http::standalone::StandaloneServer;
        use std::net::TcpListener;
        use std::time::Duration;
        use tokio::time::timeout;

        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Release the port

        let server = StandaloneServer::<PrometheusBackend>::builder()
            .port(port)
            .host("127.0.0.1")
            .build();

        // Spawn server in background
        let server_handle = tokio::spawn(async move { server.run().await });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create HTTP client
        let client = reqwest::Client::new();

        // Test health endpoint
        let health_resp = timeout(
            Duration::from_secs(5),
            client
                .get(format!("http://127.0.0.1:{}/health", port))
                .send(),
        )
        .await
        .expect("Health request timed out")
        .expect("Health request failed");

        assert_eq!(health_resp.status(), 200);
        assert_eq!(health_resp.text().await.unwrap(), "OK");

        // Test ready endpoint
        let ready_resp = timeout(
            Duration::from_secs(5),
            client
                .get(format!("http://127.0.0.1:{}/ready", port))
                .send(),
        )
        .await
        .expect("Ready request timed out")
        .expect("Ready request failed");

        assert_eq!(ready_resp.status(), 200);
        assert_eq!(ready_resp.text().await.unwrap(), "OK");

        // Test metrics endpoint
        let metrics_resp = timeout(
            Duration::from_secs(5),
            client
                .get(format!("http://127.0.0.1:{}/metrics", port))
                .send(),
        )
        .await
        .expect("Metrics request timed out")
        .expect("Metrics request failed");

        assert_eq!(metrics_resp.status(), 200);

        // Cleanup - abort the server
        server_handle.abort();
    }
}
