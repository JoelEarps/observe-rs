# Tokio

## Spawning Tasks

This is done using a JoinSet - a joinset is a collection of tasks
The way tasks are spawned and managed are by the following:

1. JoinSet::spawn - spawns a task and passes it to the set to be executed
2. JoinSet::join_next - waits for the a task to complete and then manage the result

## Custom Error Handling with the JoinSet

What should be the

## Passing state throughout the application

# Prometheus

## Promtheus Client Library

Registry
Application State

# Kubernetes

## Readiness

## Healthiness

## Metrics

### Metric Types

Counters
Gauges

## Rust notes

release Please

Prometheus

Counters
Histograms
Labels and families

Open telemetry

Release please


rlib file

Measuring binary size




From a compiler point of view

How does this work:

// For JSON files, pre-parse size if possible, or use reasonable capacity
    #[cfg(feature = "json-config")]
    pub fn from_json_file_fast(path: impl AsRef<std::path::Path>) -> Result<Self, DeserializeError> {
        use std::fs::File;
        use std::io::BufReader;

// What is deserialisation

// How does serde implement deserialisation, how much work would it be to do this myself


Examples of registry stuff


Simple for loop, is there a better way to implement deserialise from the vector for speed?


Functionality questions:
What happens if two metrics have the same title? Should it error, overwrite, or allow duplicates?
Should initial values be validated? (e.g., negative counter values, invalid bucket ranges)
What's the expected behavior if a histogram has empty buckets array?
Should missing description be an error or default to empty string?
Error handling questions:
What should happen if JSON has extra fields not in the schema? (serde will ignore by default - is that OK?)
Should file paths be validated before attempting to read? (e.g., check if file exists first)
What error messages should users see? (detailed vs. generic)
Performance questions:
What's the expected max number of metrics? (affects HashMap capacity pre-allocation)
Should we validate config size before processing? (prevent DoS from huge configs)
Integration questions:
Should ConfiguredRegistry implement Clone or Send + Sync? (for use in async contexts)
Can metrics be accessed concurrently? (thread-safety)
Should there be a way to "export" a ConfiguredRegistry back to config format? (even without serialization, for debugging)
API design questions:
Should load_json_file accept relative or absolute paths? Both?
Should loaders validate file extensions? (.json vs .yaml)
Should there be a from_config_file() convenience method that combines load + from_config?
Documentation questions:
Are there example config files users should reference?
Should we document the exact JSON schema somewhere?
Should error types be more specific? (e.g., JsonParseError, FileNotFoundError vs generic DeserializeError)