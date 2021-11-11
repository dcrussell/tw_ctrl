use serialport;
use std::error::Error;
use std::process;
use tw_ctrl::config::Config;

//TODO: Add logger for output
fn main() {
    let config =
        Config::new(&"/home/drussell/Projects/tw_ctrl/config".to_string()).unwrap_or_else(|err| {
            println!("Error processing config file: {}", err);
            process::exit(1);
        });

    // Run the controller
    if let Err(e) = tw_ctrl::run(config) {
        println!("Error running controller: {}", e);
        process::exit(1);
    }
}
