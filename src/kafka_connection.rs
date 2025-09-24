use std::error::Error;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{Consumer, StreamConsumer};
use crate::database_config::get_env;

pub fn create_kafka_consumer(group_id: &str, topics: &[&str], auto_commit: bool) -> Result<StreamConsumer, Box<dyn Error>> {
    let brokers = get_env("KAFKA_BROKERS")?;
    let truststore_path = get_env("KAFKA_TRUSTSTORE_PATH")?;
    let truststore_password = get_env("KAFKA_CREDSTORE_PASSWORD")?;
    let keystore_path = get_env("KAFKA_KEYSTORE_PATH")?;
    let auto_commit = if auto_commit { "true" } else { "false" };
    let mut config = ClientConfig::new();
    config
        .set("security.protocol", "ssl")
        .set("ssl.truststore.type", "JKS")
        .set("ssl.truststore.location", &truststore_path)
        .set("ssl.truststore.password", &truststore_password)
        .set("ssl.keystore.location", &keystore_path)
        .set("ssl.keystore.password", &truststore_password)
        .set("ssl.endpoint.identification.algorithm", "")
        .set("enable.auto.commit", auto_commit)
        .set("auto.offset.reset", "earliest")
        .set("bootstrap.servers", &brokers)
        .set("group.id", group_id)
        .set("max.poll.records", "100")
        .set_log_level(RDKafkaLogLevel::Info);
    let consumer: StreamConsumer = config.create()?;
    consumer.subscribe(topics)?;
    Ok(consumer)
}
