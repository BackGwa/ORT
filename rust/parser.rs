use crate::error::{OrtError, OrtResult};
use crate::ort_value::OrtValue;
use std::collections::HashMap;

pub fn parse_ort(content: &str) -> OrtResult<OrtValue> {
    let lines: Vec<&str> = content.lines().collect();
    let mut line_idx = 0;

    let mut result = HashMap::new();

    while line_idx < lines.len() {
        let line = lines[line_idx].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            line_idx += 1;
            continue;
        }

        // Parse header
        if line.contains(':') {
            let (key, fields, data_lines) = parse_section(&lines, line_idx)?;

            if let Some(key) = key {
                // keyName:fields: format
                let values = parse_data_lines(&lines, line_idx + 1, &fields, data_lines)?;
                result.insert(key, values);
                line_idx += data_lines + 1;
            } else {
                // :fields: format (top-level)
                let values = parse_data_lines(&lines, line_idx + 1, &fields, data_lines)?;

                // If single object, return as object
                if !fields.is_empty() && data_lines == 1 {
                    if let OrtValue::Array(ref arr) = values {
                        if arr.len() == 1 {
                            return Ok(arr[0].clone());
                        }
                    }
                }
                return Ok(values);
            }
        } else {
            line_idx += 1;
        }
    }

    Ok(OrtValue::Object(result))
}

fn parse_section(lines: &[&str], start_idx: usize) -> OrtResult<(Option<String>, Vec<Field>, usize)> {
    let line = lines[start_idx].trim();
    let line_num = start_idx + 1;

    // Count data lines (non-empty, non-comment lines until next header or end)
    let mut data_lines = 0;
    for i in (start_idx + 1)..lines.len() {
        let l = lines[i].trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        if l.contains(':') && is_header(l) {
            break;
        }
        data_lines += 1;
    }

    // Parse header
    let (key, fields_str) = parse_header(line, line_num)?;
    let fields = parse_fields(&fields_str, line, line_num)?;

    Ok((key, fields, data_lines))
}

fn is_header(line: &str) -> bool {
    // Check if line looks like a header (ends with : or has : at start)
    let trimmed = line.trim();
    if trimmed.starts_with(':') {
        return true;
    }

    // Check if it's keyName:fields: format
    let parts: Vec<&str> = trimmed.split(':').collect();
    if parts.len() >= 2 && parts[parts.len() - 1].is_empty() {
        return true;
    }

    false
}

fn parse_header(line: &str, line_num: usize) -> OrtResult<(Option<String>, String)> {
    if line.starts_with(':') {
        // :fields: format
        let content = line.trim_start_matches(':').trim_end_matches(':');
        Ok((None, content.to_string()))
    } else {
        // keyName:fields: format
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() < 2 {
            return Err(OrtError::new(
                line_num,
                line.to_string(),
                "Invalid header format".to_string(),
            ));
        }

        let key = parts[0].trim().to_string();
        let fields = parts[1].trim_end_matches(':').trim().to_string();

        Ok((Some(key), fields))
    }
}

#[derive(Debug, Clone)]
enum Field {
    Simple(String),
    Nested(String, Vec<Field>),
}

fn parse_fields(fields_str: &str, line: &str, line_num: usize) -> OrtResult<Vec<Field>> {
    if fields_str.is_empty() {
        return Ok(vec![]);
    }

    let mut result = vec![];
    let mut current = String::new();
    let mut depth = 0;
    let chars: Vec<char> = fields_str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        match ch {
            '(' => {
                if depth == 0 {
                    // Start of nested fields
                    let field_name = current.trim().to_string();
                    current.clear();
                    i += 1;

                    // Find matching closing paren
                    let mut nested_str = String::new();
                    let mut nested_depth = 1;

                    while i < chars.len() && nested_depth > 0 {
                        match chars[i] {
                            '(' => nested_depth += 1,
                            ')' => nested_depth -= 1,
                            _ => {}
                        }

                        if nested_depth > 0 {
                            nested_str.push(chars[i]);
                        }
                        i += 1;
                    }

                    let nested_fields = parse_fields(&nested_str, line, line_num)?;
                    result.push(Field::Nested(field_name, nested_fields));
                    continue;
                } else {
                    depth += 1;
                    current.push(ch);
                }
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return Err(OrtError::new(
                        line_num,
                        line.to_string(),
                        "Unmatched closing parenthesis".to_string(),
                    ));
                }
                current.push(ch);
            }
            ',' => {
                if depth == 0 {
                    let field = current.trim().to_string();
                    if !field.is_empty() {
                        result.push(Field::Simple(field));
                    }
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }

        i += 1;
    }

    let field = current.trim().to_string();
    if !field.is_empty() {
        result.push(Field::Simple(field));
    }

    Ok(result)
}

fn parse_data_lines(lines: &[&str], start_idx: usize, fields: &[Field], count: usize) -> OrtResult<OrtValue> {
    let mut result = vec![];
    let mut processed = 0;

    for i in start_idx..lines.len() {
        if processed >= count {
            break;
        }

        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line_num = i + 1;

        // Special case: array value without fields
        if fields.is_empty() {
            let value = parse_value(line, line, line_num)?;
            return Ok(value);
        }

        // Parse data values
        let values = parse_data_values(line, line_num)?;

        if values.len() != fields.len() {
            return Err(OrtError::new(
                line_num,
                line.to_string(),
                format!("Expected {} values but got {}", fields.len(), values.len()),
            ));
        }

        let mut obj = HashMap::new();
        for (field, value_str) in fields.iter().zip(values.iter()) {
            let value = parse_field_value(field, value_str, line, line_num)?;
            let key = match field {
                Field::Simple(name) => name.clone(),
                Field::Nested(name, _) => name.clone(),
            };
            obj.insert(key, value);
        }

        result.push(OrtValue::Object(obj));
        processed += 1;
    }

    Ok(OrtValue::Array(result))
}

