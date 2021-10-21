mod config;
fn main() {
    let run_args = config::parse(&"config".to_string()).expect("Wut");
    println!(
        "Serial port device: {}",
        run_args.get("serial.device").unwrap()
    );
}
