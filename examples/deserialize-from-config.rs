//! Example: Creating a registry from a JSON configuration file.
//!
//! This demonstrates how to use the deserialization module to create
//! a configured registry from a JSON file.
//!
//! Run with:
//! ```bash
//! cargo run --example deserialize-from-config --features "prometheus json-config"
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔭 Observability Kit - Deserialize from Config Example");
    println!("=====================================================\n");

    #[cfg(all(feature = "prometheus", feature = "json-config"))]
    {
        use observability_kit::backends::prometheus::PrometheusBackend;
        use observability_kit::prelude::{
            ConfiguredRegistry, DeserializeError, load_json_file, load_json_str,
        };
        // Option 1: Load from a file
        println!("📄 Loading configuration from rough_input.json...");
        let config = load_json_file("rough_input.json")?;
        let configured = ConfiguredRegistry::<PrometheusBackend>::from_config(config)?;

        println!("✅ Successfully loaded {} counters", configured.counters.len());
        println!("✅ Successfully loaded {} gauges", configured.gauges.len());
        println!("✅ Successfully loaded {} histograms\n", configured.histograms.len());

        // Access and use metrics by name
        if let Some(counter) = configured.counters.get("M_title") {
            println!("Found counter 'M_title':");
            println!("  Current value: {}", counter.get_counter());
            counter.inc();
            println!("  After increment: {}\n", counter.get_counter());
        }

        if let Some(gauge) = configured.gauges.get("M_title") {
            println!("Found gauge 'M_title':");
            println!("  Current value: {}", gauge.get_gauge());
            gauge.set(100);
            println!("  After setting to 100: {}\n", gauge.get_gauge());
        }

        // Option 2: Load from a string
        println!("📝 Loading configuration from string...");
        let json_str = r#"[
            {
                "metric_type": "Counter",
                "title": "http_requests_total",
                "description": "Total HTTP requests",
                "value": 0
            },
            {
                "metric_type": "Gauge",
                "title": "active_connections",
                "description": "Active connections",
                "value": 0
            },
            {
                "metric_type": "Histogram",
                "title": "request_duration_seconds",
                "description": "Request latency",
                "buckets": [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
            }
        ]"#;

        let config2 = load_json_str(json_str)?;
        let configured2 = ConfiguredRegistry::<PrometheusBackend>::from_config(config2)?;

        println!("✅ Successfully loaded configuration from string");
        println!("   Counters: {}, Gauges: {}, Histograms: {}\n",
            configured2.counters.len(),
            configured2.gauges.len(),
            configured2.histograms.len()
        );

        // Use the metrics
        if let Some(requests) = configured2.counters.get("http_requests_total") {
            requests.inc();
            requests.inc_by(5);
        }

        if let Some(connections) = configured2.gauges.get("active_connections") {
            connections.set(42);
        }

        if let Some(latency) = configured2.histograms.get("request_duration_seconds") {
            latency.observe(0.042);
            latency.observe(0.156);
        }

        // Render the metrics
        println!("📊 Rendered metrics:");
        let output = configured2.registry.render()
            .map_err(|e| DeserializeError::BackendError(e.to_string()))?;
        let output_str = output.as_str()
            .map_err(|e| DeserializeError::BackendError(format!("UTF-8 error: {}", e)))?;
        println!("{}", output_str);
    }

    #[cfg(not(all(feature = "prometheus", feature = "json-config")))]
    {
        println!("ℹ️  Required features not enabled.");
        println!("   Run with: cargo run --example deserialize-from-config --features \"prometheus json-config\"");
    }

    Ok(())
}
