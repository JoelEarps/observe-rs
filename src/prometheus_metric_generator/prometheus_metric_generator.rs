use std::default;

use prometheus_client::{encoding::EncodeMetric, metrics::{counter::{self, Atomic, Counter}, gauge::Gauge}};

/*
Input = vector of metrics structs

Aim: Generate vector types, perform prometheus logic required and set up access control
*/

// This would mean that all Metrics would technically need to check if they could perform that operation - as they could be one of several types
// And the functions may cause undefined behaviour

// #[derive(Debug)]
// enum MetricType {
//     WrappedCounter(Counter),
//     WrappedGauge(Gauge)
// }

// Question - do we need custom error handling where there is no implementation for a type

struct BaseMetric<T, U> {
    metric_type: T,
    title: String,
    description: String,
    value: U
}

trait BasicMetricOperations<U> {
    fn new<T>(metric_name: &str, metric_description: &str) -> Self;
    fn increment_by_one(&self);
    fn increment_by_custom_value(&self, increment: U);
    fn get_value(&self) -> U;
}

trait GaugeMetricFunctionality<U> : BasicMetricOperations <U> {
    fn reset_to_zero(&self);
    fn decrement_by_one(&self);
    fn decrement_by_custom_value(&self, increment: U);
}

impl BasicMetricOperations<Counter> for BaseMetric<Counter, u64>{
    fn new<Counter>(metric_name: &str, metric_description: &str) -> BaseMetric<Counter, u64> {
        BaseMetric<Counter, u64> {
             title: metric_name.to_owned(),
            metric: Default::default(),
            description: metric_description.to_owned(),
            value: 0 as u64
        }
    }
    fn increment_by_one(&self){
        println!("Do nothing for now 1");
    }
    fn increment_by_custom_value(&self, increment: i64){
        println!("Do nothing for now 2 with increment {}", increment);
    }
    fn get_value(&self) -> i64{
        println!("Do nothing for now 3");
        64
    }
}

// impl Default for MetricType {
//     fn default() -> Self {
//         MetricType::WrappedCounter(
//             Default::default()
//         )
//     }
// }


// impl MetricType {
//     fn new_metric(input: SupportedMetrics) -> MetricType{
//         match input {
//             SupportedMetrics::counter => {
//                 println!("Creating Counter");
//                 MetricType::WrappedCounter(Default::default())
//             },
//             SupportedMetrics::gauge => {
//                 println!("Creating Gauge");
//                 MetricType::WrappedGauge(Default::default())
//             },
//         }
//     }

//     fn increment_by_one(&self){
//         match self {
//             MetricType::WrappedCounter(counter) => {
//                 counter.inc();
//             }
//             // Throw Custom Error here for being unable to cast, better way to do this?
//             MetricType::WrappedGauge(gauge) => {
//                 gauge.inc().try_into().unwrap_or(0);
//             }
//         } 
//     }

//     fn increment_by_custom_value(&self, increment: u64){
//         match self {
//             MetricType::WrappedCounter(counter) => {
//                 counter.inc_by(increment);
//             }
//             MetricType::WrappedGauge(gauge) => {
//                 // Throw Custom Error here for being unable to cast, better way to do this?
//                 gauge.inc_by(increment as i64).try_into().unwrap_or(0);
//             }
//         } 
//     }

//     fn get_value(&self) -> u64 {
//         match self {
//             MetricType::WrappedCounter(counter) => counter.get(),
//             // Throw Custom Error here for being unable to cast, better way to do this?
//             MetricType::WrappedGauge(gauge) => gauge.get().try_into().unwrap_or(0)
//         } 
//     }

//     fn reset_to_zero(&self) -> (){
//         match self {
//             MetricType::WrappedGauge(gauge) => {
//                 gauge.set(0);
//             }
//             MetricType::WrappedCounter(_) => {
//                 // custom error returned here for nothing able to reset a counter
//                 println!("Invalid action for this metric type");
//             }
//         }
//     }
    
//     fn decrement_by_one(&self)-> (){
//         match self {
//             MetricType::WrappedGauge(gauge) => {
//                     gauge.dec();
//             }
//             MetricType::WrappedCounter(_) => {
//                     // custom error returned here for nothing able to reset a counter
//                     println!("Invalid action for this metric type");
//             }
//         }
//     }

