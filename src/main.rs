use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;
use tw_ctrl::config::Config;
use tw_ctrl::log;

//TODO: Add logger for output
fn main() {
    let mut dir = env::current_exe().expect("How did we get here?");
    dir.pop();
    dir.push("config");
    let config = Config::new(&dir.to_str().unwrap().to_string()).unwrap_or_else(|err| {
        log::fatal(&format!(
            "Failed opening config file -- {}",
            err.to_string()
        ));
        process::exit(1);
    });

    // Run the controller
    if let Err(e) = tw_ctrl::run(config) {
        log::fatal(&format!(
            "Contoller encountered error during execution -- {}",
            e.to_string()
        ));
        process::exit(1);
    }
}
