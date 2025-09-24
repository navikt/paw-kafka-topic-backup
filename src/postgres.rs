use sqlx::PgPool;
use sqlx::postgres::{PgPoolOptions};
use crate::database_config::DatabaseConfig;
use crate::errors::{DATABASE_CONNECTION, AppError};

pub async fn get_pg_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    let database_url = config.full_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .map_err(|_| AppError {
            domain: DATABASE_CONNECTION.to_string(),
            value: "Failed to create PG Pool".to_string()
        })?;
    let _ = sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| AppError {
            domain: DATABASE_CONNECTION.to_string(),
            value: format!("Failed to run 'SELECT 1', connection not ok: {}", e)
        })?;
    Ok(pool)
}