//     fn decrement_by_custom_value(&self, decrement: i64){
//        match self {
//             MetricType::WrappedGauge(gauge) => {
//                     gauge.dec_by(decrement);
//             }
//             MetricType::WrappedCounter(_) => {
//                     // custom error returned here for nothing able to reset a counter
//                     println!("Invalid action for this metric type");
//             }
//         }
//     }
// }


// // Can you use derive here instead rather than use the function methods we have created
// struct Metric {
//     metric_type: MetricType,
//     title: String,
//     description: String,
//     value: u64
// }


// trait BasicMetricOperations {
//     fn increment_by_one(&self) -> ();
//     fn increment_by_custom_value(&self, increment: u64) -> ();
//     fn get_metric_value(&self) -> u64;
// }


// impl BasicMetricOperations for Metric {
//     // When first implementing we could not implement the increment function as this does not allow us to find the increment function as the Metric type doesn't
//     // Imply the right functions
//     fn increment_by_one(&self) -> (){
//         println!("Incrementing {:?} by 1", self.title);
//         self.metric_type.increment_by_one();
//     }
//     fn increment_by_custom_value(&self, increment: u64) -> (){
//         println!("Incrementing {:?} by {}", self.title, increment);
//         self.metric_type.increment_by_custom_value(increment);
//     }

//     fn get_metric_value(&self) -> u64 {
//         println!("Getting value for {}", self.title);
//         self.metric_type.get_value()
//     }
// }


// trait GaugeFunctions {
//     fn decrement_by_one(&self) -> ();
//     fn decrement_by_custom_value(&self, decrement: i64 ) -> ();
//     fn reset_to_zero(&self) -> ();
// }

// impl GaugeFunctions for Metric {
//     fn decrement_by_one(&self) -> () {
//         // Check this is a not a counter and only allow for non counters
//         // Custom Error?
//         println!("Decrementing {:?} by 1", self.title);
//         self.metric_type.decrement_by_one();
//     }

//     fn reset_to_zero(&self) -> () {
//         println!("Resetting Metric of {:?} to 0", self.title);
//         self.metric_type.reset_to_zero();
//     }

//     fn decrement_by_custom_value(&self, decrement: i64 ) -> (){
//         println!("Decrementing {:?} by {}", self.title, decrement);
//         self.metric_type.decrement_by_custom_value(decrement);

//     }
// }


// // Test Scenarios
// /*
// 1. Create Counter
//     a. Increment by one and assert on value
//     b. Increment by custom val and assert by value
// 2. Create Gauge
//     a. Increment by one and assert on value
//     b. Increment by custom val and assert by value
//     c. Decrement by one and assert on value
//     d. Decrement by custom val and assert by value
// 3. Create array of types based on dict input and then implement a find function to perform the actions 
//  */
// #[cfg(test)]
// mod tests{
//     use super::*;

//     #[test]
//     fn test_metric_type_counter(){
//         let test_metric = Metric{
//             metric_type: MetricType::new_metric(SupportedMetrics::counter),
//             title: "Test Counter".to_string(),
//             description: ("This is a test Counter").to_string(),
//             value : 0
//         };
//         assert_eq!(test_metric.get_metric_value(), 0);
//         test_metric.increment_by_one();
//         assert_eq!(test_metric.get_metric_value(), 1);
//         test_metric.increment_by_custom_value(20);
//         assert_eq!(test_metric.get_metric_value(), 21);
//         // Should return a failure message
//         // TODO: Custom Error message for incompatible actions for types
//         test_metric.reset_to_zero();
//     }

//     #[test]
//      fn test_metric_type_gauge(){
//         let test_metric_gauge = Metric{
//             metric_type: MetricType::new_metric(SupportedMetrics::gauge),
//             title: "Test Gauge".to_string(),
//             description: ("This is a test Counter").to_string(),
//             value : 0
//         };
//         println!("Test Value: {:?}", test_metric_gauge.metric_type);
//          assert_eq!(test_metric_gauge.get_metric_value(), 0);
//         test_metric_gauge.increment_by_one();
//         assert_eq!(test_metric_gauge.get_metric_value(), 1);
//         test_metric_gauge.increment_by_custom_value(20);
//         assert_eq!(test_metric_gauge.get_metric_value(), 21);
//         test_metric_gauge.decrement_by_one();
//         assert_eq!(test_metric_gauge.get_metric_value(), 20);
//         test_metric_gauge.decrement_by_custom_value(10);
//         assert_eq!(test_metric_gauge.get_metric_value(), 10);
//         test_metric_gauge.reset_to_zero();
//         assert_eq!(test_metric_gauge.get_metric_value(), 0);
    
//     }
// }
