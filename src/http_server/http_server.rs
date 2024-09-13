
use axum::extract::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper::StatusCode;
use tokio::net::TcpListener;
use axum::{
     routing::get, Router
};
use prometheus_client::{encoding::text::encode, metrics::gauge::Gauge, registry::Registry};
// Is this the correct wat to do this use crate - or should we do this at mod level
use crate::prometheus_metric_generator::prometheus_metrics_handler::{AppState, Metrics};

// Define custom error here for http service depending on failure?
async fn root() -> &'static str {
    "Hello, World!"
}

async fn ready_check(State(metrics): State<Arc<Mutex<Metrics>>>)-> (StatusCode, String) {
    metrics.lock().await.inc_active_connections();
    (StatusCode::OK, format!("{}", "OK"))
}

async fn health_handler() -> (StatusCode, String) {
    // Check the current application status - what is best practice for metric servers
    (StatusCode::OK, format!("{}", "OK"))
}

async fn prometheus_metrics(State(state): State<Arc<Mutex<AppState>>>) -> String {
    let current_state = state.lock().await;
    let mut buffer = String::new();
    encode(&mut buffer, &current_state.registry).unwrap();
    buffer
}

// potential constructor for management of monitoring threading - maybe return a JoinSet for others


// This should be tested with integration tests
// Create custom error for here
// State here should be a metric
pub async fn create_http_server(metrics: Metrics, mut state: AppState) -> Result<(), ()> {

    state
        .registry
        .register("Active Connections", "Active number of connections between all clients and all servers", metrics.active_connections.clone());
    let metrics = Arc::new(Mutex::new(metrics));
    let state = Arc::new(Mutex::new(state));


    let app = Router::new()
        .route("/", get(root))
        .route("/ready", get(ready_check)).with_state(metrics)
        .route("/health", 
        get(health_handler)).route("/metrics", get(prometheus_metrics)).with_state(state);

    let listener = TcpListener::bind("localhost:3000").await.map_err(|error| {
        return ()
      })?;
      
    let _server_result = axum::serve(listener, app).await.map_err(|error| {
        return ()
      })?;

    Ok(())
}

pub fn say_hello() {
    println!("Hello from http");
}

#[cfg(test)]
mod tests {
    // use axum::extract::Request;
    use super::*;

    #[tokio::test]
    async fn test_hello_world(){
        assert_eq!(root().await, "Hello, World!");
    }

    #[tokio::test]
    async fn test_health_handler(){
        let metrics = Metrics {
        active_connections: Default::default()
        };
        let metrics = Arc::new(Mutex::new(metrics));
        let cloned_metrics = metrics.clone();
        let result = ready_check(State(metrics)).await;
        println!("{:?}", result);
        println!("{:?}", cloned_metrics.lock().await.active_connections);
    }

    #[tokio::test]
    async fn test_ready_handler(){
        assert_eq!(health_handler().await, (StatusCode::OK, format!("{}", "OK")));
    }
}
