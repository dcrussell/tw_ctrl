pub mod serial {
    use crate::message;
    use serialport;

    enum FrameType {
        Control,
        Data,
    }

    struct Frame {
        ftype: FrameType,
        payload_len: u8,
        payload: message::Message,
        checksum: u16,
    }

    pub struct Channel {}

    //impl Channel {
    //    fn send();
    //    fn listen();
    //    fn new();
    //    fn open();
    //}

    impl Frame {
        fn serialize();
        fn deserialize();
        fn len(&self) -> u8 {
            self.payload_len + 6
        }
        fn checksum();
        fn new(ftype: FrameType, message: message::Message) -> Frame {
            Frame {
                ftype,
                payload_len: message.len(),
                payload: message,
                checksum: crc16(&message.serialize()),
            }
        }
    }
    // TODO: make this more 'rust' like
    fn crc16(arr: &Vec<u8>) -> u16 {
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
        #[test]
        fn test_crc() {
            // TODO: This is ugly..
            let v: Vec<u8> = vec!['A' as u8, 'B' as u8, 'C' as u8, 'D' as u8];
            assert_eq!(0x3b3a, crc16(&v));
        }
    }
}
