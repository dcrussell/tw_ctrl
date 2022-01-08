//! This module is responsible for the transmission of commands to the
//! station. It implements a simple trasport-like protocol for (mostly) reliable
//! communication. Each payload is wrapped into a frame (not to be confused
//! with a serial frame) that's then transported over the serial port. There
//! is a three byte header and a three byte trailer encompassing each payload.
//!
//!
//! *Header*
//!
//! The header is defined as:
//!
//! byte: [      1     ][      2     ][       3       ]
//!       [ Start Byte ][ Frame Type ][ Paylod Length ]
//!
//!
//! Start - 0x7f
//!
//! Frame Type - 0x44: Indicates that the frame is a data frame.
//!              This is the frame type used when commands are
//!              being sent to the station and data is sent back.
//!
//!              Ox43: Indicates that the frame is a control frame.
//!              Control frames are only used by the trasport layer
//!              to signal whether a frame was successfully recieved.
//!
//!
//! Paylod length - Obvious. Note that in this implementation payload length is
//!                 an 8 bit number so the maximum payload size allowed is
//!                 255 bytes.
//!
//!
//! *Trailer*
//!
//! The trailer is defined as:
//!
//! byte: [     1     ][      2     ][     3     ]
//!       [          CRC            ][  End byte ]
//!
//!
//! CRC - a 16 bit CRC check value used by the transport layer
//!       to verify clean transmission of data. The value is calculated
//!       over the payload portion only.
//!
//! End - 0xfe
//!
//!
//!
//! *Control Frames*
//!
//! There are currently five control frames used by the trasport layer -- one
//! ACK frame, three NACK frames, and one special heartbeat frame.
//! ACK frames communicate successful reciept of a data frame.
//! NACK frames hold the opposite meaning. They communicate error anytime the
//! transport layer runs into an issue. The heartbeat frame is a special frame
//! that is used to confirm that the recieving device is up and ready.
//! Each control frame uses the control frame identifier and utilizes
//! the payload portion of a frame to indicate which kind it is.
//! All control frames are 7 bytes long and have the following layout:
//!
//! [ 0x7f ][ 0x43 ][ length 1][Control frame identifier][ CRC ][ 0xfe ]
//!
//!
//! The set of control frame identifiers are:
//! ACK - 0x01: Acknowledge.
//!
//! CRCFAIL  - 0x02: The CRC check failed.
//!
//! OVERSIZE - 0x03: The frame being recieved is larger than the reciept will
//!                  handle.
//!
//! InvalidFrame - 0x04: The frame is missing either the start, the end, or
//!                      has the wrong frame type.
//!
//! Heartbeat - 0x05: Used to confirm that a connection has been established.
//!
//! *Communication*
//!
//! When one end of the channel receives a data frame the frame is checked for
//! against the layout and the checksum is calculated. The receiver sends an
//! ACK frame if it passes, otherwise it sends a NACK frame. Depending on how
//! the sender's channel is configured, the sender may re-attempt transmission
//! if a NACK is received.
//!
//!
//!
//!
//!
//!
//!

use std::usize;

use crate::crc16;
use crate::log;
use crate::serialport;

/// Frame constants
const FRAME_START: u8 = 0x7f;
const FRAME_END: u8 = 0xfe;
const FRAME_TYPE_DATA: u8 = 0x44;
const FRAME_TYPE_CTRL: u8 = 0x43;
const FRAME_SIZE_MAX: usize = 86;
const FRAME_CTRL_SIZE: usize = 7;

enum ControlType {
    Ack = 0x01,
    CRCFail = 0x02,
    Oversize = 0x03,
    InvalidFrame = 0x04,
    Heartbeat = 0x05,
}

pub struct Channel {
    port: serialport::SerialPort,
    num_attempts: u32,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: String,
}

impl Error {
    fn new(kind: ErrorKind, description: &str) -> Error {
        Error {
            kind,
            description: description.to_string(),
        }
    }
}

