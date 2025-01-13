use prometheus_client::{metrics::gauge::Gauge, registry::Registry};

// Lets talk through private values
// Purpose of this calls
/*
1. trigger Generator and assign all vals to the registry 
2. Create Mutexes for state and registry
3. Allow access to the registry and perform actions
*/


#[derive(Debug)]
pub struct RegistryState {
    pub registry: Registry,
}
#[derive(Debug)]
pub struct Metrics {
    pub active_connections: Gauge<i64>,
}

impl Metrics {
    pub fn inc_active_connections(&self, increment: i64) {
        self.active_connections.inc_by(increment);
    }
}

pub(crate) struct PrometheusMetricHandler {
    pub all_metrics: Metrics,
    pub registry_state: RegistryState
}

impl PrometheusMetricHandler {
    pub fn new()-> Self {
        // pass in metrics desired for metrics and the generator can add them to the registry
        PrometheusMetricHandler {
            all_metrics: Metrics { active_connections: Default::default() },
            registry_state: RegistryState { registry: Default::default() }
        }
    }

    fn create_mutexes(){
        
    }

    // Access and lock for thread safety? Getter for Registry?

    // Setter for metrics?  How should we reference and store variables - Hash Map? Dictionary?

}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_metric_handler_is_valid(){
        let test_metrics = Metrics{
            active_connections: Default::default(),
        };
        let test_default = test_metrics.active_connections.get();
        assert_eq!(test_default, 0);
    }
}