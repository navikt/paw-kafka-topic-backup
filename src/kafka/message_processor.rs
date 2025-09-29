use crate::database::hwm_statements::update_hwm;
use crate::database::insert_data;
use crate::kafka::headers::extract_headers_as_json;
use log::info;
use rdkafka::Message;
use rdkafka::message::BorrowedMessage;
use sqlx::PgPool;
use std::error::Error;
use chrono::{DateTime, Utc};

/// Represents a Kafka message with owned data
/// 
/// This struct owns all the message data, making it safe to pass
/// across async boundaries and store for processing.
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub headers: Option<serde_json::Value>,
    pub key: Vec<u8>,        // Vec<u8> for owned data
    pub payload: Vec<u8>,    // Vec<u8> for owned data
    pub timestamp: DateTime<Utc>,  // Proper timestamp type
}

impl KafkaMessage {
    /// Convert a BorrowedMessage to an owned KafkaMessage
    /// 
    /// This performs the conversion from &[u8] (borrowed) to Vec<u8> (owned)
    /// for the key and payload data, and from millis to DateTime<Utc>.
    pub fn from_borrowed_message(msg: BorrowedMessage<'_>) -> Result<Self, Box<dyn Error>> {
        let timestamp_millis = msg.timestamp().to_millis().unwrap_or(0);
        let timestamp = DateTime::from_timestamp_millis(timestamp_millis)
            .ok_or_else(|| format!("Invalid timestamp: {}", timestamp_millis))?;
            
        Ok(KafkaMessage {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            headers: extract_headers_as_json(&msg)?,
            key: msg.key().unwrap_or(&[]).to_vec(),        // &[u8] -> Vec<u8>
            payload: msg.payload().unwrap_or(&[]).to_vec(), // &[u8] -> Vec<u8>
            timestamp,
        })
    }
}

pub async fn lagre_melding_i_db(
    pg_pool: PgPool,
    msg: KafkaMessage,
) -> Result<(), Box<dyn Error>> {
    let mut tx = pg_pool.begin().await?;
    
    // Borrow the topic string to avoid move issues
    let topic = &msg.topic;
    
    let hwm_ok = update_hwm(
        &mut tx,
        msg.topic.clone(),  // Clone for update_hwm which needs owned String
        msg.partition,
        msg.offset,
    )
    .await?;
    
    if hwm_ok {
        let _ = insert_data::insert_data(
            &mut tx,
            topic,           // &String -> &str (auto deref)
            msg.partition,
            msg.offset,
            msg.timestamp,
            msg.headers,
            msg.key,         // Vec<u8>
            msg.payload,     // Vec<u8>
        )
        .await?;
        tx.commit().await?;
    } else {
        info!(
            "Below HWM, skipping insert: topic={}, partition={}, offset={}",
            topic,
            msg.partition,
            msg.offset
        );
        tx.rollback().await?;
    }
    Ok(())
}

/// Convenience function to process a BorrowedMessage directly
/// 
/// This function converts the BorrowedMessage to an owned KafkaMessage
/// and then processes it. Use this when working with rdkafka consumers.
pub async fn lagre_borrowed_message_i_db(
    pg_pool: PgPool,
    msg: BorrowedMessage<'_>,
) -> Result<(), Box<dyn Error>> {
    let kafka_msg = KafkaMessage::from_borrowed_message(msg)?;
    lagre_melding_i_db(pg_pool, kafka_msg).await
}