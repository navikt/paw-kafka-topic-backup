use chrono::DateTime;
use sqlx::PgPool;
use std::error::Error;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

// Import modules from the main crate
use paw_kafka_topic_backup::database::hwm_statements::{get_hwm, insert_hwm};
use paw_kafka_topic_backup::{KafkaMessage, prosesser_melding};

/// Setup a test database container
async fn setup_test_db() -> Result<(PgPool, ContainerAsync<Postgres>), Box<dyn Error>> {
    let postgres_container = Postgres::default().start().await;

    let host_port = postgres_container.get_host_port_ipv4(5432).await;
    let connection_string = format!(
        "postgresql://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    // Set environment variables for testing
    unsafe {
        std::env::set_var("DATABASE_URL", &connection_string);
        std::env::set_var("PG_HOST", "127.0.0.1");
        std::env::set_var("PG_PORT", host_port.to_string());
        std::env::set_var("PG_USERNAME", "postgres");
        std::env::set_var("PG_PASSWORD", "postgres");
        std::env::set_var("PG_DATABASE_NAME", "postgres");
    }

    let pool = PgPool::connect(&connection_string).await?;

    // Create necessary tables
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok((pool, postgres_container))
}

// Helper function to create a mock KafkaMessage for testing
fn create_test_kafka_message(topic: &str, partition: i32, offset: i64) -> KafkaMessage {
    let timestamp = DateTime::from_timestamp_millis(1234567890000).expect("Valid timestamp");

    KafkaMessage {
        topic: topic.to_string(),
        partition,
        offset,
        headers: Some(serde_json::json!({"test": "header", "source": "integration-test"})),
        key: format!("test-key-{}", offset).into_bytes(),
        payload: format!(r#"{{"message": "test payload", "offset": {}}}"#, offset).into_bytes(),
        timestamp,
    }
}

#[tokio::test]
async fn test_lagre_melding_i_db_new_message() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Insert initial HWM record with a lower offset so our test message will be processed
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let _ = insert_hwm(&mut tx, "test-topic", 0, 50)
        .await
        .expect("Failed to insert initial HWM");
    tx.commit().await.expect("Failed to commit initial HWM");

    // Create a test message with offset higher than HWM
    let test_message = create_test_kafka_message("test-topic", 0, 100);

    // Use the actual function to process the message
    prosesser_melding(pool.clone(), test_message)
        .await
        .expect("lagre_melding_i_db should succeed");

    // Verify the HWM was updated to the new offset
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let hwm = get_hwm(&mut tx, "test-topic", 0)
        .await
        .expect("Failed to get HWM");
    assert_eq!(hwm, Some(100), "HWM should be updated to new offset");

    // Verify data was inserted
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM data_v2 WHERE kafka_topic = $1 AND kafka_partition = $2",
    )
    .bind("test-topic")
    .bind(0i32)
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to count rows");

    tx.commit().await.expect("Failed to commit");

    assert_eq!(count.0, 1, "Should have inserted 1 data record");
}

#[tokio::test]
async fn test_lagre_melding_i_db_duplicate_message() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Insert initial HWM record with the SAME offset as our test message
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let _ = insert_hwm(&mut tx, "test-topic", 0, 100)
        .await
        .expect("Failed to insert initial HWM");
    tx.commit().await.expect("Failed to commit initial HWM");

    // Create a test message with the same offset (should be skipped)
    let test_message = create_test_kafka_message("test-topic", 0, 100);

    // Use the actual function to process the duplicate message
    prosesser_melding(pool.clone(), test_message)
        .await
        .expect("lagre_melding_i_db should succeed even for duplicates");

    // Verify no additional data was inserted
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM data_v2 WHERE kafka_topic = $1 AND kafka_partition = $2",
    )
    .bind("test-topic")
    .bind(0i32)
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to count rows");

    tx.commit().await.expect("Failed to commit");

    assert_eq!(
        count.0, 0,
        "Should not have inserted any data records for duplicate offset"
    );
}

#[tokio::test]
async fn test_lagre_melding_i_db_lower_offset_message() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    let higher_hwm = 150i64;

    // Insert initial HWM record with a HIGHER offset than our test message
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let _ = insert_hwm(&mut tx, "test-topic", 0, higher_hwm)
        .await
        .expect("Failed to insert initial HWM");
    tx.commit().await.expect("Failed to commit initial HWM");

    // Create a test message with lower offset (should be skipped)
    let test_message = create_test_kafka_message("test-topic", 0, 100);

    // Use the actual function to process the lower offset message
    prosesser_melding(pool.clone(), test_message)
        .await
        .expect("lagre_melding_i_db should succeed even for lower offsets");

    // Verify HWM remains unchanged
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let hwm = get_hwm(&mut tx, "test-topic", 0)
        .await
        .expect("Failed to get HWM");
    assert_eq!(hwm, Some(higher_hwm), "HWM should remain unchanged");

    // Verify no data was inserted
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM data_v2 WHERE kafka_topic = $1 AND kafka_partition = $2",
    )
    .bind("test-topic")
    .bind(0i32)
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to count rows");

    tx.commit().await.expect("Failed to commit");

    assert_eq!(
        count.0, 0,
        "Should not have inserted any data records for lower offset"
    );
}
