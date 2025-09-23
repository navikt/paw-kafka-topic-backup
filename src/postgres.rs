use sqlx_postgres::{PgPool, PgPoolOptions};
use crate::errors::ConfigError;

fn get_pg_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .expect("Failed to create Postgres connection pool")
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

pub fn get_database_config() -> Result<DatabaseConfig, ConfigError> {
    Ok(DatabaseConfig {
        ip: get_db_env("HOST")?,
        port: get_db_env("PORT")?.parse()
            .map_err(|_| ConfigError { variable_name: "PORT".to_string() })?,
        user: get_db_env("USER")?,
        password: get_db_env("PASSWORD")?,
        db_name: get_db_env("DATABASE")?,
        pg_ssl_cert_path: get_db_env("SSLCERT")?,
        pg_ssl_key_path: get_db_env("SSLKEY")?,
        pg_ssl_root_cert_path: get_db_env("SSLROOTCERT")?,
    })
}

fn get_db_env(var: &str) -> Result<String, ConfigError> {
    let key = format!(
        "NAIS_DATABASE_PAW_KAFKA_TOPIC_BACKUP_TOPICBACKUP_{}",
        var
    );
    std::env::var(key.clone()).map_err(|_| ConfigError { variable_name: key })
}
