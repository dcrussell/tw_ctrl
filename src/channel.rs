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
//! There are currently two control frames used by the trasport layer -- ACK
//! frames and NACK frames.Each control frame uses the control frame type i
//! dentifier and utilizes the payload portion of a frame to indicate which
//! kind it is,as well as provide additional data.
//!
//! ACK frames communicate successful reciept of a data frame. All ACK frames
//! are the same:
//!
//! [ 0x7f ][ 0x43 ][ length 1][ACK ID (0x01)][ CRC ][ 0xfe ]
//!
//! NACK frames hold the opposite meaning. They communicate error anytime the
//! transport layer runs into an issue. Each NACK frame has a one byte NACK
//! frame ID indicating the type of NACK error
//!
//! The geneal layout for a NACK frame is then:
//!
//! [ 0x7f ][ 0x43 ][ length: 1 ][ NACK ID][ CRC ][ 0xfe ]
//!
//!
//! The set of NACK IDs are:
//!
//! CRCFAIL  - 0x02: The CRC check failed.
//!
//! OVERSIZE - 0x03: The frame being recieved is larger than the reciept will
//!                  handle.
//!
//! InvalidFrame - 0x04: The frame is missing either the start, the end, or
//!                      has the wrong frame type.
//!
//!
//! *Communication*
//! TODO: Write this.
//!
//!
//!
//!
//!
//!

use crate::crc16;
use crate::log;

/// Frame constants
const FRAME_START: u8 = 0x7f;
const FRAME_END: u8 = 0xfe;
const FRAME_TYPE_DATA: u8 = 0x44;
const FRAME_TYPE_CTRL: u8 = 0x43;
const FRAME_SIZE_MAX: usize = 86;

enum ControlType {
    Ack = 0x01,
    CRCFail = 0x02,
    Oversize = 0x03,
    InvalidFrame = 0x04,
}

use crate::serialport;
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

#[derive(Debug)]
pub enum ErrorKind {
    NoAck,
    Oversize,
    MaxAttempts,
    SerialPort(serialport::ErrorKind),
}

fn make_control_frame(ctype: ControlType) -> [u8; 7] {
    let mut frame: [u8; 7] = [0; 7];
    frame[0] = FRAME_START;
    frame[1] = FRAME_TYPE_CTRL;
    frame[2] = 0x01; // length
    frame[3] = ctype as u8;

    let crc = crc16::crc16(&frame[3..4]);
    frame[4] = (crc & 0xff as u16) as u8;
    frame[5] = (crc >> 8) as u8;
    frame[6] = FRAME_END;
    frame
}
pub type Result<T> = std::result::Result<T, Error>;
impl Channel {
    pub fn new(port: serialport::SerialPort, num_attempts: u32) -> Channel {
        Channel { port, num_attempts }
    }