fn parse_data_values(line: &str, _line_num: usize) -> OrtResult<Vec<String>> {
    let mut values = vec![];
    let mut current = String::new();
    let mut escaped = false;
    let mut depth = 0;
    let mut bracket_depth = 0;

    for ch in line.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => {
                escaped = true;
                current.push('\\');
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth -= 1;
                current.push(ch);
            }
            ',' => {
                if depth == 0 && bracket_depth == 0 {
                    values.push(current.clone());
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }

    values.push(current);
    Ok(values)
}

fn parse_field_value(field: &Field, value_str: &str, line: &str, line_num: usize) -> OrtResult<OrtValue> {
    match field {
        Field::Simple(_) => parse_value(value_str, line, line_num),
        Field::Nested(_, nested_fields) => {
            let trimmed = value_str.trim();

            // Check for empty value
            if trimmed.is_empty() {
                return Ok(OrtValue::Null);
            }

            // Check for empty object
            if trimmed == "()" {
                return Ok(OrtValue::Object(HashMap::new()));
            }

            // Handle array value dynamically (when field is defined as nested but value is array)
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                return parse_value(trimmed, line, line_num);
            }

            // Parse nested object
            if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
                // Fallback: parse as regular value if not in expected format
                return parse_value(trimmed, line, line_num);
            }

            let inner = &trimmed[1..trimmed.len()-1];
            let values = parse_data_values(inner, line_num)?;

            if values.len() != nested_fields.len() {
                return Err(OrtError::new(
                    line_num,
                    line.to_string(),
                    format!("Expected {} nested values but got {}", nested_fields.len(), values.len()),
                ));
            }

            let mut obj = HashMap::new();
            for (field, value_str) in nested_fields.iter().zip(values.iter()) {
                let value = parse_field_value(field, value_str, line, line_num)?;
                let key = match field {
                    Field::Simple(name) => name.clone(),
                    Field::Nested(name, _) => name.clone(),
                };
                obj.insert(key, value);
            }

            Ok(OrtValue::Object(obj))
        }
    }
}

fn parse_value(s: &str, line: &str, line_num: usize) -> OrtResult<OrtValue> {
    let trimmed = s.trim();

    // Empty value -> null
    if trimmed.is_empty() {
        return Ok(OrtValue::Null);
    }

    // Empty array
    if trimmed == "[]" {
        return Ok(OrtValue::Array(vec![]));
    }

    // Empty object
    if trimmed == "()" {
        return Ok(OrtValue::Object(HashMap::new()));
    }

    // Array
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_array(&trimmed[1..trimmed.len()-1], line, line_num);
    }

    // Inline object
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        return parse_inline_object(&trimmed[1..trimmed.len()-1], line, line_num);
    }

    // Unescape string
    let unescaped = unescape(trimmed);

    // Try parse as number
    if let Ok(num) = unescaped.parse::<i64>() {
        return Ok(OrtValue::Number(num as f64));
    }

    if let Ok(num) = unescaped.parse::<f64>() {
        return Ok(OrtValue::Number(num));
    }

    // Boolean
    if unescaped == "true" {
        return Ok(OrtValue::Bool(true));
    }
    if unescaped == "false" {
        return Ok(OrtValue::Bool(false));
    }

    // String
    Ok(OrtValue::String(unescaped))
}

fn parse_array(s: &str, line: &str, line_num: usize) -> OrtResult<OrtValue> {
    if s.trim().is_empty() {
        return Ok(OrtValue::Array(vec![]));
    }

    let mut result = vec![];
    let mut current = String::new();
    let mut escaped = false;
    let mut depth = 0;
    let mut bracket_depth = 0;

    for ch in s.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => {
                escaped = true;
                current.push('\\');
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth -= 1;
                current.push(ch);
            }
            ',' => {
                if depth == 0 && bracket_depth == 0 {
                    result.push(parse_value(&current, line, line_num)?);
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        result.push(parse_value(&current, line, line_num)?);
    }

    Ok(OrtValue::Array(result))
}

fn parse_inline_object(s: &str, line: &str, line_num: usize) -> OrtResult<OrtValue> {
    if s.trim().is_empty() {
        return Ok(OrtValue::Object(HashMap::new()));
    }

    let mut obj = HashMap::new();
    let pairs = split_pairs(s)?;

    for pair in pairs {
        if let Some(pos) = pair.find(':') {
            let key = pair[..pos].trim().to_string();
            let value_str = pair[pos+1..].trim();
            let value = parse_value(value_str, line, line_num)?;
            obj.insert(key, value);
        }
    }

    Ok(OrtValue::Object(obj))
}

fn split_pairs(s: &str) -> OrtResult<Vec<String>> {
    let mut pairs = vec![];
    let mut current = String::new();
    let mut escaped = false;
    let mut depth = 0;
    let mut bracket_depth = 0;

    for ch in s.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => {
                escaped = true;
                current.push('\\');
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth -= 1;
                current.push(ch);
            }
            ',' => {
                if depth == 0 && bracket_depth == 0 {
                    pairs.push(current.clone());
                    current.clear();
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        pairs.push(current);
    }

    Ok(pairs)
}

fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut escaped = false;

    for ch in s.chars() {
        if escaped {
            match ch {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                _ => result.push(ch),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            result.push(ch);
        }
    }

    result
}
