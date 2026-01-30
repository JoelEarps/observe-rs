# Observability Kit

A flexible, multi-backend observability library for Rust applications.

[![Crates.io](https://img.shields.io/crates/v/observability-kit.svg)](https://crates.io/crates/observability-kit)
[![Documentation](https://docs.rs/observability-kit/badge.svg)](https://docs.rs/observability-kit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🔌 **Multi-backend support** — Prometheus (more backends coming soon)
- 🎛️ **Feature-gated** — Only compile what you need
- 🚀 **Standalone server** — Built-in HTTP server for `/metrics`, `/health`, `/ready`
- 🏷️ **Labeled metrics** — Full support for dimensional metrics
- 🧪 **Test utilities** — Mock backend for easy unit testing

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
observability-kit = "0.1"
```

### Standalone Server (Sidecar/Embedded)

Perfect for sidecar deployments or embedded metrics servers:

```rust
use observability_kit::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a standalone server with Prometheus backend
    let server = StandaloneServer::<PrometheusBackend>::builder()
        .port(9090)
        .host("0.0.0.0")
        .build();

    // Create metrics via the registry
    let registry_handle = server.registry();
    let mut registry = registry_handle.write().await;
    
    let requests = registry.counter("http_requests_total", "Total HTTP requests")?;
    let latency = registry.histogram("request_duration_seconds", "Request latency")?;
    
    // Use the metrics
    requests.inc();
    latency.observe(0.042);
    
    drop(registry); // Release the lock before running

    // Start the server
    // Endpoints: /metrics, /health, /ready
    server.run().await?;
    Ok(())
}
```

### Basic Metrics (Without Server)

For simple metric creation without the HTTP server:

```rust
use observability_kit::prelude::*;

// Counters - monotonically increasing values
let requests = counter("http_requests_total", "Total HTTP requests");
requests.inc();
requests.inc_by(5);
println!("Total requests: {}", requests.get_counter()); // 6

// Gauges - values that can go up and down
let connections = gauge("active_connections", "Active connections");
connections.set(42);
connections.inc();
connections.dec();
println!("Connections: {}", connections.get_gauge()); // 42

// Histograms - distributions of values
let latency = histogram_for_latency("request_duration_seconds", "Request latency");
latency.observe(0.042);  // 42ms
latency.observe(0.156);  // 156ms
```

### Labeled Metrics

For dimensional metrics with labels:

```rust
use observability_kit::prelude::*;

// Define your label structure
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct HttpLabels {
    method: String,
    status: u16,
    endpoint: String,
}

// Create a labeled histogram family
let latency: LabeledHistogram<HttpLabels> = labeled_histogram_for_latency();

// Record metrics with specific label values
latency.get_or_create(&HttpLabels {
    method: "GET".into(),
    status: 200,
    endpoint: "/api/users".into(),
}).observe(0.042);

latency.get_or_create(&HttpLabels {
    method: "POST".into(),
    status: 201,
    endpoint: "/api/users".into(),
}).observe(0.156);

// Labeled counters
let requests: LabeledCounter<HttpLabels> = labeled_counter();
requests.get_or_create(&HttpLabels {
    method: "GET".into(),
    status: 200,
    endpoint: "/api/users".into(),
}).inc();
```

### Testing with Mock Backend

The mock backend provides easy testing without a real metrics system:

```rust
use observability_kit::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_counting() {
        let counter = test_counter("test_requests", "For testing");
        
        // Simulate your code that increments the counter
        counter.inc();
        counter.inc_by(5);
        
        // Assert on the value
        assert_eq!(counter.get_counter(), 6);
    }

    #[test]
    fn test_histogram_observations() {
        let histogram = test_histogram("test_latency", "For testing");
        
        histogram.observe(0.1);
        histogram.observe(0.2);
        histogram.observe(0.3);
        
        // Mock histogram tracks all observations
        assert_eq!(histogram.observation_count(), 3);
        assert!((histogram.observation_sum() - 0.6).abs() < 0.001);
    }
}
```

## Config file path handling (JSON/YAML)

When using `load_file`, `validate_file_path`, `load_json_file`, or `load_yaml_file` (with the `json-config` / `yaml-config` features), path handling is **restricted for security**:

- **Allowed directories** — The path must resolve to a file under one of:
  - `$XDG_CONFIG_HOME` (or `$HOME/.config` if unset)
  - Current working directory
  - An extra base you pass, e.g. `load_file("metrics.json", Some(project_root))`
- **No symlinks** — The path and none of its ancestors may be symlinks.
- **Supported extensions** — Only `.json`, `.yaml`, and `.yml` are accepted.
- **File must exist** — The path must point to an existing regular file (not a directory).

Use **`load_file(path, extra_base)`** to validate and then deserialize in one call (returns `RegistryConfig` when the matching feature is enabled). Use **`validate_file_path(path, extra_base)`** if you only need the validated path.

*Note:* Do not load config from untrusted paths; no resource limits (file size, metric count) are enforced.

## Metric names and duplicates (need to know)

### Metric names (Prometheus backend)

When using the Prometheus backend, metric names are **validated at registration**. Names must follow [Prometheus rules](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels):

- **Pattern:** `[a-zA-Z_][a-zA-Z0-9_]*` — start with a letter or underscore; after that, only letters, digits, and underscores.
- **Non-empty** — empty names are rejected.
- **Not allowed:** colons (`:`), hyphens (`-`), dots (`.`), or any other characters. Digits are allowed only after the first character.

Invalid names (e.g. `my-metric`, `my.metric`, `name_with:colon`, `123bad`) cause registration to return `PrometheusError::InvalidNamingConvention`. Use valid names such as `http_requests_total`, `request_duration_seconds`, `_private_metric`.

### Duplicate names and number of metrics

- **Same type, same name** — Registering two metrics of the **same type** (e.g. two counters) with the **same name** is not allowed. It returns `DuplicateMetricName` (deserialised config) or a backend error. Each metric name must be unique within its type.
- **Same name, different types** — One counter, one gauge, and one histogram can all share the same name (e.g. `metric`). That is allowed.
- **No hard limit** — The library does not enforce a maximum number of metrics. Very large registries may affect memory and scrape size; keep cardinality in mind for Prometheus.

## Histogram Presets

Pre-configured bucket sets for common use cases:

| Function | Buckets | Use Case |
| ---------- | --------- | ---------- |
| `histogram()` | `[0.001, 0.01, 0.1, 1, 10, 100, 1000]` | General purpose |
| `histogram_for_latency()` | `[5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s]` | HTTP/API latency |
| `histogram_for_bytes()` | `[100B, 1KB, 10KB, 100KB, 1MB, 10MB, 100MB, 1GB, 10GB, 100GB]` | Response/payload sizes |
| `histogram_with_buckets(buckets)` | Custom | Your own bucket boundaries |

## Feature Flags

| Feature | Description | Default |
| --------- | ------------- | --------- |
| `prometheus` | Prometheus metrics backend | ✅ |
| `standalone` | Standalone HTTP server | ✅ |
| `mock` | Mock backend for testing | |
| `json-config` | JSON configuration support | |
| `yaml-config` | YAML configuration support | |
| `full` | All features | |

## Build Size Comparison

| Feature Combination | Description | Binary Size | Size (KB) | Relative to Minimal |
| --------------------- | ------------- | ------------- | ----------- | --------------------- |
| `prometheus` | Minimal (prometheus only) | 177 KB | 177 KB | 0 KB (baseline) |
| `prometheus,standalone` | Default (prometheus + standalone) | 1.63 MB | 1677 KB | +1500 KB |
| `prometheus,mock` | Prometheus + mock (testing) (lib only) | 229 KB | 229 KB | +52 KB |
| `prometheus,standalone,json-config` | Prometheus + standalone + JSON config | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,yaml-config` | Prometheus + standalone + YAML config | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,json-config,yaml-config` | Prometheus + standalone + all config formats | 1.63 MB | 1676 KB | +1499 KB |
| `prometheus,standalone,axum-integration` | Prometheus + standalone + Axum integration | 1.63 MB | 1677 KB | +1500 KB |
| `prometheus,otlp` | Prometheus + OpenTelemetry (lib only) | 177 KB | 177 KB | 0 KB (baseline) |
| `prometheus,otlp,standalone` | Prometheus + OpenTelemetry + standalone | 1.65 MB | 1694 KB | +1517 KB |
| `full` | Full (all features) (lib only) | 327 KB | 327 KB | +150 KB |

### Minimal Build

For the smallest binary size:

```toml
[dependencies]
observability-kit = { version = "0.1", default-features = false, features = ["prometheus"] }
```

### Full Build

For all features:

```toml
[dependencies]
observability-kit = { version = "0.1", features = ["full"] }
```

## Running the Example

```bash
# Run the standalone server example
cargo run --example standalone-prometheus --features "prometheus standalone"

# In another terminal:
curl http://127.0.0.1:9090/metrics
curl http://127.0.0.1:9090/health
curl http://127.0.0.1:9090/ready
```

## Running Tests

```bash
# Run all tests
cargo test --features mock

# Run with all features
cargo test --features full
```

## Roadmap

- [ ] OpenTelemetry/OTLP backend
- [ ] Axum middleware integration  
- [ ] Actix middleware integration
- [ ] JSON/YAML configuration
- [ ] Fake data generator for testing

## License

MIT License - see [LICENSE](LICENSE) for details.
