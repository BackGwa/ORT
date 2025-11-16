# Object Record Table (ORT)

ORT is a CSV-like structured data format with native support for objects and arrays.  
Unlike inefficient human-readable data structures, ORT efficiently represents data, making it ideal for large language model (LLM) token optimization and structured data interchange.

## Table of Contents

- [Why ORT](#why-ort)
- [When Not to Use ORT](#when-not-to-use-ort)
- [Command-Line Tools](#command-line-tools)
- [Installation](#installation)
- [Package Links](#package-links)
- [License](#license)

---

## Why ORT

Standard JSON and YAML prioritize human readability, but their structural characteristics introduce significant computational overhead when passing data to large language models. ORT addresses this by optimizing data representation for token efficiency.

### JSON
```json
{
  "users": [
    {
      "id": 1,
      "profile": {
        "name": "John Doe",
        "age": 30,
        "address": {
          "city": "New York",
          "country": "USA"
        }
      }
    },
    {
      "id": 2,
      "profile": {
        "name": "Jane Smith",
        "age": 25,
        "address": {
          "city": "London",
          "country": "UK"
        }
      }
    }
  ]
}
```
> 398 Characters / 118 Tokens

### Compact-JSON
```json
{"users":[{"id":1,"profile":{"name":"John Doe","age":30,"address":{"city":"New York","country":"USA"}}},{"id":2,"profile":{"name":"Jane Smith","age":25,"address":{"city":"London","country":"UK"}}}]}
```
> 198 Characters / 58 Tokens

### ORT
```ort
users:id,profile(name,age,address(city,country)):
1,(John Doe,30,(New York,USA))
2,(Jane Smith,25,(London,UK))
```
> 110 Characters / 35 Tokens

---

## When Not to Use ORT

ORT is not always the optimal choice. Consider using JSON in the following scenarios:

- **Non-uniform data structures**: ORT operates efficiently and reliably with uniform data structures. For heterogeneous data, JSON is more efficient and prevents data loss.
- **Direct application data files**: ORT is not optimized for direct input/output operations. Since ORT specializes in data transformation and processing, using JSON for application data files offers better compatibility and stability.

---

## Command-Line Tools

The Rust implementation provides CLI utilities for converting between ORT and JSON formats:

```sh
ort2json <input_file>
json2ort <input_file>

# Or with output file specification

ort2json <input_file> -o <output_file>
json2ort <input_file> -o <output_file>
```

---

## Installation

### Rust Library

Add to your Cargo.toml:

```toml
[dependencies]
ort = "0.1.0"
```

### TypeScript Library
```
npm i ort-ts
```

### Python Library
```
pip install ort-python
```

---

## Package Links
- PyPI: https://pypi.org/project/ort-python/
- npm: https://www.npmjs.com/package/ort-ts

---

## License
This project is licensed under the MIT License. See the LICENSE file for details.