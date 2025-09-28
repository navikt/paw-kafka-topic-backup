use crate::database::create_tables;
use crate::database::database_config::{DatabaseConfig, get_database_config};
use crate::errors::{AppError, DATABASE_CONNECTION};
use log::info;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

async fn get_pg_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    let database_url = config.full_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .map_err(|_| AppError {
            domain: DATABASE_CONNECTION.to_string(),
            value: "Failed to create PG Pool".to_string(),
        })?;
    let _ = sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| AppError {
            domain: DATABASE_CONNECTION.to_string(),
            value: format!("Failed to run 'SELECT 1', connection not ok: {}", e),
        })?;
    Ok(pool)
}

pub async fn init_db() -> Result<PgPool, Box<dyn Error>> {
    let db_config = get_database_config()?;
    info!("Database config: {:?}", db_config);
    let pg_pool = get_pg_pool(&db_config).await?;
    info!("Postgres pool opprettet");
    let _ = create_tables(&pg_pool).await?;
    Ok(pg_pool)
}
