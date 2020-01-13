use crate::Result;
use crate::{SerialFrameError, SerialFrameStopper};
use core::convert::TryFrom;
use derive_more::*;
use serialport::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Display, Debug, Into, From, PartialEq, Eq)]
pub struct Line(pub String);

impl TryFrom<Vec<u8>> for Line {
    type Error = crate::SerialFrameError;
    fn try_from(input: Vec<u8>) -> Result<Self> {
        let inputstr = String::from_utf8(input.clone())
            .map_err(|e| SerialFrameError::FailedConversion(input))?;
        Ok(Self(inputstr))
    }
}

/// Returns a Reciever that receives strings that ends with newlines from the specified serial port
pub fn create_line_sender(
    serialport: Box<dyn SerialPort>,
) -> Result<(Receiver<Result<Line>>, SerialFrameStopper)> {
    let (tx, rx) = channel();
    let sender = crate::SerialFrameSender::new(b'\n', serialport);
    let stopper = sender.start(tx)?;
    Ok((rx, stopper))
}

/// Returns a Receiver that receives u8 vectors that ends with binary
pub fn create_cobs_sender(
    serialport: Box<dyn SerialPort>,
) -> Result<(Receiver<Result<Vec<u8>>>, SerialFrameStopper)> {
    let (tx, rx) = channel();
    let sender = crate::SerialFrameSender::new(0, serialport);
    let stopper = sender.start(tx)?;
    Ok((rx, stopper))
}
