use reqwest;
use std::error::Error;
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

#[derive(Debug)]
enum Commands {
    Reset = 0x01,
    ReqTPH = 0x02,
    ReqT = 0x03,
    ReqP = 0x04,
    ReqH = 0x05,
}

fn str_to_loglvl(s: &str) -> log::Level {
    match s.to_lowercase().as_str() {
        "debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warning" => log::Level::Warning,
        "error" => log::Level::Error,
        "fatal" => log::Level::Fatal,
        "off" => log::Level::Off,
        _ => panic!("Not an available log level: {}", s),
    }
}

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

    let logger = match config.get("log.file") {
        Some(f) => match config.get("log.level") {
            Some(lvl) => Some(log::file::Logger::new(f, str_to_loglvl(lvl))?),
            None => Some(log::file::Logger::new(f, log::Level::Debug)?),
        },
        None => None,
    };

    let timeout: u64 = match config.get("serial.timeout") {
        Some(n) => n.parse()?,
        None => 0,
    };

    //TODO: Provide conversion function from
    // u32 to rate
    let rate = match baud {
        9600 => serialport::BaudRate::B9600,
        115200 => serialport::BaudRate::B115200,
        _ => panic!("Unsupported baud rate"),
    };

    let port = serialport::SerialPort::new(device, rate, Duration::from_secs(timeout));

    if let Some(l) = &logger {
        l.info(&format!("Opening connection to {}", device));
    }

    let mut channel = Channel::new(port, 5);
    if let Err(e) = channel.open() {
        if let Some(l) = &logger {
            l.fatal(&format!("Could not open channel to device: {:?}", e));
        }
        panic!("Could not open channel to device: {:?}", e);
    }

    if let Some(l) = &logger {
        l.info("Connected!");
    }

    loop {
        sleep(Duration::from_secs(2));
        let mut payload: Vec<u8> = Vec::new();
        if let Some(l) = &logger {
            l.info(&format!("Sending command {:?}", Commands::ReqTPH));
        }
        //TODO Actual commands
        payload.push(Commands::ReqTPH as u8);
        match channel.send(&payload) {
            Ok(()) => {
                if let Some(l) = &logger {
                    l.info("Send complete");
                }
            }
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

        if let Some(l) = &logger {
            l.info(&format!("Recieved data: {:?}", data));
        }

        let mut temp_u32: u32 = 0;
        let mut press_u32: u32 = 0;
        let mut hum_u32: u32 = 0;
        for i in 0..4 {
            temp_u32 |= (data[i] as u32) << (8 * i);
        }
        for i in 0..4 {
            press_u32 |= (data[4 + i] as u32) << (8 * i);
        }
        for i in 0..4 {
            hum_u32 |= (data[8 + i] as u32) << (8 * i);
        }
        let temp_f32: f32 = temp_u32 as i32 as f32 / 100.0;
        let press_f32: f32 = press_u32 as i32 as f32 / 256.0;
        let hum_f32: f32 = hum_u32 as i32 as f32 / 1024.0;
        log::info(&format!(
            "Temp: {}, Press: {}, Hum: {}",
            temp_f32, press_f32, hum_f32
        ));

        let dt: chrono::DateTime<chrono::Local> = chrono::Local::now();

        let data = format!(
            "envSensor,node=1 temperature={},humidity={},pressure={} {}",
            temp_f32,
            hum_f32,
            press_f32,
            dt.timestamp()
        );
        //Send data to influxDB
        //
        log::debug(&format!("Writing data to Influx: {}", data));
        let addr = config.get("db.host").unwrap();
        let port = config.get("db.port").unwrap();
        let api_key = config.get("db.api.key").unwrap();
        let api_endpoint = config.get("db.api.endpoint").unwrap();
        let api = InfluxWebClient {
            host: Host {
                addr: addr.to_string(),
                port: port.parse()?,
            },
            api_key: api_key.to_string(),
            api_endpoint: api_endpoint.to_string(),
        };
        log::info(&format!("{:?}", api.send(data)));
    }

    Ok(())
}

struct Host {
    addr: String,
    port: u32,
}
struct InfluxWebClient {
    host: Host,
    api_key: String,
    api_endpoint: String,
}

impl InfluxWebClient {
    fn send(&self, data: String) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        client
            .post(
                "http://".to_string()
                    + &self.host.addr
                    + ":"
                    + &self.host.port.to_string()
                    + &self.api_endpoint,
            )
            .header("Authorization", "Token ".to_string() + &self.api_key)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(data)
            .send()
    }
}
