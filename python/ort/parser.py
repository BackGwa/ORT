"""ORT Parser for Python."""

from typing import List, Dict, Tuple, Optional, Any
from .value import OrtValue


class OrtParseError(Exception):
    """Exception raised when parsing ORT fails."""

    def __init__(self, line_num: int, line: str, message: str):
        self.line_num = line_num
        self.line = line
        self.message = message
        super().__init__(f"Line {line_num}: {message}\n  {line}")


class Field:
    """Represents a field in ORT header."""

    def __init__(self, name: str, nested_fields: Optional[List['Field']] = None):
        self.name = name
        self.nested_fields = nested_fields or []

    def is_nested(self) -> bool:
        return len(self.nested_fields) > 0

    def __repr__(self) -> str:
        if self.is_nested():
            return f"Field({self.name}, {self.nested_fields})"
        return f"Field({self.name})"


def parse_ort(content: str) -> OrtValue:
    """Parse ORT string into OrtValue."""
    lines = content.split('\n')
    line_idx = 0
    result = {}

    while line_idx < len(lines):
        line = lines[line_idx].strip()

        if not line or line.startswith('#'):
            line_idx += 1
            continue

        if ':' in line:
            key, fields, data_lines = _parse_section(lines, line_idx)

            if key is not None:
                values = _parse_data_lines(lines, line_idx + 1, fields, data_lines)
                result[key] = values.to_python()
                line_idx += data_lines + 1
            else:
                values = _parse_data_lines(lines, line_idx + 1, fields, data_lines)

                if fields and data_lines == 1:
                    if values.is_array():
                        arr = values.as_array()
                        if arr and len(arr) == 1:
                            return arr[0]
                return values
        else:
            line_idx += 1

    return OrtValue(result)


def _parse_section(lines: List[str], start_idx: int) -> Tuple[Optional[str], List[Field], int]:
    """Parse a section header and count data lines."""
    line = lines[start_idx].strip()
    line_num = start_idx + 1

    data_lines = 0
    for i in range(start_idx + 1, len(lines)):
        l = lines[i].strip()
        if not l or l.startswith('#'):
            continue
        if ':' in l and _is_header(l):
            break
        data_lines += 1

    key, fields_str = _parse_header(line, line_num)
    fields = _parse_fields(fields_str, line, line_num)

    return key, fields, data_lines


def _is_header(line: str) -> bool:
    """Check if line looks like a header."""
    trimmed = line.strip()
    if trimmed.startswith(':'):
        return True

    parts = trimmed.split(':')
    if len(parts) >= 2 and not parts[-1]:
        return True

    return False


def _parse_header(line: str, line_num: int) -> Tuple[Optional[str], str]:
    """Parse header line into key and fields string."""
    if line.startswith(':'):
        content = line.strip(':')
        return None, content
    else:
        parts = line.split(':', 1)
        if len(parts) < 2:
            raise OrtParseError(line_num, line, "Invalid header format")

        key = parts[0].strip()
        fields = parts[1].rstrip(':').strip()

        return key, fields


def _parse_fields(fields_str: str, line: str, line_num: int) -> List[Field]:
    """Parse fields string into list of Field objects."""
    if not fields_str:
        return []

    result = []
    current = []
    depth = 0
    chars = list(fields_str)
    i = 0

    while i < len(chars):
        ch = chars[i]

        if ch == '(':
            if depth == 0:
                field_name = ''.join(current).strip()
                current = []
                i += 1

                nested_str = []
                nested_depth = 1

                while i < len(chars) and nested_depth > 0:
                    if chars[i] == '(':
                        nested_depth += 1
                    elif chars[i] == ')':
                        nested_depth -= 1

                    if nested_depth > 0:
                        nested_str.append(chars[i])
                    i += 1

                nested_fields = _parse_fields(''.join(nested_str), line, line_num)
                result.append(Field(field_name, nested_fields))
                continue
            else:
                depth += 1
                current.append(ch)
        elif ch == ')':
            depth -= 1
            if depth < 0:
                raise OrtParseError(line_num, line, "Unmatched closing parenthesis")
            current.append(ch)
        elif ch == ',':
            if depth == 0:
                field = ''.join(current).strip()
                if field:
                    result.append(Field(field))
                current = []
            else:
                current.append(ch)
        else:
            current.append(ch)

        i += 1

    field = ''.join(current).strip()
    if field:
        result.append(Field(field))

    return result


def _parse_data_lines(lines: List[str], start_idx: int, fields: List[Field], count: int) -> OrtValue:
    """Parse data lines according to fields."""
    result = []
    processed = 0

    for i in range(start_idx, len(lines)):
        if processed >= count:
            break

        line = lines[i].strip()
        if not line or line.startswith('#'):
            continue

        line_num = i + 1

        if not fields:
            value = _parse_value(line, line, line_num)
            return value

        values = _parse_data_values(line, line_num)

        if len(values) != len(fields):
            raise OrtParseError(
                line_num,
                line,
                f"Expected {len(fields)} values but got {len(values)}"
            )

        obj = {}
        for field, value_str in zip(fields, values):
            value = _parse_field_value(field, value_str, line, line_num)
            obj[field.name] = value.to_python()

        result.append(obj)
        processed += 1

    return OrtValue(result)


