use serde::Deserialize;
use serde_env_field::env_field_wrap;

#[env_field_wrap]
#[derive(Deserialize)]
pub struct Config {
    pub topics: Vec<String>,
}

impl Config {
    pub fn from_string(file_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = toml::from_str(file_content)?;
        Ok(config)
    }

    pub fn from_default_file() -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = include_str!("../config/config.toml");
        Self::from_string(&file_content)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_config_parsing_content() {
        temp_env::with_var("PROD_TOPIC", Some("topic_from_env"), || {
            let toml_content = r#"
            topics = ["topic1", "$PROD_TOPIC", "topic3"]
        "#;
            let config = Config::from_string(toml_content).unwrap();
            assert_eq!(
                config.topics,
                vec![
                    "topic1".to_string(),
                    "topic_from_env".to_string(),
                    "topic3".to_string()
                ]
            );
        });
    }

    #[test]
    fn test_config_parsing_file() {
        temp_env::with_var("PAA_VEGNE_AV_TOPIC", Some("topic_from_env"), || {
            let config = Config::from_default_file().unwrap();
            assert_eq!(
                config.topics,
                vec![
                    "paw.arbeidssoker-hendelseslogg-v1".to_string(),
                    "paw.arbeidssoker-bekreftelse-v1".to_string(),
                    "topic_from_env".to_string()
                ]
            );
        });
    }
}
