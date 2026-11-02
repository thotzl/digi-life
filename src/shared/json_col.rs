//! JSON text columns (methylations, synapse weights) of the SQLite tables: encode and decode with a logged failure
//! instead of a silent default.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// JSON text of `value`; `"[]"` (empty vector column) if it cannot be serialized, with an error log naming `column`.
pub fn encode<T: Serialize + ?Sized>(value: &T, column: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|e| {
        log::error!("json column '{}': encode failed: {}", column, e);
        "[]".to_string()
    })
}

/// Value of the JSON text `text`; `None` with an error log naming `column` if it does not parse. Callers pick the
/// fallback (`unwrap_or_default()` for vectors).
pub fn decode<T: DeserializeOwned>(text: &str, column: &str) -> Option<T> {
    match serde_json::from_str(text) {
        Ok(value) => Some(value),
        Err(e) => {
            log::error!("json column '{}': decode failed: {}", column, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_and_failure() {
        let weights = vec![0.25f32, -1.5];
        assert_eq!(decode::<Vec<f32>>(&encode(&weights, "w"), "w"), Some(weights));
        assert_eq!(decode::<Vec<f32>>("not json", "w"), None);
        assert_eq!(decode::<Vec<f32>>("not json", "w").unwrap_or_default(), Vec::<f32>::new());
    }
}
