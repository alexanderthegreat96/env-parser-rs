# 🦀 env_parser_rs

A lightweight, type-safe environment variable and configuration parser for Rust.

`env_parser_rs` is designed to be efficient, easy to use and dependency free. While it excels at `.env` files, it can parse any configuration file using standard `KEY=VALUE` syntax (like `.conf` or `.ini` styles). It handles normalization, stripping quotes, and converting raw strings into usable Rust types—including Lists and HashMaps—with a single method call.

## ✨ Features

- **🚀 Type-Safe Retrieval**: Parse directly into `i32`, `f64`, `bool`, `Vec<String>`, or `HashMap<String, String>`.
    
- **📂 Universal Syntax**: Works with `.env`, `.conf`, or any plain-text file using `KEY=VALUE` formatting.
    
- **🔡 Case-Insensitive**: Keys are automatically normalized to uppercase for robust lookups.
    
- **📦 Complex Structures**: Native support for arrays `[a, b, c]` and dictionaries `{key: val}`.
    
- **🛡️ Robust Parsing**: Handles comments (`#`), extra whitespace, and quoted values (`"..."` or `'...'`).
    
- **🔍 Debug Mode**: Optional verbose logging to help track down missing keys or parsing errors.
    

## 💾 Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
env_parser_rs = "0.2.0"
```

## 📖 Data Formats & Usage

|Type|Syntax in File|Method|Description|
|---|---|---|---|
|**String**|`NAME="Rust Parser"`|`.get_str("NAME")`|Returns `Option<String>`|
|**Integer**|`PORT=8080`|`.get_int::<i32>("PORT")`|Supports any `FromStr` integer|
|**Float**|`RATE=1.23`|`.get_float::<f64>("RATE")`|Supports any `FromStr` float|
|**Boolean**|`DEBUG=on`|`.get_bool("DEBUG")`|Supports `true/1/yes/on` and `false/0/no/off`|
|**List**|`TAGS=[a, b, c]`|`.get_list("TAGS")`|Returns `Option<Vec<String>>`|
|**Map**|`META={v:1, env:dev}`|`.get_dict("META")`|Returns `Option<HashMap<String, String>>`|

## 🚀 Quick Start

### 1. Create your config (`app.conf`)

```env
# Server Configuration
PORT=8080
BASE_URL="[https://api.example.com](https://api.example.com)"

# Feature Flags
ENABLE_LOGS=yes
MAX_RETRIES=5

# Complex Data
ALLOWED_IPS=[127.0.0.1, 192.168.1.1]
DATABASE={host:localhost, port:5432, user:admin}
```

### 2. Parse in Rust

```rust
use env_parser_rs::{EnvParser, Types};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let parser = EnvParser::from_file("app.conf", true)?;

    let port: i32 = parser.get_int("PORT").expect("port is required");
    let is_enabled: bool = parser.get_bool("ENABLE_LOGS").unwrap_or(false);
    
    if let Some(ips) = parser.get_list("ALLOWED_IPS") {
        println!("whitelist size: {}", ips.len());
    }

    if let Some(db) = parser.get_dict("DATABASE") {
        let db_host = db.get("host").ok_or("database host is missing")?;
        println!("connecting to: {}", db_host);
    }

    if let Some(val) = parser.get_value("TIMEOUT", Types::Float) {
        println!("parsed value: {:?}", val);
    }
    parser.print_contents();
    Ok(())
}
```

## 🛠️ API Reference

### Initialization Methods

|Method|Parameters|Returns|Description|
|---|---|---|---|
|`from_file`|`path: P, debug: bool`|`Self`|Creates a parser and attempts to read/parse the file immediately.|
|`get_error`|None|`Option<&dyn Error>`|Returns the last error encountered during file reading or parsing.|

### Retrieval Methods

All retrieval methods are case-insensitive regarding the `key`.

|Method|Returns|Notes|
|---|---|---|
|`get_str(key)`|`Option<String>`|Clones the internal value.|
|`get_int<T>(key)`|`Option<T>`|Where `T` implements `FromStr`.|
|`get_float<T>(key)`|`Option<T>`|Where `T` implements `FromStr`.|
|`get_bool(key)`|`Option<bool>`|Truthy: `true`, `1`, `yes`, `on`. Falsy: `false`, `0`, `no`, `off`.|
|`get_list(key)`|`Option<Vec<String>>`|Handles both `[a,b]` and `a,b` syntax. Strips quotes from items.|
|`get_dict(key)`|`Option<HashMap<String, String>>`|Parses `{k:v}` pairs separated by commas.|
|`get_value(key, Types)`|`Option<AnyValue>`|Useful for dynamic dispatch or pattern matching.|

### Utility Methods

- `print_contents()`: Outputs a formatted table of all loaded variables to `stdout`.
    
- `get_data()`: Returns a reference to the internal `HashMap<String, String>`.
    

## 🧪 Advanced Examples

### Dynamic Dispatch with `AnyValue`

If you don't know the type at compile time or want to handle multiple types in a single logic flow:

```rust
use env_parser_rs::{AnyValue, Types};

let val = parser.get_value("TIMEOUT", Types::Int);

match val {
    Some(AnyValue::Int(i)) => println!("Numeric timeout: {}", i),
    Some(AnyValue::Text(s)) => println!("String timeout: {}", s),
    _ => println!("Value not found or incompatible"),
}
```

### Display Trait

You can print the `EnvParser` instance directly to see the file status and contents:

```
let parser = EnvParser::from_file(".env", false);
println!("{}", parser);
```

## 🛠 Development & Testing

The library is heavily tested to ensure edge cases (like nested quotes or mixed casing) are handled correctly.

```
# Run all tests
cargo test
```

## 📜 License

This project is licensed under the MIT License.