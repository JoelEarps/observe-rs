//! Example: Running observe-rs as a standalone server.
//!
//! This demonstrates how to use the library in standalone mode,
//! suitable for sidecar deployments or embedded metrics servers.
//!
//! Run with:
//! ```bash
//! cargo run --example standalone-prometheus --features "prometheus standalone"
//! ```

use observe_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔭 Observability Kit - Standalone Server Example");
    println!("================================================\n");

    // Create a standalone server with Prometheus backend
    #[cfg(all(feature = "prometheus", feature = "standalone"))]
    {
        let server = StandaloneServer::<PrometheusBackend>::builder()
            .port(9090)
            .host("127.0.0.1")
            .build();

        // Get the registry and create some metrics
        {
            let registry_handle = server.registry();
            let mut registry = registry_handle.write().await;

            let requests =
                registry.counter("http_requests_total", "Total HTTP requests received")?;
            let connections =
                registry.gauge("active_connections", "Number of active connections")?;
            let latency =
                registry.histogram("request_duration_seconds", "Request latency in seconds")?;

            // Simulate some metric activity
            requests.inc();
            requests.inc_by(5);
            connections.set(42);
            latency.observe(0.042); // 42ms request
            latency.observe(0.156); // 156ms request
            latency.observe(0.5); // 500ms request
            latency.observe(2.0); // 2s request

            println!("📊 Metrics created:");
            println!("   - {} = {}", requests.name(), requests.get_counter());
            println!("   - {} = {}", connections.name(), connections.get_gauge());
            println!("   - {} (4 observations recorded)", latency.name());
            println!();
        }

        println!("🚀 Starting server...");
        println!("   Metrics:   http://127.0.0.1:9090/metrics");
        println!("   Health:    http://127.0.0.1:9090/health");
        println!("   Readiness: http://127.0.0.1:9090/ready");
        println!();
        println!("Try: curl http://127.0.0.1:9090/metrics");
        println!();

        server.run().await.map_err(|e| e.to_string())?;
    }

    #[cfg(not(all(feature = "prometheus", feature = "standalone")))]
    {
        println!("ℹ️  Required features not enabled.");
        println!("   Run with: cargo run --example standalone-prometheus --features \"prometheus standalone\"");
    }

    Ok(())
}
