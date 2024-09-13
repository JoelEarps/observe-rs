mod http_server;
mod prometheus_metric_generator;
use prometheus_metric_generator::prometheus_metrics_handler::{AppState, Metrics};
use prometheus_client::registry::Registry;
use tokio::task::JoinSet;
use http_server::http_server::create_http_server;

#[tokio::main]
async fn main() {
    // Pass in config via from environment?
    println!("Hello, welcome to my library!");

    let  metrics = Metrics{
        active_connections: Default::default()
    };

    let mut state = AppState {
        registry: Registry::default(),
    };
    
    // What is Joinset by default and what should it return
    // This could be a function via an attribute? Discuss with IB
    // spawn new thread and create custom error - maybe create a new thread for it to run on automatically as a function?
    // Or leave thread management - option for threaded and none threaded management?
    let mut application_task_set = JoinSet::new();
    application_task_set.spawn({
        create_http_server(metrics, state)
    });

    // Custom error for joinset failing
    while let Some(task_return) = application_task_set.join_next().await {
        task_return.unwrap_err();
    }

    
}

// Main functions can test successful termination and running - maybe use mockall here?
// What custom errors - add to diagram?