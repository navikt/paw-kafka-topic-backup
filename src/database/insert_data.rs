use serde_json::Value;
use sqlx::Postgres;
use sqlx::Transaction;
use chrono::{DateTime, Utc};

use crate::database::INSERT_DATA;

pub async fn insert_data(
    tx: &mut Transaction<'_, Postgres>,
    kafka_topic: &str,
    kafka_partition: i32,
    kafka_offset: i64,
    timestamp: DateTime<Utc>,
    headers: Option<Value>,
    record_key: Vec<u8>,
    record_value: Vec<u8>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(INSERT_DATA)
        .bind(kafka_topic)
        .bind(kafka_partition)
        .bind(kafka_offset)
        .bind(timestamp)
        .bind(headers)
        .bind(record_key)
        .bind(record_value)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected())
}
