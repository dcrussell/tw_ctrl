mod config;
use serialport;
use std::{thread, time};
fn main() {
    let mut port = serialport::new("/dev/tty1", 9600)
        .open()
        .expect("Failed to open port");
    //    let run_args = config::parse(&"config".to_string()).expect("Wut?");
    loop {
        let out = "Hello there!".as_bytes();
        port.write(out).expect("write failed");
        thread::sleep(time::Duration::from_millis(1000));
    }
}
