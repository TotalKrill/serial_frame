use serialport::prelude::*;
use simple_logger;
use std::sync::mpsc::*;
use std::time::Duration;

use serial_frame::{create_line_sender, Line, SerialFrameError, SerialFrameSender};

fn main() {
    // Setup the serialport to act on
    let serialport: Box<dyn SerialPort> = init();

    // get a Reciever for strings that all end with a newline
    let (rx, linestop) = create_line_sender(serialport).unwrap();

    // Recieve the lines, stop if timeout
    while let Ok(line) = rx.recv_timeout(Duration::from_secs(2)) {
        // Inspect the received line
        match line {
            Ok(line) => {
                println!("line is: {}", line);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
    // This will end the thread if it not stopped
    let e = linestop.stop();
    println!("Stop: {:?}", e);
}

fn init() -> Box<dyn SerialPort> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(100);
    let baudrate = 115200;
    settings.baud_rate = baudrate;
    let serialport = serialport::open_with_settings("/dev/ttyACM0", &settings).unwrap();
    serialport
}
