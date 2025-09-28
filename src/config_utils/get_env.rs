use crate::errors::AppError;
use crate::errors::GET_ENV_VAR;

pub fn get_env(var: &str) -> Result<String, AppError> {
    let key = var;
    std::env::var(key).map_err(|_| AppError {
        domain: GET_ENV_VAR.to_string(),
        value: format!("Failed to get env var {}", var),
    })
}