def _parse_data_values(line: str, line_num: int) -> List[str]:
    """Split data line into values, respecting nested structures."""
    values = []
    current = []
    escaped = False
    depth = 0
    bracket_depth = 0

    for ch in line:
        if escaped:
            current.append(ch)
            escaped = False
            continue

        if ch == '\\':
            escaped = True
            current.append('\\')
        elif ch == '(':
            depth += 1
            current.append(ch)
        elif ch == ')':
            depth -= 1
            current.append(ch)
        elif ch == '[':
            bracket_depth += 1
            current.append(ch)
        elif ch == ']':
            bracket_depth -= 1
            current.append(ch)
        elif ch == ',':
            if depth == 0 and bracket_depth == 0:
                values.append(''.join(current))
                current = []
            else:
                current.append(ch)
        else:
            current.append(ch)

    values.append(''.join(current))
    return values


def _parse_field_value(field: Field, value_str: str, line: str, line_num: int) -> OrtValue:
    """Parse a field value, handling nested objects."""
    if not field.is_nested():
        return _parse_value(value_str, line, line_num)

    trimmed = value_str.strip()

    if not trimmed:
        return OrtValue(None)

    if trimmed == "()":
        return OrtValue({})

    # Handle array value dynamically (when field is defined as nested but value is array)
    if trimmed.startswith('[') and trimmed.endswith(']'):
        return _parse_value(trimmed, line, line_num)

    if not (trimmed.startswith('(') and trimmed.endswith(')')):
        # Fallback: parse as regular value if not in expected format
        return _parse_value(trimmed, line, line_num)

    inner = trimmed[1:-1]
    values = _parse_data_values(inner, line_num)

    if len(values) != len(field.nested_fields):
        raise OrtParseError(
            line_num,
            line,
            f"Expected {len(field.nested_fields)} nested values but got {len(values)}"
        )

    obj = {}
    for nested_field, value_str in zip(field.nested_fields, values):
        value = _parse_field_value(nested_field, value_str, line, line_num)
        obj[nested_field.name] = value.to_python()

    return OrtValue(obj)


def _parse_value(s: str, line: str, line_num: int) -> OrtValue:
    """Parse a single value."""
    trimmed = s.strip()

    if not trimmed:
        return OrtValue(None)

    if trimmed == "[]":
        return OrtValue([])

    if trimmed == "()":
        return OrtValue({})

    if trimmed.startswith('[') and trimmed.endswith(']'):
        return _parse_array(trimmed[1:-1], line, line_num)

    if trimmed.startswith('(') and trimmed.endswith(')'):
        return _parse_inline_object(trimmed[1:-1], line, line_num)

    unescaped = _unescape(trimmed)

    try:
        num = int(unescaped)
        return OrtValue(float(num))
    except ValueError:
        pass

    try:
        num = float(unescaped)
        return OrtValue(num)
    except ValueError:
        pass

    if unescaped == "true":
        return OrtValue(True)
    if unescaped == "false":
        return OrtValue(False)

    return OrtValue(unescaped)


def _parse_array(s: str, line: str, line_num: int) -> OrtValue:
    """Parse array content."""
    if not s.strip():
        return OrtValue([])

    result = []
    current = []
    escaped = False
    depth = 0
    bracket_depth = 0

    for ch in s:
        if escaped:
            current.append(ch)
            escaped = False
            continue

        if ch == '\\':
            escaped = True
            current.append('\\')
        elif ch == '(':
            depth += 1
            current.append(ch)
        elif ch == ')':
            depth -= 1
            current.append(ch)
        elif ch == '[':
            bracket_depth += 1
            current.append(ch)
        elif ch == ']':
            bracket_depth -= 1
            current.append(ch)
        elif ch == ',':
            if depth == 0 and bracket_depth == 0:
                value = _parse_value(''.join(current), line, line_num)
                result.append(value.to_python())
                current = []
            else:
                current.append(ch)
        else:
            current.append(ch)

    current_str = ''.join(current).strip()
    if current_str:
        value = _parse_value(current_str, line, line_num)
        result.append(value.to_python())

    return OrtValue(result)


def _parse_inline_object(s: str, line: str, line_num: int) -> OrtValue:
    """Parse inline object content."""
    if not s.strip():
        return OrtValue({})

    obj = {}
    pairs = _split_pairs(s)

    for pair in pairs:
        if ':' in pair:
            pos = pair.index(':')
            key = pair[:pos].strip()
            value_str = pair[pos+1:].strip()
            value = _parse_value(value_str, line, line_num)
            obj[key] = value.to_python()

    return OrtValue(obj)


def _split_pairs(s: str) -> List[str]:
    """Split inline object pairs by comma, respecting nesting."""
    pairs = []
    current = []
    escaped = False
    depth = 0
    bracket_depth = 0

    for ch in s:
        if escaped:
            current.append(ch)
            escaped = False
            continue

        if ch == '\\':
            escaped = True
            current.append('\\')
        elif ch == '(':
            depth += 1
            current.append(ch)
        elif ch == ')':
            depth -= 1
            current.append(ch)
        elif ch == '[':
            bracket_depth += 1
            current.append(ch)
        elif ch == ']':
            bracket_depth -= 1
            current.append(ch)
        elif ch == ',':
            if depth == 0 and bracket_depth == 0:
                pairs.append(''.join(current))
                current = []
            else:
                current.append(ch)
        else:
            current.append(ch)

    current_str = ''.join(current).strip()
    if current_str:
        pairs.append(current_str)

    return pairs


def _unescape(s: str) -> str:
    """Unescape string."""
    result = []
    escaped = False

    for ch in s:
        if escaped:
            if ch == 'n':
                result.append('\n')
            elif ch == 't':
                result.append('\t')
            elif ch == 'r':
                result.append('\r')
            else:
                result.append(ch)
            escaped = False
        elif ch == '\\':
            escaped = True
        else:
            result.append(ch)

    return ''.join(result)
