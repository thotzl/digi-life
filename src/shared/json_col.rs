//! JSON text columns (methylations, synapse weights) of the SQLite tables: encode and decode with a logged failure
//! instead of a silent default.

use serde::Serialize;
use serde::de::DeserializeOwned;

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

/// JSON array of `values` with each number in its shortest f32 spelling (the value an IPC payload had when it was
/// parsed from the stored column text). A direct `json!(values)` widens f32 to f64 (0.9 -> 0.8999999761581421).
pub fn f32_values(values: &[f32]) -> serde_json::Value {
    serde_json::from_str(&encode(values, "f32 values")).unwrap_or_else(|_| serde_json::Value::Array(Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_values_keep_the_column_spelling() {
        let stored = "[0.5,0.0,0.9,-2.0]";
        let values: Vec<f32> = decode(stored, "w").unwrap();
        assert_eq!(
            f32_values(&values),
            serde_json::from_str::<serde_json::Value>(stored).unwrap()
        );
        assert_eq!(f32_values(&values).to_string(), "[0.5,0.0,0.9,-2.0]");
    }

    #[test]
    fn test_roundtrip_and_failure() {
        let weights = vec![0.25f32, -1.5];
        assert_eq!(decode::<Vec<f32>>(&encode(&weights, "w"), "w"), Some(weights));
        assert_eq!(decode::<Vec<f32>>("not json", "w"), None);
        assert_eq!(
            decode::<Vec<f32>>("not json", "w").unwrap_or_default(),
            Vec::<f32>::new()
        );
    }
}
