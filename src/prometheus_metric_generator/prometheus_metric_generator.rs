use std::default;

use prometheus_client::{encoding::EncodeMetric, metrics::{counter::{self, Atomic, Counter}, gauge::Gauge}};
struct BaseMetric<T> {
    metric: T,
    title: String,
    description: String,
}

trait BasicMetricOperations<U> {
    fn new<T>(metric_name: &str, metric_description: &str) -> Self;
    fn increment_by_one(&self);
    fn increment_by_custom_value(&self, increment: U);
    fn get_metric_value(&self) -> U;
}

trait GaugeMetricFunctionality<U> : BasicMetricOperations <U> {
    fn reset_to_zero(&self);
    fn decrement_by_one(&self);
    fn decrement_by_custom_value(&self, increment: U);
}

impl BasicMetricOperations<u64> for BaseMetric<Counter>{
    fn new<Counter>(metric_name: &str, metric_description: &str) -> Self {
         BaseMetric { 
            metric: Default::default(), 
            title: metric_name.to_owned(), 
            description: metric_description.to_owned()
        }
    }
    fn increment_by_one(&self){
        println!("Do nothing for now 1");
        self.metric.inc();
    }
    fn increment_by_custom_value(&self, increment: u64){
        println!("Do nothing for now 2 with increment {}", increment);
        self.metric.inc_by(increment);
    }
    fn get_metric_value(&self) -> u64{
        println!("Do nothing for now 3");
        self.metric.get()
    }
}

impl BasicMetricOperations<i64> for BaseMetric<Gauge> {
    fn new<Gauge>(metric_name: &str, metric_description: &str) -> Self {
         BaseMetric { 
            metric: Default::default(), 
            title: metric_name.to_owned(), 
            description: metric_description.to_owned()
        }
    }
    fn increment_by_one(&self){
        println!("Do nothing for now 1");
        self.metric.inc();
    }
    fn increment_by_custom_value(&self, increment: i64){
        println!("Do nothing for now 2 with increment {}", increment);
        self.metric.inc_by(increment);
    }
    fn get_metric_value(&self) -> i64{
        println!("Do nothing for now 3");
        self.metric.get()
    }
}

impl GaugeMetricFunctionality<i64> for BaseMetric<Gauge>{
    fn reset_to_zero(&self){
        self.metric.set(0);
    }
    fn decrement_by_one(&self){
        self.metric.dec();
    }
    fn decrement_by_custom_value(&self, increment: i64){
        self.metric.dec_by(increment);
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_metric_type_counter(){
       let test_metric = BaseMetric::new::<Counter>("test_metric_counter", "A metric for declaring a counter");
        assert_eq!(test_metric.get_metric_value(), 0);
        test_metric.increment_by_one();
        assert_eq!(test_metric.get_metric_value(), 1);
        test_metric.increment_by_custom_value(20);
        assert_eq!(test_metric.get_metric_value(), 21);
    }

    #[test]
     fn test_metric_type_gauge(){
        let test_metric_gauge = BaseMetric::new::<Gauge>("test_metric_counter", "A metric for declaring a counter");
        assert_eq!(test_metric_gauge.get_metric_value(), 0);
        test_metric_gauge.increment_by_one();
        assert_eq!(test_metric_gauge.get_metric_value(), 1);
        test_metric_gauge.increment_by_custom_value(20);
        assert_eq!(test_metric_gauge.get_metric_value(), 21);
        test_metric_gauge.decrement_by_one();
        assert_eq!(test_metric_gauge.get_metric_value(), 20);
        test_metric_gauge.decrement_by_custom_value(10);
        assert_eq!(test_metric_gauge.get_metric_value(), 10);
        test_metric_gauge.reset_to_zero();
        assert_eq!(test_metric_gauge.get_metric_value(), 0);
    }
}
