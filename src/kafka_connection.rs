use std::error::Error;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{Consumer, StreamConsumer};
use crate::database_config::get_env;

pub fn create_kafka_consumer(group_id: &str, topics: &[&str], auto_commit: bool) -> Result<StreamConsumer, Box<dyn Error>> {
    let brokers = get_env("KAFKA_BROKERS")?;
    let kafka_private_key_path = get_env("KAFKA_PRIVATE_KEY_PATH")?;
    let kafka_certificate_path = get_env("KAFKA_CERTIFICATE_PATH")?;
    let kafka_ca_path = get_env("KAFKA_CA_PATH")?;
    let auto_commit = if auto_commit { "true" } else { "false" };
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("client.id", "client-123")
        .set("session.timeout.ms", "6000")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", auto_commit)
        .set("security.protocol", "ssl")
        .set("ssl.key.location", kafka_private_key_path)
        .set("ssl.certificate.location", kafka_certificate_path)
        .set("ssl.ca.location", kafka_ca_path.clone())
        .set_log_level(RDKafkaLogLevel::Info);
    let consumer: StreamConsumer = config.create()?;
    consumer.subscribe(topics)?;
    Ok(consumer)
}
