use crate::errors::{DATABASE_CONFIG, AppError};

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
            self.user,
            self.password,
            self.ip,
            self.port,
            self.db_name
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
            .map_err(|_| AppError { domain: DATABASE_CONFIG.to_string(), value: "PORT".to_string() })?,
        user: get_db_env("USERNAME")?,
        password: get_db_env("PASSWORD")?,
        db_name: get_db_env("DATABASE")?,
        pg_ssl_cert_path: get_env("PGSSLCERT")?,
        pg_ssl_key_path: get_env("PGSSLKEY")?,
        pg_ssl_root_cert_path: get_env("PGSSLROOTCERT")?,
    })
}

fn get_db_env(var: &str) -> Result<String, AppError> {
    let key = format!(
        "NAIS_DATABASE_PAW_KAFKA_TOPIC_BACKUP_TOPICBACKUP_{}",
        var
    );
    std::env::var(key).map_err(|_| AppError {
        domain: DATABASE_CONFIG.to_string(),
        value: var.to_string()
    } )
}

fn get_env(var: &str) -> Result<String, AppError> {
    let key = var;
    std::env::var(key).map_err(|_| AppError {
        domain: DATABASE_CONFIG.to_string(),
        value: var.to_string()
    } )
}
