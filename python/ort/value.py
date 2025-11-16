"""ORT Value type for Python."""

from typing import Any, Dict, List, Union, Optional


class OrtValue:
    """Represents an ORT value that can be null, bool, number, string, array, or object."""

    def __init__(self, value: Any):
        """Initialize OrtValue from a Python value."""
        self._value = self._normalize(value)

    @staticmethod
    def _normalize(value: Any) -> Any:
        """Normalize Python values to ORT-compatible types."""
        if value is None:
            return None
        elif isinstance(value, bool):
            return value
        elif isinstance(value, (int, float)):
            return float(value)
        elif isinstance(value, str):
            return value
        elif isinstance(value, list):
            return [OrtValue(v)._value for v in value]
        elif isinstance(value, dict):
            return {k: OrtValue(v)._value for k, v in value.items()}
        elif isinstance(value, OrtValue):
            return value._value
        else:
            raise TypeError(f"Unsupported type for OrtValue: {type(value)}")

    def is_null(self) -> bool:
        """Check if the value is null."""
        return self._value is None

    def is_bool(self) -> bool:
        """Check if the value is a boolean."""
        return isinstance(self._value, bool)

    def is_number(self) -> bool:
        """Check if the value is a number."""
        return isinstance(self._value, (int, float)) and not isinstance(self._value, bool)

    def is_string(self) -> bool:
        """Check if the value is a string."""
        return isinstance(self._value, str)

    def is_array(self) -> bool:
        """Check if the value is an array."""
        return isinstance(self._value, list)

    def is_object(self) -> bool:
        """Check if the value is an object."""
        return isinstance(self._value, dict)

    def as_bool(self) -> Optional[bool]:
        """Get the value as a boolean, or None if not a boolean."""
        return self._value if isinstance(self._value, bool) else None

    def as_int(self) -> Optional[int]:
        """Get the value as an integer, or None if not a number."""
        if isinstance(self._value, (int, float)) and not isinstance(self._value, bool):
            return int(self._value)
        return None

    def as_float(self) -> Optional[float]:
        """Get the value as a float, or None if not a number."""
        if isinstance(self._value, (int, float)) and not isinstance(self._value, bool):
            return float(self._value)
        return None

    def as_str(self) -> Optional[str]:
        """Get the value as a string, or None if not a string."""
        return self._value if isinstance(self._value, str) else None

    def as_array(self) -> Optional[List['OrtValue']]:
        """Get the value as an array, or None if not an array."""
        if isinstance(self._value, list):
            return [OrtValue(v) for v in self._value]
        return None

    def as_object(self) -> Optional[Dict[str, 'OrtValue']]:
        """Get the value as an object, or None if not an object."""
        if isinstance(self._value, dict):
            return {k: OrtValue(v) for k, v in self._value.items()}
        return None

    def __getitem__(self, key: Union[str, int]) -> 'OrtValue':
        """Access array elements or object fields using [] syntax."""
        if isinstance(key, str):
            if not isinstance(self._value, dict):
                raise TypeError(f"Cannot index non-object with string key: {key}")
            if key not in self._value:
                raise KeyError(f"Key not found: {key}")
            return OrtValue(self._value[key])
        elif isinstance(key, int):
            if not isinstance(self._value, list):
                raise TypeError(f"Cannot index non-array with integer: {key}")
            if key < 0 or key >= len(self._value):
                raise IndexError(f"Index out of bounds: {key}")
            return OrtValue(self._value[key])
        else:
            raise TypeError(f"Key must be str or int, got {type(key)}")

    def __setitem__(self, key: Union[str, int], value: Any):
        """Set array elements or object fields using [] syntax."""
        if isinstance(key, str):
            if not isinstance(self._value, dict):
                raise TypeError(f"Cannot set key on non-object: {key}")
            self._value[key] = OrtValue(value)._value
        elif isinstance(key, int):
            if not isinstance(self._value, list):
                raise TypeError(f"Cannot set index on non-array: {key}")
            if key < 0 or key >= len(self._value):
                raise IndexError(f"Index out of bounds: {key}")
            self._value[key] = OrtValue(value)._value
        else:
            raise TypeError(f"Key must be str or int, got {type(key)}")

    def get(self, key: str, default: Any = None) -> 'OrtValue':
        """Get object field with default value."""
        if not isinstance(self._value, dict):
            return OrtValue(default)
        return OrtValue(self._value.get(key, default))

    def to_python(self) -> Any:
        """Convert OrtValue to native Python types."""
        if self._value is None:
            return None
        elif isinstance(self._value, (bool, int, float, str)):
            return self._value
        elif isinstance(self._value, list):
            return [OrtValue(v).to_python() for v in self._value]
        elif isinstance(self._value, dict):
            return {k: OrtValue(v).to_python() for k, v in self._value.items()}
        return self._value

    def __repr__(self) -> str:
        """String representation of OrtValue."""
        return f"OrtValue({self._value!r})"

    def __eq__(self, other: Any) -> bool:
        """Check equality with another OrtValue or Python value."""
        if isinstance(other, OrtValue):
            return self._value == other._value
        return self._value == other

    def __len__(self) -> int:
        """Get the length of an array or object."""
        if isinstance(self._value, (list, dict)):
            return len(self._value)
        raise TypeError(f"Object of type {type(self._value).__name__} has no len()")
