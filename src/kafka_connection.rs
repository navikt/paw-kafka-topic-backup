use std::error::Error;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use crate::database_config::get_env;

pub fn create_kafka_consumer(group_id: &str, topics: &[&str]) -> Result<StreamConsumer, Box<dyn Error>> {
    let brokers = get_env("KAFKA_BROKERS")?;
    let cert_path = get_env("KAFKA_CERTIFICATE_PATH")?;
    let key_path = get_env("KAFKA_KEY_PATH")?;
    let ca_path = get_env("KAFKA_CA_PATH")?;

    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", &brokers)
        .set("group.id", group_id)
        .set("security.protocol", "SSL")
        .set("ssl.certificate.location", &cert_path)
        .set("ssl.key.location", &key_path)
        .set("ssl.ca.location", &ca_path)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("auto.offset.reset", "earliest");

    let consumer: StreamConsumer = config.create()?;
    consumer.subscribe(topics)?;
    Ok(consumer)
}
