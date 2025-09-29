use sqlx::PgPool;
use std::error::Error;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use chrono::DateTime;
use std::sync::Mutex;

// Import modules from the main crate
use paw_kafka_topic_backup::{lagre_melding_i_db, KafkaMessage};
use paw_kafka_topic_backup::database::hwm_statements::insert_hwm;
use paw_kafka_topic_backup::metrics::{init_metrics, get_kafka_messages_processed_count};

// Ensure metrics tests run serially to avoid registry conflicts
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Setup a test database container
async fn setup_test_db() -> Result<(PgPool, ContainerAsync<Postgres>), Box<dyn Error>> {
    let postgres_container = Postgres::default()
        .start()
        .await;
    
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
    sqlx::query(paw_kafka_topic_backup::database::sqls::CREATE_HWM_TABLE)
        .execute(&pool)
        .await?;
    
    sqlx::query(paw_kafka_topic_backup::database::sqls::CREATE_DATA_TABLE)
        .execute(&pool)
        .await?;

    Ok((pool, postgres_container))
}

// Helper function to create a mock KafkaMessage for testing
fn create_test_kafka_message(topic: &str, partition: i32, offset: i64) -> KafkaMessage {
    let timestamp = DateTime::from_timestamp_millis(1234567890000)
        .expect("Valid timestamp");
    
    KafkaMessage {
        topic: topic.to_string(),
        partition,
        offset,
        headers: Some(serde_json::json!({"test": "header", "source": "metrics-test"})),
        key: format!("test-key-{}", offset).into_bytes(),
        payload: format!(r#"{{"message": "test payload", "offset": {}}}"#, offset).into_bytes(),
        timestamp,
    }
}

#[tokio::test]
async fn test_kafka_messages_processed_counter() {
    let _lock = TEST_LOCK.lock().unwrap();
    
    // Initialize metrics for this test
    init_metrics();
    
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Insert initial HWM record with a lower offset so our test messages will be processed
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let _ = insert_hwm(&mut tx, "metrics-test-topic".to_string(), 0, 50).await
        .expect("Failed to insert initial HWM");
    tx.commit().await.expect("Failed to commit initial HWM");

    // Record initial counter values (both above_hwm=true and above_hwm=false)
    let initial_count_above = get_kafka_messages_processed_count(true);
    let initial_count_below = get_kafka_messages_processed_count(false);
    
    // Process first message
    let test_message_1 = create_test_kafka_message("metrics-test-topic", 0, 100);
    lagre_melding_i_db(pool.clone(), test_message_1).await
        .expect("First message should be processed successfully");

    // Check that above_hwm counter increased by 1, below_hwm unchanged
    let count_above_after_first = get_kafka_messages_processed_count(true);
    let count_below_after_first = get_kafka_messages_processed_count(false);
    assert_eq!(count_above_after_first - initial_count_above, 1.0, "above_hwm counter should increase by 1 after first message");
    assert_eq!(count_below_after_first - initial_count_below, 0.0, "below_hwm counter should remain unchanged");

    // Process second message
    let test_message_2 = create_test_kafka_message("metrics-test-topic", 0, 200);
    lagre_melding_i_db(pool.clone(), test_message_2).await
        .expect("Second message should be processed successfully");

    // Check that above_hwm counter increased by 2 total, below_hwm still unchanged
    let count_above_after_second = get_kafka_messages_processed_count(true);
    let count_below_after_second = get_kafka_messages_processed_count(false);
    assert_eq!(count_above_after_second - initial_count_above, 2.0, "above_hwm counter should increase by 2 after second message");
    assert_eq!(count_below_after_second - initial_count_below, 0.0, "below_hwm counter should remain unchanged");
}

#[tokio::test]
async fn test_counter_not_incremented_for_skipped_messages() {
    let _lock = TEST_LOCK.lock().unwrap();
    
    // Initialize metrics for this test
    init_metrics();
    
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Insert initial HWM record with the SAME offset as our test message
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let _ = insert_hwm(&mut tx, "skip-test-topic".to_string(), 0, 100).await
        .expect("Failed to insert initial HWM");
    tx.commit().await.expect("Failed to commit initial HWM");

    // Record initial counter values
    let initial_count_above = get_kafka_messages_processed_count(true);
    let initial_count_below = get_kafka_messages_processed_count(false);

    // Try to process a message with the same offset (should be skipped)
    let test_message = create_test_kafka_message("skip-test-topic", 0, 100);
    lagre_melding_i_db(pool.clone(), test_message).await
        .expect("Message processing should succeed even when skipped");

    // Check that above_hwm counter did NOT increase but below_hwm counter DID increase
    let count_above_after_skip = get_kafka_messages_processed_count(true);
    let count_below_after_skip = get_kafka_messages_processed_count(false);
    assert_eq!(count_above_after_skip - initial_count_above, 0.0, "above_hwm counter should not increase for skipped messages");
    assert_eq!(count_below_after_skip - initial_count_below, 1.0, "below_hwm counter should increase for skipped messages");
}