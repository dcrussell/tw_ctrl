use std::error::Error;
use std::process;
use tw_ctrl::config::Config;
use tw_ctrl::log;

//TODO: Add logger for output
fn main() {
    let config =
        Config::new(&"/home/dalton/Projects/tw_ctrl/config".to_string()).unwrap_or_else(|err| {
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
