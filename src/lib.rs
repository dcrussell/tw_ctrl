use std::error::Error;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

use channel::Channel;
mod channel;
pub mod config;
mod crc16;
pub mod log;
mod serialize;
mod serialport;
mod termios;

/// Main function of execution.
pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let baud: u32 = match config.get("serial.baud") {
        Some(n) => n.parse()?,
        None => panic!("No rate listed in config"),
    };
    let device = match config.get("serial.device") {
        Some(d) => d,
        None => panic!("No device listed in config"),
    };

    //TODO: Parse log level and convert to
    //log::Level for logger

    //let logger = match config.get("log.file") {
    //    Some(f) => match config.get("log.level") {
    //        Some(lvl) => Some(log::file::Logger::new(f, lvl)),
    //        None => Some(log::file::Logger::new(f, log::Level::Debug)),
    //    },
    //    None => None,
    //};

    //TODO: Provide conversion function from
    // u32 to rate
    let rate = match baud {
        9600 => serialport::BaudRate::B9600,
        115200 => serialport::BaudRate::B115200,
        _ => panic!("Unsupported baud rate"),
    };

    let mut port = serialport::SerialPort::new(device, rate, Duration::from_secs(2))?;
    log::debug(&format!("Connecting to {}", device));
    port.open()?;

    let mut channel = Channel::new(port, 3);

    loop {
        sleep(Duration::from_secs(2));
        let mut payload: Vec<u8> = Vec::new();
        payload.push(0x02);
        match channel.send(&payload) {
            Ok(()) => log::debug("Send complete"),
            Err(e) => log::error(&format!(
                "Channel encountered error during sending: {:?}",
                e
            )),
        }

        let data = match channel.recv() {
            Ok(v) => v,
            Err(e) => {
                log::error(&format!("Channel encountered error during recv: {:?}", e));
                break;
            }
        };

        log::debug(&format!("Bytes : {:?}", data));

        let mut temp_u32: u32 = 0;
        for i in 0..4 {
            temp_u32 |= (data[i] as u32) << (8 * i);
        }
        let temp_f32: f32 = temp_u32 as i32 as f32 / 100.0;
        log::info(&format!("Temp is: {}", temp_f32));
    }

    Ok(())
}
