use sqlx::PgPool;
use std::error::Error;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

use paw_kafka_topic_backup::database::hwm_statements::{get_hwm, insert_hwm, update_hwm};

async fn setup_test_db() -> Result<(PgPool, ContainerAsync<Postgres>), Box<dyn Error>> {
    let postgres_container = Postgres::default().start().await;

    let host_port = postgres_container.get_host_port_ipv4(5432).await;
    let connection_string = format!(
        "postgresql://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    // Set environment variables for testing (wrapped in unsafe blocks)
    unsafe {
        std::env::set_var("DATABASE_URL", &connection_string);
        std::env::set_var("PG_HOST", "127.0.0.1");
        std::env::set_var("PG_PORT", host_port.to_string());
        std::env::set_var("PG_USERNAME", "postgres");
        std::env::set_var("PG_PASSWORD", "postgres");
        std::env::set_var("PG_DATABASE_NAME", "postgres");
    }

    // Create connection pool and tables manually since init_db might not work in tests
    let pool = PgPool::connect(&connection_string).await?;

    // Create tables manually - need to import the SQL constant
    sqlx::query(paw_kafka_topic_backup::database::sqls::CREATE_HWM_TABLE)
        .execute(&pool)
        .await?;

    Ok((pool, postgres_container))
}

#[tokio::test]
async fn test_insert_hwm_data() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Start a transaction
    let mut tx = pool.begin().await.expect("Failed to start transaction");

    // Insert test data using hwm_statements function
    let topic = "test-topic".to_string();
    let partition = 0i32;
    let hwm = 100i64;

    let result = insert_hwm(&mut tx, &topic, partition, hwm).await;
    assert!(
        result.is_ok(),
        "Failed to insert HWM data: {:?}",
        result.err()
    );

    // Commit the transaction
    tx.commit().await.expect("Failed to commit transaction");

    // Verify the data was inserted by querying in a new transaction
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let retrieved_hwm = get_hwm(&mut tx, &topic, partition)
        .await
        .expect("Failed to query HWM data");
    tx.commit().await.expect("Failed to commit transaction");

    assert!(retrieved_hwm.is_some(), "HWM should exist");
    assert_eq!(retrieved_hwm.unwrap(), hwm);
}

#[tokio::test]
async fn test_get_nonexistent_hwm() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Query non-existent data using hwm_statements function
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let result = get_hwm(&mut tx, &"nonexistent-topic".to_string(), 999i32)
        .await
        .expect("Failed to query HWM data");
    tx.commit().await.expect("Failed to commit transaction");

    assert!(result.is_none(), "Non-existent HWM should return None");
}

#[tokio::test]
async fn test_update_hwm_data() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    let topic = "update-test-topic".to_string();
    let partition = 2i32;
    let initial_hwm = 100i64;
    let updated_hwm = 200i64;

    // Insert initial data
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    insert_hwm(&mut tx, &topic, partition, initial_hwm)
        .await
        .expect("Failed to insert initial test data");
    tx.commit().await.expect("Failed to commit transaction");

    // Update the HWM using hwm_statements function (should succeed because new_hwm > old_hwm)
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let update_result = update_hwm(&mut tx, &topic, partition, updated_hwm).await;
    tx.commit().await.expect("Failed to commit transaction");

    assert!(
        update_result.is_ok(),
        "Failed to update HWM: {:?}",
        update_result.err()
    );
    assert_eq!(
        update_result.unwrap(),
        true,
        "Update should return true when rows are affected"
    );

    // Verify the update
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let actual_hwm = get_hwm(&mut tx, &topic, partition)
        .await
        .expect("Failed to query updated HWM");
    tx.commit().await.expect("Failed to commit transaction");

    assert_eq!(actual_hwm.unwrap(), updated_hwm);
}

#[tokio::test]
async fn test_update_hwm_no_change_when_lower() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    let topic = "no-update-test-topic".to_string();
    let partition = 3i32;
    let initial_hwm = 200i64;
    let lower_hwm = 100i64;

    // Insert initial data
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    insert_hwm(&mut tx, &topic, partition, initial_hwm)
        .await
        .expect("Failed to insert initial test data");
    tx.commit().await.expect("Failed to commit transaction");

    // Try to update with lower HWM using hwm_statements function
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let update_result = update_hwm(&mut tx, &topic, partition, lower_hwm).await;
    tx.commit().await.expect("Failed to commit transaction");

    assert!(update_result.is_ok());
    assert_eq!(
        update_result.unwrap(),
        false,
        "Should return false when no rows are affected"
    );

    // Verify the HWM remains unchanged
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let actual_hwm = get_hwm(&mut tx, &topic, partition)
        .await
        .expect("Failed to query HWM after attempted update");
    tx.commit().await.expect("Failed to commit transaction");

    assert_eq!(
        actual_hwm.unwrap(),
        initial_hwm,
        "HWM should remain unchanged"
    );
}

#[tokio::test]
async fn test_multiple_topics_and_partitions() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    // Insert data for multiple topics and partitions using hwm_statements functions
    let test_data = vec![
        ("topic1".to_string(), 0, 100i64),
        ("topic1".to_string(), 1, 150i64),
        ("topic2".to_string(), 0, 200i64),
        ("topic2".to_string(), 1, 250i64),
    ];

    for (topic, partition, hwm) in &test_data {
        let mut tx = pool.begin().await.expect("Failed to start transaction");
        insert_hwm(&mut tx, &topic, *partition, *hwm)
            .await
            .expect("Failed to insert test data");
        tx.commit().await.expect("Failed to commit transaction");
    }

    // Verify all data was inserted correctly using hwm_statements functions
    for (topic, partition, expected_hwm) in test_data {
        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let actual_hwm = get_hwm(&mut tx, &topic, partition)
            .await
            .expect("Failed to query HWM");
        tx.commit().await.expect("Failed to commit transaction");

        assert_eq!(
            actual_hwm.unwrap(),
            expected_hwm,
            "HWM mismatch for topic {} partition {}",
            topic,
            partition
        );
    }
}

#[tokio::test]
async fn test_transaction_rollback() {
    let (pool, _container) = setup_test_db()
        .await
        .expect("Failed to setup test database");

    let topic = "rollback-test-topic".to_string();
    let partition = 0i32;
    let hwm = 100i64;

    // Start a transaction and insert data but don't commit
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    insert_hwm(&mut tx, &topic, partition, hwm)
        .await
        .expect("Failed to insert test data");

    // Rollback the transaction instead of committing
    tx.rollback().await.expect("Failed to rollback transaction");

    // Verify the data was not persisted
    let mut tx = pool.begin().await.expect("Failed to start transaction");
    let result = get_hwm(&mut tx, &topic, partition)
        .await
        .expect("Failed to query HWM data");
    tx.commit().await.expect("Failed to commit transaction");

    assert!(result.is_none(), "HWM should not exist after rollback");
}
