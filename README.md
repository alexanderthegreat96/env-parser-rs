# EnvParser

A lightweight, type-safe environment variable parser for Rust. While designed with `.env` files in mind, **EnvParser can parse any configuration file that utilizes standard environment variable syntax** (e.g., `.conf`, `.ini` style key-value pairs, or custom config files).

It supports standard scalars (Strings, Integers, Booleans) as well as complex structures like Lists and Dictionaries directly from your configuration files.

## Features

- **Universal Syntax Support**: Works with `.env`, `.conf`, or any plain-text file using `KEY=VALUE` formatting.
    
- **Type-Safe Retrieval**: Parse variables directly into `i32`, `f64`, `bool`, etc.
    
- **Complex Structures**: Support for arrays `[a, b, c]` and dictionaries `{key:val}`.
    
- **Normalization**: Automatic key normalization (case-insensitivity).
    
- **Debug Mode**: Optional verbose logging for missing keys or parsing errors.
    
- **Robustness**: Handles comments, whitespace, and quoted values (`"like this"` or `'this'`).
    

## Installation

Add this to your `Cargo.toml`:

```
[dependencies]
env_parser_rs = "0.1.0"
```

## Quick Start

Create a configuration file (e.g., `app.conf`):

```
PORT=8080
DEBUG=true
ALLOWED_HOSTS=[localhost, 127.0.0.1]
DATABASE_CONFIG={timeout:30, pool:5}
```

Use it in your Rust code:

```
use env_parser::{EnvParser, Types};

fn main() {
    // Initialize and parse automatically from any file path
    let parser = EnvParser::from_file("app.conf", true);

    // 1. Get simple types
    let port = parser.get_int::<i32>("PORT").unwrap_or(3000);
    let is_debug = parser.get_bool("debug").unwrap_or(false);

    // 2. Get lists
    if let Some(hosts) = parser.get_list("ALLOWED_HOSTS") {
        for host in hosts {
            println!("Allowed: {}", host);
        }
    }

    // 3. Get dictionaries
    if let Some(db_conf) = parser.get_dict("DATABASE_CONFIG") {
        println!("DB Timeout: {}", db_conf.get("timeout").unwrap());
    }

    // 4. Use the dynamic dispatcher
    use env_parser::AnyValue;
    if let Some(AnyValue::Int(val)) = parser.get_value("PORT", Types::Int) {
        println!("Dynamic port: {}", val);
    }
}
```

## Data Formats Supported

|Format|Example|Retrieval Method|
|---|---|---|
|**String**|`NAME="App"`|`get_str("NAME")`|
|**Integer**|`PORT=8080`|`get_int::<i32>("PORT")`|
|**Boolean**|`ACTIVE=yes`|`get_bool("ACTIVE")` (Supports true/1/yes/on)|
|**List**|`TAGS=[a,b]`|`get_list("TAGS")`|
|**Dictionary**|`MAP={a:1,b:2}`|`get_dict("MAP")`|

## API Reference

### `EnvParser::from_file(path, is_debug)`

Creates a new parser and immediately reads the file at `path`. This can be a `.env` file or any configuration file using `=` as a delimiter.

### `get_int<T>(key)`

Generic method to parse any type that implements `FromStr`.

### `get_value(key, Types)`

Returns an `AnyValue` enum variant, useful for dynamic processing or logging.

### `print_contents()`

Pretty-prints the entire loaded configuration to the console in a table format.

## Development

Run tests using:

```
cargo test
```

## License
MIT