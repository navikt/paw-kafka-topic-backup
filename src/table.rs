use std::error::Error;
use sqlx::PgPool;
use crate::errors::{AppError, AppDomain};

pub async fn create_table(pool: &PgPool) -> Result<(), AppError> {
    _create_table(&pool)
        .await
        .map_err(|e| AppError {
        domain: AppDomain::DatabaseTableCreation,
        value: format!("Failed to create table: {}", e)
    })
}

async fn _create_table(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let tx = pool.begin().await?;
    let query = r#"
        CREATE TABLE IF NOT EXISTS data_v1 (
            id BIGSERIAL PRIMARY KEY,
            kafka_topic VARCHAR(255) NOT NULL,
            kafka_partition SMALLINT NOT NULL,
            kafka_offset BIGINT NOT NULL,
            timestamp TIMESTAMP(3) WITH TIME ZONE NOT NULL,
            headers BYTEA,
            record_key BYTEA,
            record_value BYTEA,
            UNIQUE(kafka_topic, kafka_partition, kafka_offset)
        );
    "#;
    sqlx::query(query)
        .execute(pool)
        .await?;
    tx.commit().await?;
    Ok(())
}