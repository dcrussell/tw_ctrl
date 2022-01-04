//! This module provides logging to a file and to std out
use chrono::prelude::*;
use chrono::{DateTime, Local};
use std::io::Write;
#[derive(PartialOrd, PartialEq)]
pub enum Level {
    Off,
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            Level::Off => "".to_string(),
            Level::Debug => "DEBUG".to_string(),
            Level::Info => "INFO".to_string(),
            Level::Warning => "WARN".to_string(),
            Level::Error => "ERROR".to_string(),
            Level::Fatal => "FATAL".to_string(),
        }
    }
}

pub mod file {

    use super::Level;
    use super::Write;
    use super::LOGLEVEL;
    use std::fs::{File, OpenOptions};

    type Error = std::io::Error;
    type Result<T> = std::result::Result<T, Error>;
    pub struct Logger {
        file: File,
        level: Level,
    }

    impl Logger {
        pub fn new(path: &str, level: Level) -> Result<Logger> {
            Ok(Logger {
                file: OpenOptions::new().append(true).create(true).open(path)?,
                level: if level > LOGLEVEL { LOGLEVEL } else { level },
            })
        }

        pub fn set_level(&mut self, level: Level) {
            self.level = if level > LOGLEVEL { LOGLEVEL } else { level };
        }

        pub fn log(&self, level: &Level, s: &str) -> Result<()> {
            let dt = chrono::Local::now().to_rfc3339();
            match level {
                Level::Off => (),
                _ => writeln!(&self.file, "[{}] [{}] {}", dt, level.to_string(), s)?,
            };

            Ok(())
        }
        pub fn debug(&self, s: &str) -> Result<()> {
            if Level::Debug <= self.level {
                self.log(&Level::Debug, &s)?;
            }
            Ok(())
        }
        pub fn info(&self, s: &str) -> Result<()> {
            if Level::Info <= self.level {
                self.log(&Level::Info, &s)?;
            }
            Ok(())
        }
        pub fn warn(&self, s: &str) -> Result<()> {
            if Level::Warning <= self.level {
                self.log(&Level::Warning, &s)?;
            }
            Ok(())
        }
        pub fn error(&self, s: &str) -> Result<()> {
            if Level::Error <= self.level {
                self.log(&Level::Error, &s)?;
            }
            Ok(())
        }
        pub fn fatal(&self, s: &str) -> Result<()> {
            if Level::Fatal <= self.level {
                self.log(&Level::Fatal, &s)?;
            }
            Ok(())
        }
    }
}

// Global log Level
const LOGLEVEL: Level = Level::Debug;

//#[macro_export]
//macro_rules! log {
//    ($($arg:tt)*) => {
//        let mut w = File::create("./test.txt").unwrap();
//        writeln!(&mut w, "{} {}", Debug.description, format_args!($($arg)*)).unwrap();
//    }
//}

pub fn log(level: &Level, s: &str) {
    let dt = chrono::Local::now().to_rfc3339();
    match level {
        Level::Off => (),
        _ => println!("[{}] [{}] {}", dt, level.to_string(), s),
    }
}

pub fn debug(s: &str) {
    if Level::Debug <= LOGLEVEL {
        log(&Level::Debug, &s);
    }
}

pub fn info(s: &str) {
    if Level::Info <= LOGLEVEL {
        log(&Level::Info, &s);
    }
}

pub fn warn(s: &str) {
    if Level::Warning <= LOGLEVEL {
        log(&Level::Warning, &s);
    }
}

pub fn error(s: &str) {
    if Level::Error <= LOGLEVEL {
        log(&Level::Error, &s);
    }
}

pub fn fatal(s: &str) {
    if Level::Fatal <= LOGLEVEL {
        log(&Level::Fatal, &s);
    }
}
