use crate::database::hwm_statements::update_hwm;
use crate::database::insert_data;
use crate::kafka::headers::extract_headers_as_json;
use crate::metrics;
use chrono::{DateTime, Utc};
use log::{info, trace};
use rdkafka::Message;
use rdkafka::message::BorrowedMessage;
use sqlx::PgPool;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub headers: Option<serde_json::Value>,
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl KafkaMessage {
    pub fn from_borrowed_message(msg: BorrowedMessage<'_>) -> Result<Self, Box<dyn Error>> {
        let timestamp_millis = msg.timestamp().to_millis().unwrap_or(0);
        let timestamp = DateTime::from_timestamp_millis(timestamp_millis)
            .ok_or_else(|| format!("Invalid timestamp: {}", timestamp_millis))?;

        Ok(KafkaMessage {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            headers: extract_headers_as_json(&msg)?,
            key: msg.key().unwrap_or(&[]).to_vec(),
            payload: msg.payload().unwrap_or(&[]).to_vec(),
            timestamp,
        })
    }
}

pub async fn prosesser_melding(pg_pool: PgPool, msg: KafkaMessage) -> Result<(), Box<dyn Error>> {
    let mut tx = pg_pool.begin().await?;
    let topic = &msg.topic;

    let hwm_ok = update_hwm(&mut tx, topic, msg.partition, msg.offset).await?;

    if hwm_ok {
        let _ = insert_data::insert_data(
            &mut tx,
            topic,
            msg.partition,
            msg.offset,
            msg.timestamp,
            msg.headers,
            msg.key,
            msg.payload,
        )
        .await?;
        tx.commit().await?;

        trace!(
            "Message processed: topic={}, partition={}, offset={}",
            topic, msg.partition, msg.offset
        );
    } else {
        info!(
            "Below HWM, skipping insert: topic={}, partition={}, offset={}",
            topic, msg.partition, msg.offset
        );
        tx.rollback().await?;
    }
    metrics::increment_kafka_messages_processed(hwm_ok, topic.clone(), msg.partition);
    Ok(())
}