impl From<serialport::Error> for Error {
    fn from(e: serialport::Error) -> Error {
        Error {
            kind: ErrorKind::SerialPort(*e.kind()),
            description: e.desc().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NoAck,
    NoHeartBeat,
    Oversize,
    MaxAttempts,
    SerialPort(serialport::ErrorKind),
    InvalidFrame,
    CRCFail,
}

fn make_control_frame(ctype: ControlType) -> [u8; FRAME_CTRL_SIZE] {
    let mut frame: [u8; FRAME_CTRL_SIZE] = [0; FRAME_CTRL_SIZE];
    frame[0] = FRAME_START;
    frame[1] = FRAME_TYPE_CTRL;
    frame[2] = 0x01; // length of control frame payloads are always 1
    frame[3] = ctype as u8;

    let crc = crc16::crc16(&frame[3..4]);
    frame[4] = (crc & 0xff as u16) as u8;
    frame[5] = (crc >> 8) as u8;
    frame[6] = FRAME_END;
    frame
}

fn make_data_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame: Vec<u8> = Vec::new();
    frame.push(FRAME_START);
    frame.push(FRAME_TYPE_DATA);
    frame.push(payload.len() as u8);
    for i in payload.iter() {
        frame.push(*i);
    }
    let frame_crc = crc16::crc16(&frame[3..3 + payload.len()]);
    frame.push((frame_crc & 0xff as u16) as u8);
    frame.push((frame_crc >> 8) as u8);
    frame.push(FRAME_END);
    frame
}

pub type Result<T> = std::result::Result<T, Error>;

impl Channel {
    /// Create a new channel to the serial device
    pub fn new(port: serialport::SerialPort, num_attempts: u32) -> Channel {
        Channel { port, num_attempts }
    }

    /// Open the channel for communication
    pub fn open(&mut self) -> Result<()> {
        if let Err(e) = self.port.open() {
            return Err(Error::new(ErrorKind::SerialPort(*e.kind()), &e.to_string()));
        }
        // A heartbeat is used to confirm that the station is up.
        let mut n_attempts = 0;
        let mut n_bytes = 0;
        let mut frame: [u8; FRAME_CTRL_SIZE] = [0; FRAME_CTRL_SIZE];
        log::info("Attempting to establish a heartbeat..");
        while n_attempts < self.num_attempts && n_bytes < 7 {
            self.send_ctrl_frame(ControlType::Heartbeat)?;
            match self.port.read(&mut frame[n_bytes..7]) {
                Ok(n) => {
                    n_bytes += n;
                }
                Err(e) => {
                    log::debug(&format!("{:?}", e));
                }
            }
            n_attempts += 1;
            // Clear the IO queues on each attempt.
            self.port.flush()?;
        }
        if frame[1] != FRAME_TYPE_CTRL && frame[3] != ControlType::Heartbeat as u8 {
            self.port.close()?;
            log::error("Could not establish heartbeat");
            return Err(Error::new(
                ErrorKind::NoHeartBeat,
                "Failed to establish heartbeat",
            ));
        }
        log::info("Heartbeat confirmed");
        Ok(())
    }

