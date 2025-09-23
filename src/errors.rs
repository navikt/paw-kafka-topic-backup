use std::fmt;

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub variable_name: String
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Environment variable {} is not set or has invalid value", self.variable_name)
    }
}

