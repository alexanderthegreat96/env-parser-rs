use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum AnyValue {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<String>),
    Dict(HashMap<String, String>),
}

#[derive(Debug, Clone, Copy)]
pub enum Types {
    String,
    Int,
    Integer,
    Float,
    Double,
    Boolean,
    Bool,
    Array,
    Vector,
    List,
    HashMap,
    Dictionary,
    Map,
}

pub struct EnvParser {
    file_path: PathBuf,
    env_contents: Option<HashMap<String, String>>,
    is_debug: bool,
    last_error: Option<Box<dyn Error>>,
}

impl EnvParser {
    pub fn from_file<P: AsRef<Path>>(file_path: P, is_debug: bool) -> Self {
        let mut parser: EnvParser = Self::new(file_path, is_debug);
        parser.parse();
        return parser;
    }

    // grabs the last error
    pub fn get_error(&self) -> Option<&dyn Error> {
        return self.last_error.as_deref();
    }

    // retrieves the entire data set as a hashmap
    pub fn get_data(&self) -> Option<&HashMap<String, String>> {
        self.env_contents.as_ref()
    }

    // get the value as string
    pub fn get_str(&self, key: &str) -> Option<String> {
        return self.get(key).cloned();
    }

    // get the value as int
    pub fn get_int<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        return self.get(key)?.parse::<T>().ok();
    }

    // get the value as float
    pub fn get_float<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        return self.get(key)?.parse::<T>().ok();
    }

    // get the value as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let val = self.get(key)?.to_lowercase();
        match val.as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    // get the value as a vector / slice / list
    pub fn get_list(&self, key: &str) -> Option<Vec<String>> {
        let val = self.get(key)?;
        let val = val.trim();

        let inner = if val.starts_with('[') && val.ends_with(']') {
            &val[1..val.len() - 1]
        } else {
            val
        };

        let list: Vec<String> = inner
            .split(',')
            .map(|item| {
                item.trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string()
            })
            .filter(|item| !item.is_empty())
            .collect();

        if list.is_empty() { None } else { Some(list) }
    }

    // get the value as a dict
    pub fn get_dict(&self, key: &str) -> Option<HashMap<String, String>> {
        let val = self.get(key)?;
        let val = val.trim();

        let inner = if val.starts_with('{') && val.ends_with('}') {
            &val[1..val.len() - 1]
        } else {
            val
        };

        let mut map = HashMap::new();
        for pair in inner.split(',') {
            if let Some((k, v)) = pair.split_once(':') {
                let clean_key = k.trim().trim_matches(|c| c == '"' || c == '\'');
                let clean_val = v.trim().trim_matches(|c| c == '"' || c == '\'');

                map.insert(clean_key.to_string(), clean_val.to_string());
            }
        }

        if map.is_empty() { None } else { Some(map) }
    }

    // get the value as a specific type
    pub fn get_value(&self, key: &str, convert_to: Types) -> Option<AnyValue> {
        match convert_to {
            Types::Int | Types::Integer => self.get_int::<i64>(key).map(AnyValue::Int),
            Types::Float | Types::Double => self.get_int::<f64>(key).map(AnyValue::Float),
            Types::Boolean | Types::Bool => self.get_bool(key).map(AnyValue::Bool),
            Types::Array | Types::Vector | Types::List => self.get_list(key).map(AnyValue::List),
            Types::HashMap | Types::Dictionary | Types::Map => {
                self.get_dict(key).map(AnyValue::Dict)
            }
            Types::String => self.get(key).map(|s| AnyValue::Text(s.clone())),
        }
    }

    // prints the contents of the env file
    pub fn print_contents(&self) -> () {
        println!("{:<20} | {:<20}", "VARIABLE", "VALUE");
        println!("{}", "-".repeat(43));

        match &self.env_contents {
            Some(data) if !data.is_empty() => {
                let mut keys: Vec<&String> = data.keys().collect();
                keys.sort();

                for key in keys {
                    let value = &data[key];
                    println!("{:<20} | {:<20}", key, value);
                }
            }
            Some(_) => println!("(No variables found in file)"),
            None => {
                let status = self
                    .get_error()
                    .map(|e| format!("ERROR ({})", e))
                    .unwrap_or_else(|| "NOT PARSED".to_string());
                println!("Status: {}", status);
            }
        }
        println!("{}", "-".repeat(43));
    }

    fn new<P: AsRef<Path>>(file_path: P, is_debug: bool) -> Self {
        EnvParser {
            file_path: file_path.as_ref().to_path_buf(),
            env_contents: None,
            is_debug,
            last_error: None,
        }
    }

    fn parse(&mut self) {
        let mut file = match File::open(&self.file_path) {
            Ok(f) => f,
            Err(e) => {
                if self.is_debug {
                    println!("Error opening file: {e}");
                }
                self.env_contents = None;
                self.last_error = Some(Box::new(e));
                return;
            }
        };

        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            if self.is_debug {
                println!("Error reading file: {e}");
            }
            self.env_contents = None;
            self.last_error = Some(Box::new(e));
            return;
        }

        let mut data: HashMap<String, String> = HashMap::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let val = val.trim();

                let final_val = if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    val[1..val.len() - 1].replace("\\\"", "\"")
                } else if val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2 {
                    val[1..val.len() - 1].replace("\\'", "'")
                } else {
                    val.to_string()
                };

                data.insert(key, final_val);
            }
        }

        self.env_contents = Some(data);
    }

    fn get(&self, key: &str) -> Option<&String> {
        let key = key.to_uppercase();
        let data: &HashMap<String, String> = self.get_data()?;
        let value: Option<&String> = data.get(&key);

        if value.is_none() && self.is_debug {
            println!("No key: {} found in {}", key, self.file_path.display());
        }

        return value;
    }
}

