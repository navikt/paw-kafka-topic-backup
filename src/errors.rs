use std::fmt;
use std::error::Error;

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone)]
pub struct AppError {
    pub domain: AppDomain,
    pub value: String
}

impl Error for AppError {}

#[derive(Debug, Clone)]
pub enum AppDomain {
    DatabaseConfig,
    DatabaseInitialization,
}

impl fmt::Display for AppDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppDomain::DatabaseConfig => write!(f, "Database"),
            AppDomain::DatabaseInitialization => write!(f, "Database Initialization"),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} environment variable {} is not set or has invalid value",
            self.domain,
            self.value
        )
    }
}
