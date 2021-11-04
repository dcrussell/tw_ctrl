use std::error::Error;

pub mod channel;
pub mod config;
pub mod message;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    // 1. get config args
    // 2. loop

    Ok(())
}
