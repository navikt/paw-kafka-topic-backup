use sqlx_postgres::{PgPool, PgPoolOptions};
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

pub struct DatabaseConfig {
    ip: String,
    port: u16,
    user: String,
    password: String,
    db_name: String,
    pg_ssl_cert_path: String,
    pg_ssl_key_path: String,
    pg_ssl_root_cert_path: String,
}

impl DatabaseConfig {
    pub fn full_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode=verify-full&sslcert={}&sslkey={}&sslrootcert={}",
            self.user,
            self.password,
            self.ip,
            self.port,
            self.db_name,
            self.pg_ssl_cert_path,
            self.pg_ssl_key_path,
            self.pg_ssl_root_cert_path
        )
    }
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .field("username", &self.user)
            .field("password", &"********")
            .field("db_name", &self.db_name)
            .field("pg_ssl_cert_path", &self.pg_ssl_cert_path)
            .field("pg_ssl_key_path", &self.pg_ssl_key_path)
            .field("pg_ssl_root_cert_path", &self.pg_ssl_root_cert_path)
            .finish()
    }
}

pub fn get_database_config() -> Result<DatabaseConfig, AppError> {
    Ok(DatabaseConfig {
        ip: get_db_env("HOST")?,
        port: get_db_env("PORT")?.parse()
            .map_err(|_| AppError { domain: AppDomain::DatabaseConfig, value: "PORT".to_string() })?,
        user: get_db_env("USERNAME")?,
        password: get_db_env("PASSWORD")?,
        db_name: get_db_env("DATABASE")?,
        pg_ssl_cert_path: get_db_env("SSLCERT")?,
        pg_ssl_key_path: get_db_env("SSLKEY")?,
        pg_ssl_root_cert_path: get_db_env("SSLROOTCERT")?,
    })
}

fn get_db_env(var: &str) -> Result<String, AppError> {
    let key = format!(
        "NAIS_DATABASE_PAW_KAFKA_TOPIC_BACKUP_TOPICBACKUP_{}",
        var
    );
    std::env::var(key).map_err(|_| AppError {
        domain: AppDomain::DatabaseConfig,
        value: var.to_string()
    } )
}
