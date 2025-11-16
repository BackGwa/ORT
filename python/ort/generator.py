"""ORT Generator for Python."""

from typing import List, Dict, Any
from .value import OrtValue


def generate_ort(value: OrtValue) -> str:
    """Generate ORT string from OrtValue."""
    if value.is_object():
        obj = value.as_object()
        if obj:
            obj_dict = {k: v.to_python() for k, v in obj.items()}

            if len(obj_dict) > 1 or not obj_dict:
                return _generate_multi_object(obj_dict)
            elif len(obj_dict) == 1:
                key, val = next(iter(obj_dict.items()))
                val_ort = OrtValue(val)
                if val_ort.is_array():
                    arr = val_ort.as_array()
                    if not arr:
                        return f"{key}:\n[]\n"
                    else:
                        arr_list = [v.to_python() for v in arr]
                        if _is_uniform_object_array(arr_list):
                            return _generate_object_array(key, arr_list)
                        else:
                            return _generate_simple_array(key, arr_list)
                else:
                    return f"{key}:\n{_generate_value(val, False)}\n"
        return ""
    elif value.is_array():
        arr = value.as_array()
        if arr:
            arr_list = [v.to_python() for v in arr]
            if _is_uniform_object_array(arr_list):
                return _generate_top_level_object_array(arr_list)
            else:
                return f":{_generate_array_content(arr_list, False)}\n"
        return ":[]\n"
    else:
        return _generate_value(value.to_python(), False)


def _generate_multi_object(obj: Dict[str, Any]) -> str:
    """Generate multi-key object."""
    result = []

    for key, val in obj.items():
        val_ort = OrtValue(val)
        if val_ort.is_array():
            arr = val_ort.as_array()
            if arr:
                arr_list = [v.to_python() for v in arr]
                if _is_uniform_object_array(arr_list):
                    result.append(_generate_object_array(key, arr_list))
                else:
                    result.append(_generate_simple_array(key, arr_list))
            else:
                result.append(f"{key}:\n[]\n")
        else:
            result.append(f"{key}:\n{_generate_value(val, False)}\n")
        result.append("\n")

    return "".join(result)


def _is_uniform_object_array(arr: List[Any]) -> bool:
    """Check if all elements are objects with the same keys."""
    if not arr:
        return False

    if not isinstance(arr[0], dict):
        return False

    first_keys = sorted(arr[0].keys())

    for item in arr[1:]:
        if not isinstance(item, dict):
            return False
        if sorted(item.keys()) != first_keys:
            return False

    return True


def _generate_object_array(key: str, arr: List[Dict[str, Any]]) -> str:
    """Generate object array with header and data rows."""
    if not arr:
        return f"{key}:\n[]\n"

    first = arr[0]
    keys = list(first.keys())
    header = _generate_header(keys, first)

    result = [f"{key}:{header}\n"]

    for item in arr:
        values = [
            _generate_object_field_value(
                item.get(k),
                keys,
                k,
                item
            )
            for k in keys
        ]
        result.append(",".join(values))
        result.append("\n")

    return "".join(result)


def _generate_top_level_object_array(arr: List[Dict[str, Any]]) -> str:
    """Generate top-level object array."""
    if not arr:
        return ":[]\n"

    first = arr[0]
    keys = list(first.keys())
    header = _generate_header(keys, first)

    result = [f":{header}\n"]

    for item in arr:
        values = [
            _generate_object_field_value(
                item.get(k),
                keys,
                k,
                item
            )
            for k in keys
        ]
        result.append(",".join(values))
        result.append("\n")

    return "".join(result)


def _generate_header(keys: List[str], first_obj: Dict[str, Any]) -> str:
    """Generate header with field names."""
    header_parts = []

    for k in keys:
        value = first_obj.get(k)
        if isinstance(value, dict):
            nested_keys = list(value.keys())
            nested_header = _generate_header_fields(nested_keys, value)
            header_parts.append(f"{k}({nested_header})")
        else:
            header_parts.append(k)

    return ",".join(header_parts) + ":"


def _generate_header_fields(keys: List[str], obj: Dict[str, Any]) -> str:
    """Generate header fields (recursive for nested objects)."""
    header_parts = []

    for k in keys:
        value = obj.get(k)
        if isinstance(value, dict):
            nested_keys = list(value.keys())
            nested_header = _generate_header_fields(nested_keys, value)
            header_parts.append(f"{k}({nested_header})")
        else:
            header_parts.append(k)

    return ",".join(header_parts)


def _generate_object_field_value(
    value: Any,
    keys: List[str],
    current_key: str,
    parent: Dict[str, Any]
) -> str:
    """Generate field value in object array."""
    if value is None:
        return ""
    elif isinstance(value, dict):
        if not value:
            return "()"
        else:
            nested_keys = list(value.keys())
            values = [
                _generate_object_field_value(
                    value.get(k),
                    nested_keys,
                    k,
                    value
                )
                for k in nested_keys
            ]
            return f"({','.join(values)})"
    elif isinstance(value, list):
        if not value:
            return "[]"
        else:
            return f"[{_generate_array_content(value, True)}]"
    else:
        return _generate_value(value, True)


def _generate_simple_array(key: str, arr: List[Any]) -> str:
    """Generate simple array."""
    return f"{key}:\n{_generate_array_content(arr, False)}\n"


def _generate_array_content(arr: List[Any], inline: bool) -> str:
    """Generate array content."""
    if not arr:
        return "[]"

    values = [_generate_value(v, inline) for v in arr]

    if inline:
        return ",".join(values)
    else:
        return f"[{','.join(values)}]"


def _generate_value(value: Any, inline: bool) -> str:
    """Generate a single value."""
    if value is None:
        return ""
    elif isinstance(value, bool):
        return "true" if value else "false"
    elif isinstance(value, int):
        return str(value)
    elif isinstance(value, float):
        if value == int(value):
            return str(int(value))
        return str(value)
    elif isinstance(value, str):
        return _escape(value)
    elif isinstance(value, list):
        if not value:
            return "[]"
        else:
            return f"[{_generate_array_content(value, True)}]"
    elif isinstance(value, dict):
        if not value:
            return "()"
        else:
            return _generate_inline_object(value)
    else:
        return str(value)


def _generate_inline_object(obj: Dict[str, Any]) -> str:
    """Generate inline object."""
    pairs = [f"{k}:{_generate_value(v, True)}" for k, v in obj.items()]
    return f"({','.join(pairs)})"


def _escape(s: str) -> str:
    """Escape special characters in string."""
    result = []

    for ch in s:
        if ch == '(':
            result.append('\\(')
        elif ch == ')':
            result.append('\\)')
        elif ch == '[':
            result.append('\\[')
        elif ch == ']':
            result.append('\\]')
        elif ch == ',':
            result.append('\\,')
        elif ch == '\\':
            result.append('\\\\')
        elif ch == '\n':
            result.append('\\n')
        elif ch == '\t':
            result.append('\\t')
        elif ch == '\r':
            result.append('\\r')
        else:
            result.append(ch)

    return "".join(result)
