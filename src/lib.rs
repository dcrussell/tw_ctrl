use channel::serial::Channel;
use serialport;
use std::error::Error;
use std::{thread, time};
pub mod channel;
pub mod config;
pub mod message;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    // 1. get config args
    // 2. loop
    let baud: u32 = match config.get("serial.baud") {
        Some(n) => n.parse()?,
        None => panic!("No rate listed in config"),
    };
    let device = match config.get("serial.device") {
        Some(d) => d,
        None => panic!("No device listed in config"),
    };
    let port = serialport::new(device, baud)
        .timeout(time::Duration::from_secs(2))
        .open_native()?;

    let mut channel = Channel::new(port, 5);

    loop {
        let t = time::Duration::from_secs(1);
        let msg = message::Message::new(message::MessageId::CmdTph);

        match channel.send(msg) {
            Ok(_) => println!("Success send"),
            Err(e) => println!("Error: {}", e),
        }
        //let reply = match channel.listen() {
        //    Ok(_) => println!("Success recv"),
        //    Err(e) => println!("Fail recv: {}", e),
        //};
        thread::sleep(t);
    }

    Ok(())
}