    pub fn send(&self, payload: &[u8]) -> Result<()> {
        use crate::crc16;
        let mut frame: [u8; FRAME_SIZE_MAX] = [0; FRAME_SIZE_MAX];
        let payload_len = payload.len();
        if payload_len > FRAME_SIZE_MAX - 6 {
            return Err(Error::new(
                ErrorKind::Oversize,
                "Payload larger than maximum payload size",
            ));
        }
        frame[0] = 0x7f;
        frame[1] = FRAME_TYPE_DATA;
        frame[2] = payload_len as u8; // Length
        for i in 0..payload.len() {
            frame[3 + i] = payload[i];
        }

        let frame_crc = crc16::crc16(&frame[3..3 + payload_len]);
        frame[3 + payload_len] = (frame_crc & 0xff as u16) as u8;
        frame[3 + payload_len + 1] = (frame_crc >> 8) as u8;
        frame[3 + payload_len + 2] = 0xfe;
        log::debug(&format!("Sending command {:?}", &frame[..payload_len + 6]));

        // send and listen for ACK or NACK
        let mut n_attempts = 0;
        while n_attempts < 3 {
            match self.port.write(&frame[..6 + payload_len]) {
                Ok(n) => log::debug(&format!("Sent bytes: {}", n)),
                Err(e) => {
                    log::debug(&format!("Error: {:?}", e));
                    return Err(Error::new(ErrorKind::SerialPort(*e.kind()), &e.to_string()));
                }
            }
            let mut control: [u8; FRAME_SIZE_MAX] = [0; FRAME_SIZE_MAX];
            let mut nbytes = 0;
            // pull in header
            while nbytes < 3 {
                match self.port.read(&mut control[nbytes..3]) {
                    Ok(n) => {
                        nbytes = nbytes + n;
                        log::debug(&format!("Recieved {} bytes", n));
                    }
                    Err(e) => log::error(&format!("Error {:?}", e)),
                }
            }
            if control[0] != 0x7f || control[1] != FRAME_TYPE_CTRL {
                self.port.flush();
            } else {
                let payload_len = control[2] as usize;
                //get the remaining parts of the frame
                nbytes = 3;
                while nbytes < payload_len as usize + 6 {
                    match self.port.read(&mut control[nbytes..payload_len + 6]) {
                        Ok(n) => {
                            nbytes = nbytes + n;
                            log::debug(&format!("Recieved {} bytes", n));
                        }
                        Err(e) => log::error(&format!("Error {:?}", e)),
                    }
                }

                if control[3] == 0x01 {
                    log::debug("Recieved ACK");
                    return Ok(());
                }
            }
            n_attempts += 1;
        }
        Err(Error::new(
            ErrorKind::NoAck,
            "Failed recieving ACK after command",
        ))
    }
    pub fn recv(&self) -> Result<Vec<u8>> {
        use crate::crc16;
        let mut attempts = 0;
        while attempts < self.num_attempts {
            let mut frame: Vec<u8> = vec![0; FRAME_SIZE_MAX];
            let mut nbytes = 0;

            // pull in header
            while nbytes < 3 {
                match self.port.read(&mut frame[nbytes..3]) {
                    Ok(n) => {
                        nbytes = nbytes + n;
                        log::debug(&format!("Recieved {} bytes", n));
                    }
                    Err(e) => {
                        log::error(&format!("{:?}", e));
                        break;
                    }
                }
            }
            if frame[0] != FRAME_START || frame[1] != FRAME_TYPE_DATA {
                self.send_ctrl_frame(ControlType::InvalidFrame)?;
                self.port.flush();
            } else if frame[2] as usize > FRAME_SIZE_MAX - 6 {
                self.send_ctrl_frame(ControlType::Oversize);
                self.port.flush();
            } else {
                let payload_len = frame[2] as usize;

                //get the remaining parts of the frame
                nbytes = 3;
                while nbytes < payload_len as usize + 6 {
                    match self.port.read(&mut frame[nbytes..payload_len + 6]) {
                        Ok(n) => {
                            nbytes = nbytes + n;
                            log::debug(&format!("Recieved {} bytes", n));
                        }
                        Err(e) => log::error(&format!("Error {:?}", e)),
                    }
                }
                if frame[payload_len + 6 - 1] != FRAME_END {
                    self.send_ctrl_frame(ControlType::InvalidFrame)?;
                    self.port.flush();
                } else {
                    let check: u16 = crc16::crc16(&frame[3..3 + payload_len]);
                    let mut frame_crc: u16 = frame[payload_len + 6 - 3] as u16 & 0xff;
                    frame_crc |= (frame[payload_len + 6 - 2] as u16) << 8;
                    if check != frame_crc {
                        self.send_ctrl_frame(ControlType::CRCFail);
                        self.port.flush();
                    } else {
                        self.send_ctrl_frame(ControlType::Ack);
                        return Ok(frame[3..3 + payload_len].to_vec());
                    }
                }
            }
            attempts += 1;
        }
        Err(Error::new(
            ErrorKind::MaxAttempts,
            "Maximum number of recieve attempts reached",
        ))
    }
    fn send_ctrl_frame(&self, ctype: ControlType) -> Result<()> {
        let frame = make_control_frame(ctype);
        match self.port.write(&frame) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::SerialPort(*e.kind()), e.desc())),
        }
    }
}
