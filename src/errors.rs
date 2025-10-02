use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
    pub domain: String,
    pub value: String,
}

pub const GET_ENV_VAR: &str = "get_env_var";
pub const DATABASE_CONNECTION: &str = "database_connection";

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :: {}", self.domain, self.value)
    }
}
