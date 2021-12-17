//! Module used for parsing the config file
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct Config {
    kv_pairs: HashMap<String, String>,
}

struct KVPair {
    key: String,
    value: String,
}
impl Config {
    /// Retuns a Config parsed from the file path provided
    pub fn new(path: &str) -> Result<Config, std::io::Error> {
        let file = File::open(path)?;
        let mut config = Config {
            kv_pairs: HashMap::new(),
        };

        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            match parse_line(String::from(line?)) {
                None => None,
                Some(pair) => config.kv_pairs.insert(pair.key, pair.value),
            };
        }

        return Ok(config);
    }
    ///Return a value for a key if it exists.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.kv_pairs.get(key)
    }
}

fn filter_comments(line: &str) -> String {
    let comment_pos = match line.find("#") {
        Some(i) => i,
        None => return line.to_string(),
    };
    //TODO:  Maybe a better way to do this?
    let filtered = line[..comment_pos].to_string();
    filtered.trim().to_string()
}

fn parse_line(line: String) -> Option<KVPair> {
    let filtered = filter_comments(&line);
    // TODO: Find works fine here with ASCII text
    // but fails to work with unicode. Need
    // to find a better way to do this
    let sep_position = match filtered.find("=") {
        Some(i) => i,
        None => return None,
    };
    if !filtered[..sep_position].is_empty() && !filtered[sep_position + 1..].is_empty() {
        return Some(KVPair {
            key: filtered[..sep_position].to_string(),
            value: filtered[sep_position + 1..].to_string(),
        });
    }
    return None;
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use std::io::Write;
    fn create_empty(s: &String) {
        File::create(s).expect("Error creating test cfg file");
    }

    fn delete_file(s: &String) {
        fs::remove_file(s).expect("Unable to remove test cfg");
    }
    fn write(file: &String, s: &String) {
        let mut f = fs::OpenOptions::new().append(true).open(&file).unwrap();
        f.write(s.as_bytes()).unwrap();
        f.write("\n".as_bytes()).unwrap();
    }

    // Invalid paths should return an Error
    #[test]
    #[should_panic]
    fn test_invalid_path() {
        //TODO: Implement PartialEq?
        let res = Config::new(&String::from("none.text")).unwrap();
        //let expected = Err(std::io::ErrorKind::NotFound);
        //assert_eq!(expected, res);
    }

    //Empty hashmap is returned if file is Empty
    //    #[test]
    //    fn test_empty_file() {
    //        let file = String::from("test1");
    //        create_empty(&file);
    //        let res = Config::new(&file).unwrap();
    //        assert_eq!(0, res.len());
    //        delete_file(&file);
    //    }

    // comments (#) on their own line are ignored
    #[test]
    fn test_ignore_comments() {
        let file = String::from("test2");
        let comment = String::from("#Test comment");
        create_empty(&file);
        write(&file, &comment);
        let res = Config::new(&file).unwrap();
        let value = res.get("#Test comment");
        assert!(!value.is_some());
        delete_file(&file);
    }

    // can parse key value pairs
    #[test]
    fn test_config() {
        let file = String::from("test3");
        let kv_pair = String::from("key=value");
        create_empty(&file);
        write(&file, &kv_pair);
        let res = Config::new(&file).unwrap();
        let value = res.get("key");
        assert_eq!(Some(&String::from("value")), value);
        delete_file(&file);
    }

    // Comments on the same line as kv pairs
    // should be ignored
    #[test]
    fn test_shared_lines() {
        let file = String::from("test4");
        let kv_pair = String::from("key=value # A comment");
        let kv_pair2 = String::from("#key1=value1");
        create_empty(&file);
        write(&file, &kv_pair);
        write(&file, &kv_pair2);
        let res = Config::new(&file).unwrap();
        let value = res.get("key");
        assert_eq!(Some(&String::from("value")), value);
        assert!(!res.get("#key1").is_some());
        delete_file(&file);
    }

    // KV pairs with out a key or value are ignored
    // e.g key= or =value
    #[test]
    fn test_invalid_kvpairs() {
        let file = String::from("test5");
        let invalid = String::from("key=");
        let invalid1 = String::from("=value");
        create_empty(&file);
        write(&file, &invalid);
        write(&file, &invalid1);
        let res = Config::new(&file).unwrap();
        assert!(!res.get("key").is_some());
        delete_file(&file);
    }
}