    fn try_send(&self, frame: &[u8]) -> Result<()> {
        match self.port.write(&frame) {
            Ok(n) => log::debug(&format!("Sent bytes: {:?}", frame)),
            Err(e) => {
                log::error(&format!("{:?}", e));
                return Err(Error::new(ErrorKind::SerialPort(*e.kind()), &e.to_string()));
            }
        }
        let mut control: [u8; FRAME_CTRL_SIZE] = [0; FRAME_CTRL_SIZE];
        let mut nbytes = 0;
        while nbytes < FRAME_CTRL_SIZE {
            match self.port.read(&mut control[nbytes..FRAME_CTRL_SIZE]) {
                Ok(n) => {
                    nbytes = nbytes + n;
                }
                Err(e) => {
                    log::error(&format!("{:?}", e));
                    return Err(Error::new(ErrorKind::SerialPort(*e.kind()), &e.to_string()));
                }
            }
        }
        if control[0] != FRAME_START
            || control[1] != FRAME_TYPE_CTRL
            || control[3] != ControlType::Ack as u8
        {
            self.port.flush()?;
            return Err(Error::new(ErrorKind::NoAck, "ACK not recieved"));
        }
        Ok(())
    }
    ///Send the payload over the channel.
    pub fn send(&self, payload: &[u8]) -> Result<()> {
        if payload.len() > FRAME_SIZE_MAX - 6 {
            return Err(Error::new(
                ErrorKind::Oversize,
                "Payload larger than maximum payload size",
            ));
        }

        let frame = make_data_frame(payload);

        // send and listen for ACK or NACK
        let mut n_attempts = 0;
        while n_attempts < self.num_attempts {
            match self.try_send(&frame) {
                Ok(_) => return Ok(()),
                Err(e) => log::error(&format!("{:?}", e)),
            }
            n_attempts += 1;
        }
        Err(Error::new(
            ErrorKind::MaxAttempts,
            "Maximum number of resend attempts reached",
        ))
    }

    fn try_recv(&self) -> Result<Vec<u8>> {
        let mut frame: Vec<u8> = vec![0; FRAME_SIZE_MAX];
        let mut nbytes = 0;

        // pull in header
        while nbytes < 3 {
            match self.port.read(&mut frame[nbytes..3]) {
                Ok(n) => {
                    nbytes = nbytes + n;
                }
                Err(e) => {
                    log::error(&format!("{:?}", e));
                    break;
                }
            }
        }
        let payload_size: usize = {
            if frame[2] as usize > FRAME_SIZE_MAX - 6 {
                self.send_ctrl_frame(ControlType::Oversize);
                self.port.flush();
                return Err(Error::new(ErrorKind::Oversize, "Frame oversize"));
            } else {
                frame[2] as usize
            }
        };

        while nbytes < payload_size {
            match self.port.read(&mut frame[nbytes..payload_size + 6]) {
                Ok(n) => {
                    nbytes = nbytes + n;
                    log::debug(&format!("Recieved {} bytes", n));
                }
                Err(e) => log::error(&format!("Error {:?}", e)),
            }
        }
        if frame[0] != FRAME_START
            || frame[1] != FRAME_TYPE_DATA
            || frame[payload_size + 6 - 1] != FRAME_END
        {
            self.send_ctrl_frame(ControlType::InvalidFrame)?;
            self.port.flush();
            return Err(Error::new(
                ErrorKind::InvalidFrame,
                "Recieved frame is invalid",
            ));
        }

        let check: u16 = crc16::crc16(&frame[3..3 + payload_size]);
        let mut frame_crc: u16 = frame[payload_size + 6 - 3] as u16 & 0xff;
        frame_crc |= (frame[payload_size + 6 - 2] as u16) << 8;
        if check != frame_crc {
            self.send_ctrl_frame(ControlType::CRCFail);
            self.port.flush();
            return Err(Error::new(ErrorKind::CRCFail, "CRC check did not pass"));
        }
        self.send_ctrl_frame(ControlType::Ack);
        Ok(frame[3..3 + payload_size].to_vec())
    }

    pub fn recv(&self) -> Result<Vec<u8>> {
        let mut attempts = 0;
        while attempts < self.num_attempts {
            match self.try_recv() {
                Ok(v) => return Ok(v),
                Err(e) => log::error(&format!("channel: {:?}", e)),
            }
            attempts += 1;
        }
        Err(Error::new(
            ErrorKind::MaxAttempts,
            "Maximum number of recieve attempts reached",
        ))
    }
    pub fn send_heartbeat(&self) -> Result<()> {
        self.send_ctrl_frame(ControlType::Heartbeat)
    }
    fn send_ctrl_frame(&self, ctype: ControlType) -> Result<()> {
        let frame = make_control_frame(ctype);
        match self.port.write(&frame) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::SerialPort(*e.kind()), e.desc())),
        }
    }
}
