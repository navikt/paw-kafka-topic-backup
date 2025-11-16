use serde::Deserialize;
use serde_env_field::env_field_wrap;

#[env_field_wrap]
#[derive(Deserialize)]
pub struct Config {
    pub topics: Vec<String>,
}
