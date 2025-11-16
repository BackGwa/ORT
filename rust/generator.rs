use crate::ort_value::OrtValue;
use std::collections::HashMap;

pub fn generate_ort(value: &OrtValue) -> String {
    match value {
        OrtValue::Object(obj) => {
            // Check if this is a multi-key object
            if obj.len() > 1 || obj.is_empty() {
                generate_multi_object(obj)
            } else if obj.len() == 1 {
                // Single key - might be a named array
                let (key, val) = obj.iter().next().unwrap();
                if let OrtValue::Array(arr) = val {
                    if arr.is_empty() {
                        format!("{}:\n[]", key)
                    } else if is_uniform_object_array(arr) {
                        generate_object_array(key, arr)
                    } else {
                        generate_simple_array(key, arr)
                    }
                } else {
                    // Single key with non-array value
                    format!("{}:\n{}", key, generate_value(val, false))
                }
            } else {
                String::new()
            }
        }
        OrtValue::Array(arr) => {
            // Top-level array
            if is_uniform_object_array(arr) {
                generate_top_level_object_array(arr)
            } else {
                format!(":{}", generate_array_content(arr, false))
            }
        }
        _ => generate_value(value, false),
    }
}

fn generate_multi_object(obj: &HashMap<String, OrtValue>) -> String {
    let mut result = Vec::new();
    let mut entries: Vec<_> = obj.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    for (i, (key, val)) in entries.iter().enumerate() {
        if let OrtValue::Array(arr) = val {
            if is_uniform_object_array(arr) {
                result.push(generate_object_array(key, arr).trim_end().to_string());
            } else {
                result.push(generate_simple_array(key, arr).trim_end().to_string());
            }
        } else {
            result.push(format!("{}:\n{}", key, generate_value(val, false)));
        }

        if i < entries.len() - 1 {
            result.push("\n\n".to_string());
        } else {
            result.push("\n".to_string());
        }
    }

    result.join("")
}

fn is_uniform_object_array(arr: &[OrtValue]) -> bool {
    if arr.is_empty() {
        return false;
    }

    // Check if all elements are objects with the same keys
    let first_keys = match &arr[0] {
        OrtValue::Object(obj) => {
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            keys
        }
        _ => return false,
    };

    for item in arr.iter().skip(1) {
        match item {
            OrtValue::Object(obj) => {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                if keys != first_keys {
                    return false;
                }
            }
            _ => return false,
        }
    }

    true
}

fn generate_object_array(key: &str, arr: &[OrtValue]) -> String {
    if arr.is_empty() {
        return format!("{}:\n[]", key);
    }

    let first = &arr[0];
    if let OrtValue::Object(obj) = first {
        let keys: Vec<_> = obj.keys().cloned().collect();
        let header = generate_header(&keys, obj);

        let mut result = vec![format!("{}:{}", key, header)];

        for item in arr {
            if let OrtValue::Object(obj) = item {
                let values: Vec<String> = keys
                    .iter()
                    .map(|k| generate_object_field_value(obj.get(k).unwrap_or(&OrtValue::Null), &keys, k, obj))
                    .collect();
                result.push(values.join(","));
            }
        }

        result.join("\n")
    } else {
        format!("{}:\n{}", key, generate_array_content(arr, false))
    }
}

fn generate_top_level_object_array(arr: &[OrtValue]) -> String {
    if arr.is_empty() {
        return ":[]".to_string();
    }

    let first = &arr[0];
    if let OrtValue::Object(obj) = first {
        let keys: Vec<_> = obj.keys().cloned().collect();
        let header = generate_header(&keys, obj);

        let mut result = vec![format!(":{}", header)];

        for item in arr {
            if let OrtValue::Object(obj) = item {
                let values: Vec<String> = keys
                    .iter()
                    .map(|k| generate_object_field_value(obj.get(k).unwrap_or(&OrtValue::Null), &keys, k, obj))
                    .collect();
                result.push(values.join(","));
            }
        }

        result.join("\n")
    } else {
        format!(":{}", generate_array_content(arr, false))
    }
}

fn generate_header(keys: &[String], first_obj: &HashMap<String, OrtValue>) -> String {
    keys.iter()
        .map(|k| {
            if let Some(value) = first_obj.get(k) {
                match value {
                    OrtValue::Object(nested_obj) => {
                        // Generate nested field
                        let nested_keys: Vec<_> = nested_obj.keys().cloned().collect();
                        let nested_header = generate_header_fields(&nested_keys, nested_obj);
                        format!("{}({})", k, nested_header)
                    }
                    _ => k.clone(),
                }
            } else {
                k.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",") + ":"
}

fn generate_header_fields(keys: &[String], obj: &HashMap<String, OrtValue>) -> String {
    keys.iter()
        .map(|k| {
            if let Some(value) = obj.get(k) {
                match value {
                    OrtValue::Object(nested_obj) => {
                        // Recursively generate nested field
                        let nested_keys: Vec<_> = nested_obj.keys().cloned().collect();
                        let nested_header = generate_header_fields(&nested_keys, nested_obj);
                        format!("{}({})", k, nested_header)
                    }
                    _ => k.clone(),
                }
            } else {
                k.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn generate_object_field_value(
    value: &OrtValue,
    _keys: &[String],
    _current_key: &str,
    _parent: &HashMap<String, OrtValue>,
) -> String {
    match value {
        OrtValue::Null => String::new(),
        OrtValue::Object(obj) => {
            if obj.is_empty() {
                "()".to_string()
            } else {
                // Nested object - output values only (keys are in header)
                let nested_keys: Vec<_> = obj.keys().cloned().collect();
                let values: Vec<String> = nested_keys
                    .iter()
                    .map(|k| {
                        generate_object_field_value(
                            obj.get(k).unwrap_or(&OrtValue::Null),
                            &nested_keys,
                            k,
                            obj,
                        )
                    })
                    .collect();
                format!("({})", values.join(","))
            }
        }
        OrtValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", generate_array_content(arr, true))
            }
        }
        _ => generate_value(value, true),
    }
}

fn generate_simple_array(key: &str, arr: &[OrtValue]) -> String {
    format!("{}:\n{}", key, generate_array_content(arr, false))
}

fn generate_array_content(arr: &[OrtValue], inline: bool) -> String {
    if arr.is_empty() {
        return "[]".to_string();
    }

    let values: Vec<String> = arr.iter().map(|v| generate_value(v, inline)).collect();

    if inline {
        values.join(",")
    } else {
        format!("[{}]", values.join(","))
    }
}

fn generate_value(value: &OrtValue, _inline: bool) -> String {
    match value {
        OrtValue::Null => String::new(),
        OrtValue::Bool(b) => b.to_string(),
        OrtValue::Number(n) => n.to_string(),
        OrtValue::String(s) => escape(s),
        OrtValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", generate_array_content(arr, true))
            }
        }
        OrtValue::Object(obj) => {
            if obj.is_empty() {
                "()".to_string()
            } else {
                generate_inline_object(obj)
            }
        }
    }
}

fn generate_inline_object(obj: &HashMap<String, OrtValue>) -> String {
    let pairs: Vec<String> = obj
        .iter()
        .map(|(k, v)| format!("{}:{}", k, generate_value(v, true)))
        .collect();
    format!("({})", pairs.join(","))
}

fn escape(s: &str) -> String {
    let mut result = String::new();

    for ch in s.chars() {
        match ch {
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '[' => result.push_str("\\["),
            ']' => result.push_str("\\]"),
            ',' => result.push_str("\\,"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            _ => result.push(ch),
        }
    }

    result
}
