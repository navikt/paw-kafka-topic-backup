use sqlx::PgPool;
use sqlx::postgres::{PgPoolOptions};
use crate::database_config::DatabaseConfig;
use crate::errors::{AppDomain, AppError};

pub fn get_pg_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    let database_url = config.full_url();
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .map_err(|_| AppError {
            domain: AppDomain::DatabaseInitialization,
            value: "Failed to create PG Pool".to_string()
        })
}

