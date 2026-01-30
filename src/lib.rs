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
#[derive(Debug)]
pub struct EnvParser {
    file_path: PathBuf,
    env_contents: Option<HashMap<String, String>>,
    is_debug: bool,
}

impl EnvParser {
    pub fn from_file<P: AsRef<Path>>(file_path: P, is_debug: bool) -> Result<Self, Box<dyn Error>> {
        let mut parser: EnvParser = Self::new(file_path, is_debug);
        parser.parse()?;
        Ok(parser)
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

    pub fn print_contents(&self) {
        println!("{:<20} | {:<20}", "VARIABLE", "VALUE");
        println!("{}", "-".repeat(43));

        if let Some(data) = &self.env_contents {
            if data.is_empty() {
                println!("(no variables found in file)");
            } else {
                let mut keys: Vec<&String> = data.keys().collect();
                keys.sort();

                for key in keys {
                    println!("{:<20} | {:<20}", key, data[key]);
                }
            }
        }

        println!("{}", "-".repeat(43));
    }

    fn new<P: AsRef<Path>>(file_path: P, is_debug: bool) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
            env_contents: None,
            is_debug,
        }
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(&self.file_path).map_err(|e| {
            if self.is_debug {
                eprintln!("DEBUG: Failed to open {:?}: {}", self.file_path, e);
            }
            e
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            if self.is_debug {
                eprintln!("DEBUG: Failed to read {:?}: {}", self.file_path, e);
            }
            e
        })?;

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
        Ok(())
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
        writeln!(f, "EnvParser: (file: {})", self.file_path.display())?;
        writeln!(f, "{}", "-".repeat(43))?;

        match &self.env_contents {
            Some(data) if !data.is_empty() => {
                let mut keys: Vec<&String> = data.keys().collect();
                keys.sort();
                for key in keys {
                    writeln!(f, "{:<20} | {:<20}", key, data[key])?;
                }
            }
            Some(_) => {
                writeln!(f, "(no variables found in file)")?;
            }
            None => {
                writeln!(f, "status: uninitialized")?;
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
    fn test_missing_file() {
        let result = EnvParser::from_file("this_file_does_not_exist.env", false);
        assert!(result.is_err());

        let err: Box<dyn Error> = result.unwrap_err();
        assert!(err.to_string().contains("No such file or directory"));
    }

    #[test]
    fn test_valid_file() {
        let file = create_test_env("PORT=8080");
        let parser = EnvParser::from_file(file.path(), false).expect("Failed to parse");
        assert_eq!(parser.get_int::<i32>("PORT"), Some(8080));
    }

    #[test]
    fn test_case_integers() {
        let file: NamedTempFile = create_test_env("port=8080\ninvalidations=\"18\"");
        let parser: EnvParser =
            EnvParser::from_file(file.path(), true).expect("Failed to load env file");

        assert_eq!(parser.get_int::<i32>("port"), Some(8080));
        assert_eq!(parser.get_int::<i32>("invalidations"), Some(18));
    }

    #[test]
    fn test_case_floats() {
        let file: NamedTempFile = create_test_env("exchange_rate=16.2\ndivisions=\"13.1\"");
        let parser: EnvParser =
            EnvParser::from_file(file.path(), true).expect("Failed to load env file");

        assert_eq!(parser.get_float::<f64>("exchange_rate"), Some(16.2));
        assert_eq!(parser.get_int::<f64>("divisions"), Some(13.1));
    }

    #[test]
    fn test_case_bools() {
        let file: NamedTempFile =
            create_test_env("has_data=yes\nis_male=no\nis_debug=true\nhas_active_users=\"no\"");
        let parser: EnvParser =
            EnvParser::from_file(file.path(), true).expect("Failed to load env file");

        assert_eq!(parser.get_bool("has_data"), Some(true));
        assert_eq!(parser.get_bool("is_male"), Some(false));
        assert_eq!(parser.get_bool("is_debug"), Some(true));
        assert_eq!(parser.get_bool("has_active_users"), Some(false));
    }

    #[test]
    fn test_case_string() {
        let file: NamedTempFile = create_test_env("first_name=mike\nlast_name=dotnet");
        let parser: EnvParser =
            EnvParser::from_file(file.path(), false).expect("Failed to load env file");

        assert_eq!(parser.get_str("first_name"), Some("mike".to_string()));
        assert_eq!(parser.get_str("last_name"), Some("dotnet".to_string()));
    }

    #[test]
    fn test_case_insensitivity() {
        let file: NamedTempFile =
            create_test_env("port=8080\nAPI_KEY=secret123\nEXCHANGE_RATE=12.2\nIS_DEBUG=false");
        let parser: EnvParser =
            EnvParser::from_file(file.path(), false).expect("Failed to load env file");

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
        let parser: EnvParser =
            EnvParser::from_file(file.path(), false).expect("Failed to load env file");

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
        let parser: EnvParser =
            EnvParser::from_file(file.path(), false).expect("Failed to load env file");

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
        let parser: EnvParser =
            EnvParser::from_file(file.path(), false).expect("Failed to load env file.");

        assert_eq!(parser.get_str("NAME").unwrap(), "Rust Parser");
        assert_eq!(parser.get_str("DESC").unwrap(), "A simple tool");
        assert_eq!(parser.get_str("RAW").unwrap(), "no_quotes");
    }
}
