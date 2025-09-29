use prometheus::{CounterVec, register_counter_vec};
use std::sync::OnceLock;

/// Counter for total Kafka messages processed with above_hwm label
static KAFKA_MESSAGES_PROCESSED: OnceLock<CounterVec> = OnceLock::new();

/// Initialize all Prometheus metrics
/// This should be called once at application startup
/// Safe to call multiple times - will only initialize once
pub fn init_metrics() {
    KAFKA_MESSAGES_PROCESSED.get_or_init(|| {
        register_counter_vec!(
            "kafka_messages_processed_total",
            "Total number of Kafka messages processed",
            &["above_hwm"]
        ).expect("Failed to register kafka_messages_processed_total counter")
    });
}

/// Increment the counter for processed Kafka messages
/// above_hwm: true if message was above HWM and processed, false if skipped
pub fn increment_kafka_messages_processed(above_hwm: bool) {
    if let Some(counter_vec) = KAFKA_MESSAGES_PROCESSED.get() {
        counter_vec
            .with_label_values(&[&above_hwm.to_string()])
            .inc();
    }
}

/// Get the current count of processed Kafka messages
/// Useful for testing or debugging
pub fn get_kafka_messages_processed_count(above_hwm: bool) -> f64 {
    KAFKA_MESSAGES_PROCESSED
        .get()
        .map(|counter_vec| {
            counter_vec
                .with_label_values(&[&above_hwm.to_string()])
                .get()
        })
        .unwrap_or(0.0)
}