// allows you to print the var assigned to EnvParser::from_file
// directly
// which will output the contents if found
impl std::fmt::Display for EnvParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnvParser: (file: {})\n", self.file_path.display())?;
        println!("{}", "-".repeat(43));

        match &self.env_contents {
            Some(data) if !data.is_empty() => {
                let mut keys: Vec<&String> = data.keys().collect();
                keys.sort();
                for key in keys {
                    writeln!(f, "{:<20} | {:<20}", key, data[key])?;
                }
            }
            Some(_) => {
                writeln!(f, "Status: Parsed, but no keys found.")?;
            }
            None => {
                if let Some(err) = self.get_error() {
                    writeln!(f, "Status: Error - {}", err)?;
                } else {
                    writeln!(f, "Status: Not yet parsed.")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_env(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_case_integers() {
        let file: NamedTempFile = create_test_env("port=8080\ninvalidations=\"18\"");
        let parser: EnvParser = EnvParser::from_file(file.path(), true);

        assert_eq!(parser.get_int::<i32>("port"), Some(8080));
        assert_eq!(parser.get_int::<i32>("invalidations"), Some(18));
    }

    #[test]
    fn test_case_floats() {
        let file: NamedTempFile = create_test_env("exchange_rate=16.2\ndivisions=\"13.1\"");
        let parser: EnvParser = EnvParser::from_file(file.path(), true);

        assert_eq!(parser.get_float::<f64>("exchange_rate"), Some(16.2));
        assert_eq!(parser.get_int::<f64>("divisions"), Some(13.1));
    }

    #[test]
    fn test_case_bools() {
        let file: NamedTempFile =
            create_test_env("has_data=yes\nis_male=no\nis_debug=true\nhas_active_users=\"no\"");
        let parser: EnvParser = EnvParser::from_file(file.path(), true);

        assert_eq!(parser.get_bool("has_data"), Some(true));
        assert_eq!(parser.get_bool("is_male"), Some(false));
        assert_eq!(parser.get_bool("is_debug"), Some(true));
        assert_eq!(parser.get_bool("has_active_users"), Some(false));
    }

    #[test]
    fn test_case_string() {
        let file: NamedTempFile = create_test_env("first_name=mike\nlast_name=dotnet");
        let parser: EnvParser = EnvParser::from_file(file.path(), false);

        assert_eq!(parser.get_str("first_name"), Some("mike".to_string()));
        assert_eq!(parser.get_str("last_name"), Some("dotnet".to_string()));
    }

    #[test]
    fn test_case_insensitivity() {
        let file: NamedTempFile =
            create_test_env("port=8080\nAPI_KEY=secret123\nEXCHANGE_RATE=12.2\nIS_DEBUG=false");
        let parser = EnvParser::from_file(file.path(), false);

        assert_eq!(parser.get_int::<i32>("port"), Some(8080));
        assert_eq!(parser.get_str("api_key"), Some("secret123".to_string()));
        assert_eq!(parser.get_float::<f64>("exchange_rate"), Some(12.2));
        assert_eq!(parser.get_bool("is_debug"), Some(false));
    }

    #[test]
    fn test_complex_types() {
        let file = create_test_env(
            "FLAGS=[run,build,test]\nMETADATA={version:1.0, env:prod, is_debug: true}",
        );
        let parser = EnvParser::from_file(file.path(), false);

        let list = parser.get_list("FLAGS").unwrap();
        assert_eq!(list, vec!["run", "build", "test"]);

        let dict = parser.get_dict("METADATA").unwrap();
        assert_eq!(dict.get("version").unwrap(), "1.0");
        assert_eq!(dict.get("env").unwrap(), "prod");
        assert_eq!(dict.get("is_debug").unwrap(), "true");
    }

    #[test]
    fn test_get_value_anyvalue() {
        let file = create_test_env("IS_ACTIVE=true\nTIMEOUT=30.5");
        let parser = EnvParser::from_file(file.path(), false);

        match parser.get_value("IS_ACTIVE", Types::Bool) {
            Some(AnyValue::Bool(b)) => assert!(b),
            _ => panic!("Expected AnyValue::Bool"),
        }

        match parser.get_value("TIMEOUT", Types::Double) {
            Some(AnyValue::Float(f)) => assert_eq!(f, 30.5),
            _ => panic!("Expected AnyValue::Float"),
        }
    }

    #[test]
    fn test_quoting_and_comments() {
        let file = create_test_env(
            "
            # This is a comment
            NAME=\"Rust Parser\"
            DESC='A simple tool'
            RAW=no_quotes
        ",
        );
        let parser = EnvParser::from_file(file.path(), false);

        assert_eq!(parser.get_str("NAME").unwrap(), "Rust Parser");
        assert_eq!(parser.get_str("DESC").unwrap(), "A simple tool");
        assert_eq!(parser.get_str("RAW").unwrap(), "no_quotes");
    }

    #[test]
    fn test_missing_file() {
        let parser = EnvParser::from_file("this_file_does_not_exist.env", false);
        assert!(parser.get_error().is_some());
        assert!(parser.get_data().is_none());
    }
}
