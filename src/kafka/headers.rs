use base64::{Engine as _, engine::general_purpose};
use rdkafka::{
    Message,
    message::{BorrowedMessage, Headers},
};
use serde_json::{Map, Value};
use std::error::Error;

/// Converts Kafka message headers to JSON format
///
/// Returns None if the message has no headers, otherwise returns a JSON object
/// where keys are header names and values are either strings (for UTF-8 data)
/// or base64-encoded strings (for binary data)
pub fn extract_headers_as_json(msg: &BorrowedMessage<'_>) -> Result<Option<Value>, Box<dyn Error>> {
    match msg.headers() {
        Some(headers) => {
            let mut header_map = Map::new();

            for header in headers.iter() {
                let key = header.key;
                let value = match header.value {
                    Some(val) => {
                        // Try to decode as UTF-8, fallback to base64 for binary data
                        match std::str::from_utf8(val) {
                            Ok(s) => Value::String(s.to_string()),
                            Err(_) => Value::String(general_purpose::STANDARD.encode(val)),
                        }
                    }
                    None => Value::Null,
                };
                header_map.insert(key.to_string(), value);
            }

            Ok(Some(serde_json::to_value(header_map)?))
        }
        None => Ok(None),
    }
}
