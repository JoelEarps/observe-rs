//! Example: Running the observability kit as a standalone server.
//!
//! This demonstrates how to use the library in standalone mode,
//! suitable for sidecar deployments or embedded metrics servers.

use observability_kit::prelude::*;

#[tokio::main]
async fn main() {
    println!("🔭 Observability Kit - Standalone Server Example");
    println!("================================================\n");

    // Create some example metrics
    #[cfg(feature = "prometheus")]
    {
        let requests = counter("http_requests_total", "Total HTTP requests received");
        let connections = gauge("active_connections", "Number of active connections");

        // Simulate some metric activity
        requests.inc();
        requests.inc_by(5);
        connections.set(42);

        println!("📊 Metrics created:");
        println!("   - {} = {}", requests.name(), requests.get_counter());
        println!("   - {} = {}", connections.name(), connections.get_gauge());
        println!();
    }

    // Start the standalone server
    #[cfg(feature = "standalone")]
    {
        let server = StandaloneServer::builder()
            .port(9090)
            .host("127.0.0.1")
            .build();

        println!("🚀 Starting server...");
        println!("   Metrics:   http://127.0.0.1:9090/metrics");
        println!("   Health:    http://127.0.0.1:9090/health");
        println!("   Readiness: http://127.0.0.1:9090/ready");
        println!();

        if let Err(e) = server.run().await {
            eprintln!("❌ Server error: {}", e);
        }
    }

    #[cfg(not(feature = "standalone"))]
    {
        println!("ℹ️  Standalone feature not enabled.");
        println!("   Run with: cargo run --features standalone");
    }
}
