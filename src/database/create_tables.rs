use crate::{
    database::CREATE_DATA_TABLE,
    database::CREATE_HWM_TABLE,
    errors::{AppError, DATABASE_CREATE_TABLE},
};
use sqlx::PgPool;
use std::error::Error;

pub async fn create_tables(pool: &PgPool) -> Result<(), AppError> {
    _create_table(&pool, &[CREATE_DATA_TABLE, CREATE_HWM_TABLE])
        .await
        .map_err(|e| AppError {
            domain: DATABASE_CREATE_TABLE.to_string(),
            value: format!("Failed to create table: {}", e),
        })
}

async fn _create_table(pool: &PgPool, statements: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut tx = pool.begin().await?;
    for stmt in statements {
        sqlx::query(stmt).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}
