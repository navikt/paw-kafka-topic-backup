pub mod app_state;
pub mod config_utils;
pub mod database;
pub mod errors;
pub mod kafka;
pub mod logging;
pub mod nais_http_apis;

// Re-export the functions we want to test from their proper location
pub use kafka::message_processor::{lagre_melding_i_db, lagre_borrowed_message_i_db, KafkaMessage};
