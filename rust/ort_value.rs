use std::collections::HashMap;
use std::ops::Index;
use std::fmt;

/// ORT native value type
#[derive(Debug, Clone, PartialEq)]
pub enum OrtValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<OrtValue>),
    Object(HashMap<String, OrtValue>),
}

impl OrtValue {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, OrtValue::Null)
    }

    /// Check if value is boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, OrtValue::Bool(_))
    }

    /// Check if value is number
    pub fn is_number(&self) -> bool {
        matches!(self, OrtValue::Number(_))
    }

    /// Check if value is string
    pub fn is_string(&self) -> bool {
        matches!(self, OrtValue::String(_))
    }

    /// Check if value is array
    pub fn is_array(&self) -> bool {
        matches!(self, OrtValue::Array(_))
    }

    /// Check if value is object
    pub fn is_object(&self) -> bool {
        matches!(self, OrtValue::Object(_))
    }

    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OrtValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            OrtValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as i64
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            OrtValue::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    /// Get as u64
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            OrtValue::Number(n) if *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    /// Get as string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            OrtValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get as array reference
    pub fn as_array(&self) -> Option<&Vec<OrtValue>> {
        match self {
            OrtValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as mutable array reference
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<OrtValue>> {
        match self {
            OrtValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as object reference
    pub fn as_object(&self) -> Option<&HashMap<String, OrtValue>> {
        match self {
            OrtValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get as mutable object reference
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, OrtValue>> {
        match self {
            OrtValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get value by key (for objects)
    pub fn get(&self, key: &str) -> Option<&OrtValue> {
        match self {
            OrtValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    /// Get mutable value by key (for objects)
    pub fn get_mut(&mut self, key: &str) -> Option<&mut OrtValue> {
        match self {
            OrtValue::Object(obj) => obj.get_mut(key),
            _ => None,
        }
    }
}

// Implement Index for string keys (object access)
impl Index<&str> for OrtValue {
    type Output = OrtValue;

    fn index(&self, key: &str) -> &Self::Output {
        static NULL: OrtValue = OrtValue::Null;
        match self {
            OrtValue::Object(obj) => obj.get(key).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

// Implement Index for usize (array access)
impl Index<usize> for OrtValue {
    type Output = OrtValue;

    fn index(&self, index: usize) -> &Self::Output {
        static NULL: OrtValue = OrtValue::Null;
        match self {
            OrtValue::Array(arr) => arr.get(index).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

// Display implementation
impl fmt::Display for OrtValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrtValue::Null => write!(f, "null"),
            OrtValue::Bool(b) => write!(f, "{}", b),
            OrtValue::Number(n) => write!(f, "{}", n),
            OrtValue::String(s) => write!(f, "\"{}\"", s),
            OrtValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            OrtValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// Convenience constructors
impl From<bool> for OrtValue {
    fn from(b: bool) -> Self {
        OrtValue::Bool(b)
    }
}

impl From<f32> for OrtValue {
    fn from(n: f32) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<f64> for OrtValue {
    fn from(n: f64) -> Self {
        OrtValue::Number(n)
    }
}

impl From<i8> for OrtValue {
    fn from(n: i8) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<i16> for OrtValue {
    fn from(n: i16) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<i32> for OrtValue {
    fn from(n: i32) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<i64> for OrtValue {
    fn from(n: i64) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<u8> for OrtValue {
    fn from(n: u8) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<u16> for OrtValue {
    fn from(n: u16) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<u32> for OrtValue {
    fn from(n: u32) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<u64> for OrtValue {
    fn from(n: u64) -> Self {
        OrtValue::Number(n as f64)
    }
}

impl From<String> for OrtValue {
    fn from(s: String) -> Self {
        OrtValue::String(s)
    }
}

impl From<&str> for OrtValue {
    fn from(s: &str) -> Self {
        OrtValue::String(s.to_string())
    }
}

impl From<Vec<OrtValue>> for OrtValue {
    fn from(arr: Vec<OrtValue>) -> Self {
        OrtValue::Array(arr)
    }
}

impl From<HashMap<String, OrtValue>> for OrtValue {
    fn from(obj: HashMap<String, OrtValue>) -> Self {
        OrtValue::Object(obj)
    }
}
