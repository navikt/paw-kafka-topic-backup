use crate::config_utils::get_env::get_env;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use std::{error::Error, time::SystemTime};

fn get_kafka_config(
    application_kafka_config: ApplicationKafkaConfig,
) -> Result<ClientConfig, Box<dyn Error>> {
    let brokers = get_env("KAFKA_BROKERS")?;
    let kafka_private_key_path = get_env("KAFKA_PRIVATE_KEY_PATH")?;
    let kafka_certificate_path = get_env("KAFKA_CERTIFICATE_PATH")?;
    let kafka_ca_path = get_env("KAFKA_CA_PATH")?;
    let auto_commit = if application_kafka_config.auto_commit {
        "true"
    } else {
        "false"
    };
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", brokers)
        .set("group.id", application_kafka_config.group_id)
        .set("client.id", application_kafka_config.client_id)
        .set(
            "session.timeout.ms",
            application_kafka_config.session_timeout_ms.to_string(),
        )
        .set(
            "auto.offset.reset",
            application_kafka_config.auto_offset_reset,
        )
        .set("enable.auto.commit", auto_commit)
        .set(
            "security.protocol",
            application_kafka_config.security_protocol,
        )
        .set("ssl.key.location", kafka_private_key_path)
        .set("ssl.certificate.location", kafka_certificate_path)
        .set("ssl.ca.location", kafka_ca_path)
        // Ultra-aggressive memory settings using only valid rdkafka properties
        .set("fetch.max.bytes", "65536")          // 64KB max fetch
        .set("max.partition.fetch.bytes", "32768") // 32KB per partition
        .set("queued.max.messages.kbytes", "1024") // 1MB internal queue
        .set("queued.min.messages", "1")          // Min messages in queue
        .set("queued.max.messages", "100")        // Max 100 messages in queue
        .set("socket.receive.buffer.bytes", "4096") // 4KB socket receive buffer
        .set("socket.send.buffer.bytes", "4096")   // 4KB socket send buffer
        .set("fetch.min.bytes", "1")              // Don't wait for much data
        .set("fetch.wait.max.ms", "100")          // Don't wait long for data
        .set_log_level(RDKafkaLogLevel::Info);
    Ok(config)
}

#[derive(Debug, Clone)]
pub struct ApplicationKafkaConfig {
    pub group_id: String,
    pub client_id: String,
    pub auto_commit: bool,
    pub security_protocol: String,
    pub auto_offset_reset: String,
    pub session_timeout_ms: i64,
}

impl Default for ApplicationKafkaConfig {
    fn default() -> Self {
        let client_id = format!(
            "client-{}",
            unix_timestamp_millis().expect("Failed to get unix timestamp millis")
        );
        Self {
            group_id: "default-group".to_string(),
            client_id: client_id,
            auto_commit: false,
            security_protocol: "ssl".to_string(),
            auto_offset_reset: "earliest".to_string(),
            session_timeout_ms: 6000,
        }
    }
}

impl ApplicationKafkaConfig {
    pub fn new(group_id: &str, security_protocol: &str) -> Self {
        ApplicationKafkaConfig {
            group_id: group_id.to_string(),
            security_protocol: security_protocol.to_string(),
            ..Default::default()
        }
    }
    pub fn rdkafka_config(&self) -> Result<ClientConfig, Box<dyn Error>> {
        get_kafka_config(self.clone())
    }
}

fn unix_timestamp_millis() -> Result<u128, Box<dyn Error>> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| e.into())
        .map(|d| d.as_millis())
}
