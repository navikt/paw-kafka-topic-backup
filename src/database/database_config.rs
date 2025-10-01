use crate::config_utils::get_env::get_env;
use crate::errors::{AppError, GET_ENV_VAR};

pub struct DatabaseConfig {
    pub ip: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub pg_ssl_cert_path: String,
    pub pg_ssl_key_path: String,
    pub pg_ssl_root_cert_path: String,
}

impl DatabaseConfig {
    pub fn full_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.ip, self.port, self.db_name
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

impl std::fmt::Display for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database connection to {}:{}/{}",
            self.ip, self.port, self.db_name
        )
    }
}

pub fn get_database_config() -> Result<DatabaseConfig, AppError> {
    Ok(DatabaseConfig {
        ip: get_db_env("HOST")?,
        port: get_db_env("PORT")?.parse().map_err(|_| AppError {
            domain: GET_ENV_VAR.to_string(),
            value: "PORT".to_string(),
        })?,
        user: get_db_env("USERNAME")?,
        password: get_db_env("PASSWORD")?,
        db_name: get_db_env("DATABASE")?,
        pg_ssl_cert_path: get_env("PGSSLCERT")?,
        pg_ssl_key_path: get_env("PGSSLKEY")?,
        pg_ssl_root_cert_path: get_env("PGSSLROOTCERT")?,
    })
}

fn get_db_env(var: &str) -> Result<String, AppError> {
    let key = format!("NAIS_DATABASE_PAW_KAFKA_TOPIC_BACKUP_TOPICBACKUPHDD_{}", var);
    std::env::var(&key).map_err(|_| AppError {
        domain: GET_ENV_VAR.to_string(),
        value: format!("Failed to get env var {}", key),
    })
}
