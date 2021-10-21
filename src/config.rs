use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn filter_comments(line: String) -> String {
    let comment_pos = match line.find("#") {
        Some(i) => i,
        None => return line,
    };
    //TODO:  Maybe a better way to do this?
    let filtered = line[..comment_pos].to_string();
    filtered.trim().to_string()
}

fn parse_line(map: &mut HashMap<String, String>, line: String) {
    let filtered = filter_comments(line);
    // TODO: Find works fine here with ASCII text
    // but fails to work with unicode. Need
    // to find a better way to do this
    let sep_position = match filtered.find("=") {
        Some(i) => i,
        None => return,
    };
    if !filtered[..sep_position].is_empty() && !filtered[sep_position + 1..].is_empty() {
        map.insert(
            filtered[..sep_position].to_string(),
            filtered[sep_position + 1..].to_string(),
        );
    }
}

pub fn parse(path: &String) -> Result<HashMap<String, String>, std::io::Error> {
    let mut kv_pairs = HashMap::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        parse_line(&mut kv_pairs, String::from(line?));
    }
    return Ok(kv_pairs);
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
    fn test_invalid_path() {
        let res = crate::config::parse(&String::from("none.text")).map_err(|e| e.kind());
        let expected = Err(std::io::ErrorKind::NotFound);
        assert_eq!(expected, res);
    }

    //Empty hashmap is returned if file is empty
    #[test]
    fn test_empty_file() {
        let file = String::from("test1");
        create_empty(&file);
        let res = crate::config::parse(&file).unwrap();
        assert_eq!(0, res.len());
        delete_file(&file);
    }

    // comments (#) on their own line are ignored
    #[test]
    fn test_ignore_comments() {
        let file = String::from("test2");
        let comment = String::from("#Test comment");
        create_empty(&file);
        write(&file, &comment);
        let res = crate::config::parse(&file).unwrap();
        assert_eq!(0, res.len());
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
        let res = crate::config::parse(&file).unwrap();
        assert_eq!(1, res.len());
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
        let res = crate::config::parse(&file).unwrap();
        assert_eq!(1, res.len());
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
        let res = crate::config::parse(&file).unwrap();
        assert_eq!(0, res.len());
        assert!(!res.get("key").is_some());
    }
}
