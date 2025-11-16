use crate::ort_value::OrtValue;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// Convert OrtValue to serde_json::Value
impl From<OrtValue> for JsonValue {
    fn from(ort: OrtValue) -> Self {
        match ort {
            OrtValue::Null => JsonValue::Null,
            OrtValue::Bool(b) => JsonValue::Bool(b),
            OrtValue::Number(n) => {
                if let Some(num) = serde_json::Number::from_f64(n) {
                    JsonValue::Number(num)
                } else {
                    JsonValue::Null
                }
            }
            OrtValue::String(s) => JsonValue::String(s),
            OrtValue::Array(arr) => {
                let json_arr: Vec<JsonValue> = arr.into_iter().map(|v| v.into()).collect();
                JsonValue::Array(json_arr)
            }
            OrtValue::Object(obj) => {
                let json_obj: serde_json::Map<String, JsonValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect();
                JsonValue::Object(json_obj)
            }
        }
    }
}

// Convert serde_json::Value to OrtValue
impl From<JsonValue> for OrtValue {
    fn from(json: JsonValue) -> Self {
        match json {
            JsonValue::Null => OrtValue::Null,
            JsonValue::Bool(b) => OrtValue::Bool(b),
            JsonValue::Number(n) => OrtValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => OrtValue::String(s),
            JsonValue::Array(arr) => {
                let ort_arr: Vec<OrtValue> = arr.into_iter().map(|v| v.into()).collect();
                OrtValue::Array(ort_arr)
            }
            JsonValue::Object(obj) => {
                let ort_obj: HashMap<String, OrtValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect();
                OrtValue::Object(ort_obj)
            }
        }
    }
}

// Reference conversions
impl From<&JsonValue> for OrtValue {
    fn from(json: &JsonValue) -> Self {
        match json {
            JsonValue::Null => OrtValue::Null,
            JsonValue::Bool(b) => OrtValue::Bool(*b),
            JsonValue::Number(n) => OrtValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => OrtValue::String(s.clone()),
            JsonValue::Array(arr) => {
                let ort_arr: Vec<OrtValue> = arr.iter().map(|v| v.into()).collect();
                OrtValue::Array(ort_arr)
            }
            JsonValue::Object(obj) => {
                let ort_obj: HashMap<String, OrtValue> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), v.into()))
                    .collect();
                OrtValue::Object(ort_obj)
            }
        }
    }
}
