use serialport::SerialPort;

use log::*;
use std::thread;
use std::thread::JoinHandle;

use std::io::ErrorKind;
use std::sync::mpsc::*;

#[derive(Debug)]
pub enum SerialFrameError {
    CouldNotStart,
    CouldNotSendStop,
    SerialportDisconnected,
    SerialThreadPaniced,
    RecieverDropped,
}

/// Structure which can only be obtained by starting a SerialFrameSender structure, and can only be
/// used to stop the resulting thread from the SerialFrameSender::start method. When this structure
/// is dropped, the SerialFrameSender will also stop
pub struct SerialFrameStopper {
    handle: JoinHandle<()>,
    stopsender: Sender<()>,
}

impl SerialFrameStopper {
    pub fn stop(self) -> Result<(), SerialFrameError> {
        self.stopsender
            .send(())
            .map_err(|_e| SerialFrameError::CouldNotSendStop)?;
        self.handle
            .join()
            .map_err(|_e| SerialFrameError::SerialThreadPaniced)?;
        Ok(())
    }
}

/// The frame sender structure, this will create a SerialFrameSender, that once started will split
/// incoming bytes from the serialport and send them framed by the separator
///
/// Ex: "This is one line\nAnd this is another\n"
///
/// will return "This is one line\n", and "This is another\n" in two separate vectors over the
/// channel sent in when starting the thread
pub struct SerialFrameSender {
    separator: u8,
    port: Box<dyn SerialPort>,
}

impl SerialFrameSender {
    pub fn new(separator: u8, port: Box<dyn SerialPort>) -> SerialFrameSender {
        Self { separator, port }
    }

    /// Consumes the SerialFrameSender and creates a new running thread, that will send complete
    /// frames over the Channel it takes as input separated by the specified separator
    ///
    /// Returned is structure that can be used to stop this thread, and thus unblock the serialport
    /// or an error
    pub fn start(
        mut self,
        send: Sender<Result<Vec<u8>, SerialFrameError>>,
    ) -> Result<SerialFrameStopper, SerialFrameError> {
        let (stoptx, stoprx) = channel();

        let handle = thread::Builder::new()
            .name("SerialFrameSender".to_string())
            .spawn(move || {
                let mut buf: Vec<u8> = Vec::new();
                let mut serial_byte = [0; 10240];
                'thread: loop {
                    // Functionality to close the thread
                    match stoprx.try_recv() {
                        Err(TryRecvError::Empty) => {
                            match self.port.read(&mut serial_byte[..]) {
                                Ok(n) => {
                                    buf.extend_from_slice(&serial_byte[..n]);
                                }
                                Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                                    trace!("{}", e);
                                }
                                // ends up here if unplugged
                                Err(e) => {
                                    error!("{}", e);
                                    let res =
                                        send.send(Err(SerialFrameError::SerialportDisconnected));
                                    if let Err(e) = res {
                                        error!("Could not send error, quitting: {}", e);
                                        break 'thread;
                                    }
                                    break 'thread;
                                }
                            }

                            while let Some(end) = buf.iter().position(|&f| f == self.separator) {
                                trace!("end: {}", end);
                                let line: Vec<u8> = buf.drain(..end + 1).collect();
                                trace!("Line: {:?}", line);
                                let res = send.send(Ok(line));
                                if let Err(e) = res {
                                    error!("Could not send frame, quitting: {}", e);
                                    break 'thread;
                                }
                            }
                        }
                        Err(TryRecvError::Disconnected) => {
                            info!("Thread handle was dropped");
                        }
                        _ => {
                            info!("Thread got stop request");
                            break 'thread;
                        }
                    }
                }
            });

        let handle = handle.map_err(|_e| SerialFrameError::CouldNotStart)?;

        let stopsend = SerialFrameStopper {
            handle,
            stopsender: stoptx,
        };

        Ok(stopsend)
    }
    pub fn stop(&mut self) -> () {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
