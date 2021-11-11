pub mod serial {
    use std::error::Error;
    use std::io::{Read, Write};

    use crate::message::{Message, MessageId};
    use serialport;

    const MAX_FRAME_SIZE: u8 = 86;
    const MIN_FRAME_SIZE: u8 = 6; // Assume a minimum
    const FRAME_START: u8 = 0x7f;
    const FRAME_END: u8 = 0xfe;
    const FRAME_ACK: u8 = 0x01;
    const FRAME_NACK: u8 = 0x02;
    const ACK_FRAME: [u8; 7] = [FRAME_START, 0x43, 0x01, FRAME_ACK, 0x72, 0x26, FRAME_END];

    enum FrameType {
        Control,
        Data,
    }

    impl FrameType {
        fn value(&self) -> u8 {
            match *self {
                FrameType::Control => 0x43,
                FrameType::Data => 0x44,
            }
        }

        fn from_value(value: &u8) -> Result<FrameType, &'static str> {
            match &value {
                0x43 => Ok(FrameType::Control),
                0x44 => Ok(FrameType::Data),
                _ => Err("Value is not a valid frame type"),
            }
        }
    }

    struct Frame {
        ftype: FrameType,
        payload_len: u8,
        payload: Message,
        checksum: u16,
    }

    pub struct Channel {
        port: serialport::TTYPort,
        attempts: u32,
    }

    impl Channel {
        pub fn send(&mut self, message: Message) -> Result<(), &'static str> {
            let f = Frame::new(FrameType::Data, message);
            let mut attempt = 0;
            while attempt < self.attempts {
                let n = match self.port.write(&f.serialize()) {
                    Ok(n) => n,
                    Err(_) => 0,
                };
                println!("N bytes sent: {}", n);
                if n == f.len().into() {
                    //TODO
                    //If NACK, do something (print, log)
                    let mut ack: [u8; 4] = [0; 4]; // no need to read full ack
                    let mut n = 0;
                    self.port.read(&mut ack);
                    println!(
                        "RECV: {:#04x} {:#04x} {:#04x} {:#04x}",
                        ack[0], ack[1], ack[2], ack[3]
                    );
                    if ack[3] == FRAME_ACK {
                        return Ok(());
                    } else if ack[3] == FRAME_NACK {
                        println!("NACK");
                    } else {
                        println!("Unknown err: {}", ack[3]);
                    }
                }
                attempt += 1;
            }
            Err("Maximum number of send attempts reached")
        }

        //TODO: Log intermediate errors
        pub fn listen(&mut self) -> Result<Message, &'static str> {
            let mut attempt: u32 = 0;
            while attempt < self.attempts {
                let mut frame: Vec<u8> = vec![0; MAX_FRAME_SIZE.into()];
                let n = match self.port.read(&mut frame[..3]) {
                    Ok(n) => n,
                    Err(_) => 0,
                };
                if n == 3 && frame[1] == FrameType::Data.value() {
                    let payload_size = *frame.get(2).unwrap() as usize;
                    //TODO Read rest of data and parse
                    let nbytes = match self.port.read(&mut frame[3..payload_size + 3]) {
                        Ok(n) => n,
                        Err(_) => 0,
                    };
                    if nbytes == payload_size + 3 {
                        let f = Frame::deserialize(&frame)?;
                        return Ok(f.payload_copy());
                    }
                }

                attempt += 1;
            }
            Err("Maximum number of read attempts reached")
        }

        pub fn new(port: serialport::TTYPort, attempts: u32) -> Channel {
            Channel { port, attempts }
        }

        //    fn open();
    }

    impl Frame {
        fn serialize(&self) -> Vec<u8> {
            let mut data: Vec<u8> = Vec::new();
            data.push(FRAME_START);
            data.push(self.ftype.value());
            data.push(self.payload.len() as u8);
            data.append(&mut self.payload.serialize());
            data.push((self.checksum & 0xff) as u8);
            data.push((self.checksum >> 8) as u8);
            data.push(FRAME_END);
            data
        }
        fn payload_copy(&self) -> Message {
            // This works for now
            let mut msg = Message::new(match MessageId::from_value(&self.payload.id().value()) {
                Ok(id) => id,
                Err(_) => panic!("Failed id conversion"),
            });
            msg.set_payload(&self.payload.payload().clone());
            msg
        }
        fn payload(&self) -> &Message {
            &self.payload
        }

        fn deserialize(bytes: &[u8]) -> Result<Frame, &'static str> {
            // TODO: 1) Check for start and end bytes
            // 2) Check len < MAX_FRAME_SIZE and Min frame
            // 3) attempt to deserialize
            //
            if bytes.len() > MAX_FRAME_SIZE.into() {
                return Err("Invalid frame: Frame larger than maximum frame size");
            }
            if bytes.len() < MIN_FRAME_SIZE.into() {
                return Err("Invalid frame: Frame less than minimum size");
            }
            let start = match bytes.get(0) {
                Some(n) => n,
                None => return Err("Empty bytes"),
            };
            if *start != FRAME_START {
                return Err("Invalid frame: no frame start value");
            }
            let end = match bytes.get(bytes.len() - 1) {
                Some(n) => n,
                None => return Err("Empty bytes"),
            };
            if *end != FRAME_END {
                return Err("Invalid frame: no frame end value");
            }

            let mut checksum: u16 = *bytes.get(bytes.len() - 3).unwrap() as u16;
            checksum |= (*bytes.get(bytes.len() - 2).unwrap() as u16) << 8;
            if checksum != crc16(&bytes[3..bytes.len() - 3]) {
                return Err("Invalid frame: Failed CRC");
            }

            Ok(Frame {
                ftype: match bytes.get(1) {
                    Some(id) => FrameType::from_value(id)?,
                    None => return Err("Invalid Frame: No type"),
                },
                payload_len: match bytes.get(2) {
                    Some(n) => *n,
                    None => return Err("Invalid Frame: No length"),
                },
                checksum,
                payload: Message::deserialize(&bytes[3..])?,
            })
        }
        fn len(&self) -> u8 {
            self.payload_len + 6
        }
        //fn checksum();
        fn new(ftype: FrameType, message: Message) -> Frame {
            let crc = crc16(&message.serialize());

            Frame {
                ftype,
                payload_len: message.len() as u8, // How do I want to do this?
                payload: message,
                checksum: crc,
            }
        }
        fn ftype(&self) -> &FrameType {
            &self.ftype
        }
    }
    // TODO: make this more 'rust' like
    fn crc16(arr: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        let mut it = arr.iter();
        let mut sz: usize = arr.len();
        while 0 != sz {
            let data: u16 = *it.next().unwrap() as u16;
            crc = crc ^ (data << 8);
            for i in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
            sz = sz - 1;
        }
        crc
    }
    #[cfg(test)]
    mod tests {

        use super::*;
        use crate::message::{Message, MessageId};
        #[test]
        fn test_crc() {
            // TODO: This is ugly..
            let v: Vec<u8> = vec!['A' as u8, 'B' as u8, 'C' as u8, 'D' as u8];
            assert_eq!(0x3b3a, crc16(&v));
        }

        #[test]
        fn new_frame() {
            let msg = Message::new(MessageId::CmdTph);
            let frame = Frame::new(FrameType::Data, msg);
            assert_eq!(FrameType::Data.value(), frame.ftype().value());
            assert_eq!(7, frame.len());
            assert_eq!(MessageId::CmdTph.value(), frame.payload().id().value());
        }

        #[test]
        fn frame_serialize() {
            let expected = vec![
                0x7f, 0x44, 0x05, 0x01, 0x41, 0x42, 0x43, 0x44, 0x6b, 0x91, 0xfe,
            ];
            let v = vec![0x01, 0x41, 0x42, 0x43, 0x44];
            let msg = Message::deserialize(&v).unwrap();
            let frame = Frame::new(FrameType::Data, msg);
            let f = frame.serialize();
            assert_eq!(expected, f);
        }

        #[test]
        fn frame_deserialize() {
            let frame = vec![
                0x7f, 0x44, 0x05, 0x01, 0x41, 0x42, 0x43, 0x44, 0x6b, 0x91, 0xfe,
            ];
            let f = Frame::deserialize(&frame).unwrap();
            assert_eq!(f.ftype().value(), FrameType::Data.value());
            assert_eq!(f.len(), 11);
            assert_eq!(f.payload().id().value(), MessageId::CmdReset.value());
        }
        #[test]
        #[should_panic]
        fn oversized_frame() {
            let v: [u8; 100] = [0x46; 100];
            let f = match Frame::deserialize(&v) {
                Ok(f) => f,
                Err(e) => panic!("Err {}", e),
            };
        }
        #[test]
        #[should_panic]
        fn undersized_frame() {
            let v = [0x45];
            let f = match Frame::deserialize(&v) {
                Ok(f) => f,
                Err(e) => panic!("Err {}", e),
            };
        }

        #[test]
        #[should_panic]
        fn no_frame_start() {
            let v: [u8; 6] = [0x10; 6];
            let f = match Frame::deserialize(&v) {
                Ok(f) => f,
                Err(e) => {
                    println!("E: {}", e);
                    panic!("Err");
                }
            };
        }

        #[test]
        #[should_panic]
        fn no_frame_end() {
            let v: [u8; 6] = [0x7f; 6];
            let f = match Frame::deserialize(&v) {
                Ok(f) => f,
                Err(e) => {
                    println!("E: {}", e);
                    panic!("Err");
                }
            };
        }

        #[test]
        #[should_panic]
        fn no_frame_type() {
            let v: [u8; 6] = [0x7f, 0x44, 0x00, 0x00, 0x00, 0xfe];
            let f = match Frame::deserialize(&v) {
                Ok(f) => f,
                Err(e) => {
                    println!("E: {}", e);
                    panic!("Err");
                }
            };
        }
        #[test]
        #[should_panic]
        fn crc_failure() {
            let frame = vec![
                0x7f, 0x44, 0x05, 0x01, 0x41, 0x42, 0x43, 0x48, 0x6b, 0x91, 0xfe,
            ];

            let f = match Frame::deserialize(&frame) {
                Ok(f) => f,
                Err(e) => {
                    println!("E: {}", e);
                    panic!("Err");
                }
            };
        }
    }
}
