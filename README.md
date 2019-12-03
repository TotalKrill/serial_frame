
# SerialFrame

[![Latest version](https://img.shields.io/crates/v/serial_frame.svg)](https://crates.io/crates/serial_frame)
![License](https://img.shields.io/crates/l/serial_frame.svg)

Simple serialport frame reciever, that asynchrounous sends chunks of bytes over an mpsc channel.
the chunks are sent with the chosen delimiter.

Can be used to recieve lines over serialports in an asynchronous manner, or to recieve cobs messages

## Example

```rust
use serialport::*;

use serial_frame::SerialFrameSender;

use std::sync::mpsc::*;
use std::time::Duration;

use simple_logger;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(100);
    let baudrate = 115200;
    settings.baud_rate = baudrate;
    let serialport = serialport::open_with_settings("/dev/ttyACM0", &settings).unwrap();

    // Send chunks that all end with a newline
    let linesend = SerialFrameSender::new(b'\n', serialport);
    let (tx, rx) = channel();
    let linestop = linesend.start(tx).unwrap();

    // Recieve the lines, stop if timeout
    while let Ok(line) = rx.recv_timeout(Duration::from_secs(2)) {
        // Inspect the received line
        match line {
            Ok(line) => {
                println!("line is: {}", String::from_utf8_lossy(&line));
            }
            Err(e) => {
                // An error in the sender has occured, the thread will be dead here
                println!("Error: {:?}", e);
            }
        }
    }
    // This will end the thread if it not stopped
    let e = linestop.stop();
    println!("Stop: {:?}", e);
}

```


