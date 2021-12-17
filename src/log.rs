//! This module provides logging to a file and to std out
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

        pub fn log(&mut self, level: &Level, s: &str) -> Result<()> {
            match level {
                Level::Off => (),
                Level::Debug => writeln!(&mut self.file, "[DEBUG] {}", s)?,
                Level::Info => writeln!(&mut self.file, "[INFO] {}", s)?,
                Level::Warning => writeln!(&mut self.file, "[WARN] {}", s)?,
                Level::Error => writeln!(&mut self.file, "[ERROR] {}", s)?,
                Level::Fatal => writeln!(&mut self.file, "[FATAL] {}", s)?,
            };

            Ok(())
        }
        pub fn debug(&mut self, s: &str) -> Result<()> {
            if Level::Debug <= self.level {
                self.log(&Level::Debug, &s)?;
            }
            Ok(())
        }
        pub fn info(&mut self, s: &str) -> Result<()> {
            if Level::Info <= self.level {
                self.log(&Level::Info, &s)?;
            }
            Ok(())
        }
        pub fn warn(&mut self, s: &str) -> Result<()> {
            if Level::Warning <= self.level {
                self.log(&Level::Warning, &s)?;
            }
            Ok(())
        }
        pub fn error(&mut self, s: &str) -> Result<()> {
            if Level::Error <= self.level {
                self.log(&Level::Error, &s)?;
            }
            Ok(())
        }
        pub fn fatal(&mut self, s: &str) -> Result<()> {
            if Level::Fatal <= self.level {
                self.log(&Level::Fatal, &s)?;
            }
            Ok(())
        }
    }
}

const LOGLEVEL: Level = Level::Debug;

//#[macro_export]
//macro_rules! log {
//    ($($arg:tt)*) => {
//        let mut w = File::create("./test.txt").unwrap();
//        writeln!(&mut w, "{} {}", Debug.description, format_args!($($arg)*)).unwrap();
//    }
//}

pub fn log(level: &Level, s: &str) {
    match level {
        Level::Off => (),
        Level::Debug => println!("[DEBUG] {}", s),
        Level::Info => println!("[INFO] {}", s),
        Level::Warning => println!("[WARN] {}", s),
        Level::Error => println!("[ERROR] {}", s),
        Level::Fatal => println!("[FATAL] {}", s),
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
