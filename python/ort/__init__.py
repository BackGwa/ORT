"""
Object Record Table - a CSV like structured data format with native object and array support.
"""

from .value import OrtValue
from .parser import parse_ort, OrtParseError
from .generator import generate_ort

__version__ = "0.1.0"
__all__ = ["OrtValue", "parse_ort", "generate_ort", "OrtParseError"]


def parse(content: str) -> OrtValue:
    """
    Parse ORT string into OrtValue.

    Args:
        content: ORT format string

    Returns:
        OrtValue object

    Raises:
        OrtParseError: If parsing fails
    """
    return parse_ort(content)


def generate(value: any) -> str:
    """
    Generate ORT string from Python value.

    Args:
        value: Python dict, list, or other JSON-compatible value

    Returns:
        ORT format string
    """
    ort_value = OrtValue(value)
    return generate_ort(ort_value)


def load(file_path: str) -> OrtValue:
    """
    Load ORT from file.

    Args:
        file_path: Path to ORT file

    Returns:
        OrtValue object

    Raises:
        OrtParseError: If parsing fails
        FileNotFoundError: If file doesn't exist
    """
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    return parse_ort(content)


def dump(value: any, file_path: str):
    """
    Save value as ORT to file.

    Args:
        value: Python dict, list, or other JSON-compatible value
        file_path: Path to output file
    """
    ort_value = OrtValue(value)
    ort_string = generate_ort(ort_value)
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(ort_string)
