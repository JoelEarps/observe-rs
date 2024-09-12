mod http_server;
mod prometheus_metric_generator;
use http_server::http_server::say_hello;
use prometheus_metric_generator::prometheus_metric_generator::say_hello_two;

fn main() {
    println!("Hello, world!");
    say_hello();
    say_hello_two();
}
