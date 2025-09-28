use sqlx::{Postgres, Transaction};

use crate::database::{INSERT_HWM, QUERY_HWM, UPDATE_HWM};

pub async fn update_hwm(
    tx: &mut Transaction<'_, Postgres>,
    topic: String,
    partition: i32,
    new_hwm: i64,
) -> Result<bool, Box<dyn std::error::Error>> {
    let result = sqlx::query(UPDATE_HWM)
        .bind(&topic)
        .bind(partition)
        .bind(new_hwm)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn insert_hwm(
    tx: &mut Transaction<'_, Postgres>,
    topic: String,
    partition: i32,
    hwm: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(INSERT_HWM)
        .bind(&topic)
        .bind(partition)
        .bind(hwm)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn get_hwm(
    tx: &mut Transaction<'_, Postgres>,
    topic: &String,
    partition: i32,
) -> Result<Option<i64>, Box<dyn std::error::Error>> {
    let hwm: Option<i64> = sqlx::query_scalar(QUERY_HWM)
        .bind(&topic)
        .bind(partition)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(hwm)
}
