use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct AppError {
    pub domain: String,
    pub value: String
}

pub const GET_ENV_VAR: &str = "get_env_var";
pub const DATABASE_CONNECTION: &str = "database_connection";
pub const DATABASE_CREATE_TABLE: &str = "database_create_table";


impl Error for AppError {}


impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} :: {}",
            self.domain,
            self.value
        )
    }
}
