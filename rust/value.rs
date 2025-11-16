use crate::{parse_ort, generate_ort, OrtResult, OrtValue};
use std::fs;
use std::path::Path;

/// Parse ORT string into an OrtValue
///
/// # Example
/// ```
/// let ort_str = "users:id,name:\n1,John\n2,Jane";
/// let value = ort::from_str(ort_str)?;
/// ```
pub fn from_str(s: &str) -> OrtResult<OrtValue> {
    parse_ort(s)
}

/// Parse ORT file into an OrtValue
///
/// # Example
/// ```
/// let value = ort::from_file("data.ort")?;
/// let name = value["users"][0]["name"].as_str().unwrap();
/// ```
pub fn from_file<P: AsRef<Path>>(path: P) -> OrtResult<OrtValue> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| crate::error::OrtError {
            line: 0,
            code: String::new(),
            message: format!("Failed to read file: {}", e),
        })?;
    parse_ort(&content)
}

/// Convert an OrtValue to ORT string
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use ort::OrtValue;
///
/// let mut obj = HashMap::new();
/// obj.insert("id".to_string(), OrtValue::from(1));
/// obj.insert("name".to_string(), OrtValue::from("John"));
/// let ort_str = ort::to_string(&OrtValue::Object(obj));
/// ```
pub fn to_string(value: &OrtValue) -> String {
    generate_ort(value)
}

/// Convert an OrtValue to ORT string and write to file
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use ort::OrtValue;
///
/// let mut obj = HashMap::new();
/// obj.insert("id".to_string(), OrtValue::from(1));
/// let value = OrtValue::Object(obj);
/// ort::to_file(&value, "output.ort")?;
/// ```
pub fn to_file<P: AsRef<Path>>(value: &OrtValue, path: P) -> OrtResult<()> {
    let ort_string = generate_ort(value);
    fs::write(path.as_ref(), ort_string)
        .map_err(|e| crate::error::OrtError {
            line: 0,
            code: String::new(),
            message: format!("Failed to write file: {}", e),
        })
}
